#!/usr/bin/env python3
# -*- coding:utf-8 -*-
# Copyright (c) Megvii, Inc. and its affiliates.

import argparse
import os
import time

import cv2
import megengine as mge
import megenginelite as mgelite
from loguru import logger

from .coco_classes import COCO_CLASSES
from .process import postprocess, preprocess
from .visualize import vis
import numpy as np

IMAGE_EXT = [".jpg", ".jpeg", ".webp", ".bmp", ".png"]


def make_parser():
    parser = argparse.ArgumentParser("YOLOX Demo!")
    parser.add_argument("-n",
                        "--name",
                        type=str,
                        default="yolox-s",
                        help="model name")
    parser.add_argument("--path",
                        default="./test.png",
                        help="path to images or video")

    parser.add_argument("-c",
                        "--ckpt",
                        default=None,
                        type=str,
                        help="ckpt for eval")
    parser.add_argument("--conf", default=None, type=float, help="test conf")
    parser.add_argument("--nms",
                        default=None,
                        type=float,
                        help="test nms threshold")
    parser.add_argument("--tsize",
                        default=None,
                        type=int,
                        help="test img size")
    return parser


def get_image_list(path):
    image_names = []
    for maindir, subdir, file_name_list in os.walk(path):
        for filename in file_name_list:
            apath = os.path.join(maindir, filename)
            ext = os.path.splitext(apath)[1]
            if ext in IMAGE_EXT:
                image_names.append(apath)
    return image_names


class PredictorLite(object):
    def __init__(
        self,
        path,
        confthre=0.01,
        nmsthre=0.65,
        test_size=(640, 640),
        cls_names=COCO_CLASSES,
        trt_file=None,
        decoder=None,
        device="gpu",
        device_id=0,
    ):

        if "gpu" in device.lower():
            device_type = mgelite.LiteDeviceType.LITE_CUDA
        else:
            device_type = mgelite.LiteDeviceType.LITE_CPU

        net_config = mgelite.LiteConfig(device_type=device_type)
        ios = mgelite.LiteNetworkIO()
        ios.add_input(mgelite.LiteIO("data", is_host=True))
        # ios.add_input(LiteIO("idx", is_host=True))
        # ios.add_input(LiteIO("roi", is_host=True))

        net = mgelite.LiteNetwork(config=net_config, io=ios)
        net.device_id = device_id
        net.load(path)

        self.net = net
        self.cls_names = cls_names
        self.decoder = decoder
        self.num_classes = 80
        self.confthre = confthre
        self.nmsthre = nmsthre
        self.test_size = test_size
        self.rgb_means = (0.485, 0.456, 0.406)
        self.std = (0.229, 0.224, 0.225)

    def lite_postprocess(self, outputs, img_size, p6=False):
        grids = []
        expanded_strides = []

        if not p6:
            strides = [8, 16, 32]
        else:
            strides = [8, 16, 32, 64]

        hsizes = [img_size[0] // stride for stride in strides]
        wsizes = [img_size[1] // stride for stride in strides]

        for hsize, wsize, stride in zip(hsizes, wsizes, strides):
            xv, yv = np.meshgrid(np.arange(wsize), np.arange(hsize))
            grid = np.stack((xv, yv), 2).reshape(1, -1, 2)
            grids.append(grid)
            shape = grid.shape[:2]
            expanded_strides.append(np.full((*shape, 1), stride))

        grids = np.concatenate(grids, 1)
        expanded_strides = np.concatenate(expanded_strides, 1)
        outputs[..., :2] = (outputs[..., :2] + grids) * expanded_strides
        outputs[..., 2:4] = np.exp(outputs[..., 2:4]) * expanded_strides

        return outputs

    def restrict(self, val, min, max):
        assert (min < max)
        if val < min:
            val = min
        if val > max:
            val = max
        return val

    def inference(self, img):
        t0 = time.time()

        (max_h, max_w, _) = img.shape

        # preprocess
        img, ratio = preprocess(img, self.test_size, self.rgb_means, self.std)

        # build input tensor
        data = img[np.newaxis, :]
        inp_data = self.net.get_io_tensor("data")
        inp_data.set_data_by_copy(data)

        # forward
        self.net.forward()
        self.net.wait()

        # postprocess
        output_keys = self.net.get_all_output_name()
        outputs = self.net.get_io_tensor(output_keys[0]).to_numpy()

        outputs = self.lite_postprocess(outputs[0], list(self.test_size))
        outputs = outputs[np.newaxis, :]
        output = mge.tensor(outputs)

        ret = postprocess(output, self.num_classes, self.confthre,
                          self.nmsthre)
        if ret is None:
            return None
        bboxes = ret.copy()
        for i in range(bboxes.shape[0]):
            bbox = bboxes[i][0:4] / ratio
            bbox[0] = self.restrict(bbox[0], 0, max_w)
            bbox[1] = self.restrict(bbox[1], 0, max_h)
            bbox[2] = self.restrict(bbox[2], bbox[0], max_w)
            bbox[3] = self.restrict(bbox[3], bbox[1], max_h)
            bboxes[i][0:4] = bbox

        logger.debug("YOLOX infer time: {:.4f}s".format(time.time() - t0))
        return bboxes

    def visual(self, output, img, cls_conf=0.35):
        if output is None:
            return img

        # preprocessing: resize
        bboxes = output[:, 0:4]
        cls = output[:, 6]
        scores = output[:, 4] * output[:, 5]

        vis_res = vis(img, bboxes, scores, cls, cls_conf, self.cls_names)
        return vis_res


def main(args):
    dirname = os.path.join("./yolox_outputs", args.name)
    os.makedirs(dirname, exist_ok=True)

    confthre = 0.01
    nmsthre = 0.65
    test_size = (640, 640)
    if args.conf is not None:
        confthre = args.conf
    if args.nms is not None:
        nmsthre = args.nms
    if args.tsize is not None:
        test_size = (args.tsize, args.tsize)

    predictor = PredictorLite(args.ckpt, confthre, nmsthre, test_size,
                              COCO_CLASSES, None, None, "gpu", 0)

    frame = cv2.imread(args.path)
    outputs = predictor.inference(frame)
    result_frame = predictor.visual(outputs, frame)

    filename = os.path.join(dirname, "out.jpg")
    cv2.imwrite(filename, result_frame)


if __name__ == "__main__":
    args = make_parser().parse_args()
    main(args)
