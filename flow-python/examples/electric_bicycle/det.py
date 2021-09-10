# MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
#
# Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

#!/usr/bin/env python
# coding=utf-8

import cv2
import numpy as np
from megflow import register
from loguru import logger
from warehouse.detection_memd import *

@register(inputs=['inp'], outputs=['out'])
class Detect:
    def __init__(self, name, args):
        logger.info("loading MEMD detection...")
        self._nms = args['nms_thres']
        self._score = args['score_thres']
        self._interval = args['interval']
        self._visualize = args['visualize']
        self.name = name

        # load model and warmup
        self._model = load_onnx_model(args['path'])
        warmup_data = np.zeros((256, 256, 3), dtype=np.uint8)
        onnx_model.run(self._model, warmup_data, ["elec_cycle"], self._score, self._nms)
        
        logger.info(" MEMD loaded.")

    @staticmethod
    def restrict(val, min, max):
        assert(min < max)
        if val < min:
            val = min
        if val > max:
            val = max
        return round(val)

    def exec(self):
        envelope = self.inp.recv()
        if envelope is None:
            return
        image = envelope.msg

        process = envelope.partial_id % self._interval == 0
        image['items'] = []

        if process:
            data = image['data']
            outputs = onnx_model.run(self._model, data, ["elec_cycle"], self._score, self._nms)

            items = []
            (max_h, max_w, _) = data.shape
            for output in outputs:
                item = dict()
                bbox = output[0:4]
                bbox[0] = self.restrict(bbox[0], 0, max_w)
                bbox[1] = self.restrict(bbox[1], 0, max_h)
                bbox[2] = self.restrict(bbox[2], bbox[0], max_w)
                bbox[3] = self.restrict(bbox[3], bbox[1], max_h)
                item["bbox"] = bbox
                item["cls"] = 0
                item["score"] = output[4]
                items.append(item)
            image['items'] = items

            self.out.send(envelope)
        
