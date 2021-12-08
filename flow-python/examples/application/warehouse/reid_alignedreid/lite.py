#!/usr/bin/env python3
# -*- coding:utf-8 -*-
# Copyright (c) Megvii, Inc. and its affiliates.

# import argparse
# import os
import time

import cv2
import megenginelite as mgelite
from .process import preprocess
from loguru import logger
import numpy as np


class PredictorLite(object):
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

    def inference(self, mat):
        t0 = time.time()

        img = preprocess(mat,
                         input_size=(224, 224),
                         scale_im=True,
                         mean=[0.486, 0.459, 0.408],
                         std=[0.229, 0.224, 0.225])

        # build input tensor
        data = img[np.newaxis, :]
        inp_data = self.net.get_io_tensor("data")
        inp_data.set_data_by_copy(data)

        # forward
        self.net.forward()
        self.net.wait()

        # postprocess
        output_keys = self.net.get_all_output_name()
        output = self.net.get_io_tensor(output_keys[0]).to_numpy()
        logger.debug("ReID infer time: {:.4f}s".format(time.time() - t0))
        return output


if __name__ == "__main__":
    import sys
    predictor = PredictorLite(sys.argv[1])
    img2 = cv2.imread(sys.argv[2])
    output2 = predictor.inference(img2)

    img3 = cv2.imread(sys.argv[3])
    output3 = predictor.inference(img3)

    img4 = cv2.imread(sys.argv[4])
    output4 = predictor.inference(img4)

    positive = np.linalg.norm(output2 - output3)
    print(f'distance_positive: {positive}')

    negative = np.linalg.norm(output3 - output4)
    print(f'distance_positive: {negative}')
