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
import cv2
import numpy as np
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
        ios.add_input(mgelite.LiteIO("##INP##", is_host=True))

        net = mgelite.LiteNetwork(config=net_config, io=ios)
        net.device_id = device_id
        net.load(path)

        inp_shape = net.get_io_tensor("##INP##").to_numpy().shape
        self.HEIGHT = inp_shape[2]
        self.WIDTH = inp_shape[3]

        self.net = net

    def inference(self, img):
        t0 = time.time()

        img = cv2.resize(img, (self.HEIGHT, self.WIDTH))
        # build input tensor
        inp_data = self.net.get_io_tensor("##INP##")
        inp_data.set_data_by_share(img)

        # forward
        self.net.forward()
        self.net.wait()

        # postprocess
        output_keys = self.net.get_all_output_name()
        output_dict = dict()
        for key in output_keys:
            value = self.net.get_io_tensor(key).to_numpy().tolist()
            output_dict[key] = value
        logger.debug("timecost: {:.4f}s".format(time.time() - t0))
        return json.dumps(output_dict)


def make_parser():
    parser = argparse.ArgumentParser("Single Model Demo!")
    parser.add_argument("--path", default="./test.png", help="path to image")
    parser.add_argument("--model",
                        default=None,
                        type=str,
                        help=".mge for eval")
    return parser


if __name__ == "__main__":
    args = make_parser().parse_args()
    predictor = PredictorLite(args.model)
    image = cv2.imread(args.path)
    out = predictor.inference(image)
    print(out)
