# MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
#
# Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

#!/usr/bin/env python
# coding=utf-8

import argparse
import time
import numpy as np
import cv2
import megenginelite as mgelite
from loguru import logger


class PredictorLite:
    def __init__(
        self,
        path,
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

        net = mgelite.LiteNetwork(config=net_config, io=ios)
        net.device_id = device_id
        net.load(path)

        self.net = net

    def preprocess(self,
                   image,
                   input_size,
                   scale_im,
                   mean,
                   std,
                   swap=(2, 0, 1)):
        if image is None:
            logger.error("image is None")
            return image
        image = cv2.resize(image, input_size)
        image = image.astype(np.float32)
        image = image[:, :, ::-1]
        if scale_im:
            image /= 255.0
        if mean is not None:
            image -= mean
        if std is not None:
            image /= std
        image = image.transpose(swap)
        image = np.ascontiguousarray(image, dtype=np.float32)
        return image

    def inference(self, mat):
        t0 = time.time()

        img = self.preprocess(mat,
                              input_size=(224, 224),
                              scale_im=False,
                              mean=[103.530, 116.280, 123.675],
                              std=[57.375, 57.120, 58.395])

        # build input tensor
        inp_data = self.net.get_io_tensor("data")
        inp_data.set_data_by_share(img)

        # forward
        self.net.forward()
        self.net.wait()

        # postprocess
        output_keys = self.net.get_all_output_name()
        output = self.net.get_io_tensor(output_keys[0]).to_numpy()
        logger.debug("resnet18 infer time: {:.4f}s".format(time.time() - t0))

        return np.argmax(output[0])


def make_parser():
    parser = argparse.ArgumentParser("Classification Demo!")
    parser.add_argument("--path",
                        default="./test.png",
                        help="path to images or video")
    parser.add_argument("--model",
                        default=None,
                        type=str,
                        help=".mge for eval")
    return parser


if __name__ == "__main__":
    args = make_parser().parse_args()
    predictor = PredictorLite(args.model)
    out = predictor.inference(cv2.imread(args.path))
    logger.info(f'{out}')
