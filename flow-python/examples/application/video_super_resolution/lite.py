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
import os
import cv2
import numpy as np
import megenginelite as mgelite


class PredictorLite:
    def load(self, fullpath, config, device_id):
        net = mgelite.LiteNetwork(config=config)
        net.device_id = device_id
        print(fullpath)
        net.load(fullpath)
        return net

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

        self.flownet = self.load(os.path.join(path, "flownet.mge"), net_config,
                                 device_id)
        self.gen = self.load(os.path.join(path, "generator.mge"), net_config,
                             device_id)
        self.upsample = self.load(os.path.join(path, "upsample.mge"),
                                  net_config, device_id)
        self.HIDDEN_CHANNELS = 96
        print("basicVSR model loaded.")

    def get_bilinear(self, image):
        B, T, C, h, w = image.shape
        image = image.reshape(-1, C, h, w)
        ret = np.zeros((image.shape[0], C, 4 * h, 4 * w), dtype=np.float32)
        for i in range(image.shape[0]):
            chw = image[i:i + 1].reshape(C, h, w)
            hwc = np.transpose(chw, (1, 2, 0))
            hwc = cv2.resize(hwc, (w * 4, h * 4))
            ret[i:i + 1] = np.transpose(hwc, (2, 0, 1))
        ret = ret.reshape(B, T, C, h * 4, w * 4)
        return ret

    def inference_flownet(self, now_frame, ref):
        begin = time.time()

        data0 = self.flownet.get_io_tensor("tenFirst")
        data0.set_data_by_share(now_frame)

        data1 = self.flownet.get_io_tensor("tenSecond")
        data1.set_data_by_share(ref)
        self.flownet.forward()
        self.flownet.wait()

        tensor = self.flownet.get_io_tensor(
            self.flownet.get_all_output_name()[0])
        timecost = time.time() - begin
        print(f"flownet timecost {timecost} ms")
        return tensor.to_numpy()

    def inference_gen(self, hidden, flow, nowFrame):
        begin = time.time()

        data0 = self.gen.get_io_tensor("hidden")
        data0.set_data_by_share(hidden)

        data1 = self.gen.get_io_tensor("flow")
        data1.set_data_by_share(flow)

        data2 = self.gen.get_io_tensor("nowFrame")
        data2.set_data_by_share(nowFrame)

        self.gen.forward()
        self.gen.wait()

        tensor = self.gen.get_io_tensor(self.gen.get_all_output_name()[0])
        timecost = time.time() - begin
        print(f"gen timecost {timecost} ms")
        return tensor.to_numpy()

    def inference_upsample(self, forward_hidden, backward_hidden):
        begin = time.time()

        data0 = self.upsample.get_io_tensor("forward_hidden")
        data0.set_data_by_share(forward_hidden)

        data1 = self.upsample.get_io_tensor("backward_hidden")
        data1.set_data_by_share(backward_hidden)

        self.upsample.forward()
        self.upsample.wait()

        tensor = self.upsample.get_io_tensor(
            self.upsample.get_all_output_name()[0])
        timecost = time.time() - begin
        print(f"upsample timecost {timecost} ms")
        return tensor.to_numpy()

    # shape [batch, 3, H, W]
    def inference(self, input):
        input = input.astype(np.float32) / 255.0
        input = np.expand_dims(input, axis=0)  # [1,100,3,180,320]

        image = np.ascontiguousarray(input, np.float32)

        B, T, _, h, w = image.shape
        biup = self.get_bilinear(image)
        forward_hiddens = []
        backward_hiddens = []
        res = []
        hidden = np.zeros((2 * B, self.HIDDEN_CHANNELS, h, w),
                          dtype=np.float32)
        for i in range(T):
            now_frame = np.concatenate(
                [image[:, i, ...], image[:, T - i - 1, ...]], axis=0)
            if i == 0:
                flow = self.inference_flownet(now_frame, now_frame)
            else:
                ref = np.concatenate(
                    [image[:, i - 1, ...], image[:, T - i, ...]], axis=0)
                flow = self.inference_flownet(now_frame, ref)

            hidden = self.inference_gen(hidden, flow, now_frame)
            forward_hiddens.append(hidden[0:B, ...])
            backward_hiddens.append(hidden[B:2 * B, ...])

        for i in range(T):
            res.append(
                self.inference_upsample(forward_hiddens[i],
                                        backward_hiddens[T - i - 1]))

        res = np.stack(res, axis=1)  # [B,T,3,H,W]
        HR_G = res + biup
        HR_G = (np.clip(HR_G, a_min=0.0, a_max=1.0) * 255.0).round().astype(
            np.uint8)

        ret = []
        for i in range(T):
            x = HR_G[0, i, ...]
            img_np = np.transpose(x[[2, 1, 0], :, :], (1, 2, 0))
            ret.append(img_np)
        return ret


def make_parser():
    parser = argparse.ArgumentParser("ModelServing Demo!")
    parser.add_argument("--model",
                        default=None,
                        type=str,
                        help=".mge for eval")
    return parser


# if __name__ == "__main__":

#     batchdata = None
#     imagelist = []
#     for parent, _, filenames in os.walk("images"):
#         filenames.sort()
#         for filename in filenames:
#             mat = cv2.imread(os.path.join(parent, filename))
#             print(filename)
#             mat = cv2.cvtColor(mat, cv2.COLOR_BGR2RGB)
#             mat = np.transpose(mat, (2, 0, 1))
#             mat = np.expand_dims(mat, axis=0)
#             if mat is not None:
#                 imagelist.append(mat[...])
#     batchdata = np.concatenate(imagelist, axis=0)
#     assert(batchdata is not None)

#     predictor = PredictorLite("./")
#     result = predictor.inference(batchdata)
#     assert(len(result) == batchdata.shape[0])

#     for idx, image in enumerate(result):
#         cv2.imwrite(f"{idx}.jpg", image)
