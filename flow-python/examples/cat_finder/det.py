# MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
#
# Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

#!/usr/bin/env python
# coding=utf-8

import numpy as np
from loguru import logger
from megflow import register
from warehouse.detection_yolox import PredictorLite


@register(inputs=['inp'], outputs=['out'])
class Detect:
    def __init__(self, name, args):
        logger.info("loading YOLOX detection...")
        self._tsize = args['tsize']
        self._interval = args['interval']
        self._visualize = args['visualize']
        self.name = name

        # load detect model and warmup
        self._predictor = PredictorLite(path=args['path'],
                                        confthre=args['conf'],
                                        nmsthre=args['nms'],
                                        test_size=(self._tsize, self._tsize),
                                        device=args['device'],
                                        device_id=args['device_id'])
        warmup_data = np.zeros((224, 224, 3), dtype=np.uint8)
        self._predictor.inference(warmup_data)
        logger.info(" YOLOX loaded.")

    @staticmethod
    def restrict(val, min, max):
        assert min < max
        if val < min:
            val = min
        if val > max:
            val = max
        return val

    def exec(self):
        envelope = self.inp.recv()
        if envelope is None:
            return

        msg = envelope.msg
        msg['items'] = []

        process = envelope.partial_id % self._interval == 0
        if process:
            data = msg['data']
            outputs = self._predictor.inference(data)
            # skip if detect nothing
            if outputs is not None:
                items = []

                for i in range(outputs.shape[0]):
                    output = outputs[i]
                    # if neither cat nor dog, skip
                    if round(output[6]) != 15 and round(output[6]) != 16:
                        continue

                    item = dict()
                    item["bbox"] = output[0:4]
                    item["cls"] = round(output[6])
                    item["score"] = output[4] * output[5]
                    items.append(item)
                msg['items'] = items

                # import cv2
                # x = self._predictor.visual(outputs, data)
                # name = 'frame{0:07d}.jpg'.format(envelope.partial_id)
                # cv2.imwrite(name, x)

            if self._visualize == 1:
                msg['data'] = self._predictor.visual(outputs, data)
        msg['process'] = process
        self.out.send(envelope)
