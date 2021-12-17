# MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
#
# Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

#!/usr/bin/env python
# coding=utf-8

import json
import numpy as np
import cv2
from loguru import logger
from megflow import register
from .lite import PredictorLite


@register(inputs=['inp'], outputs=['out'])
class Classify:
    def __init__(self, name, arg):
        logger.info("loading Resnet18 Classification...")
        self.name = name
        self.batch_size = arg['max_batch']
        self.timeout = arg['wait_time']

        # load ReID model and warmup
        self._model = PredictorLite(path=arg['path'],
                                    device=arg['device'],
                                    device_id=arg['device_id'])
        warmup_data = np.zeros((224, 224, 3), dtype=np.uint8)
        self._model.inference(warmup_data)
        logger.info("Resnet18  loaded.")

    def expand(self, box, max_w, max_h, ratio):
        l = box[0]
        r = box[2]
        t = box[1]
        b = box[3]
        center_x = (l + r) / 2
        center_y = (t + b) / 2
        w_side = (r - l) * ratio / 2
        h_side = (b - t) * ratio / 2

        l = center_x - w_side
        r = center_x + w_side
        t = center_y - h_side
        b = center_y + h_side
        l = max(0, l)
        t = max(0, t)
        r = min(max_w, r)
        b = min(max_h, b)
        return int(l), int(t), int(r), int(b)

    def exec(self):
        # batching
        (envelopes, _) = self.inp.batch_recv(self.batch_size, self.timeout)

        if len(envelopes) == 0:
            return

        crops = []
        for env in envelopes:
            data = env.msg['data']
            items = env.msg['items']
            for item in items:
                assert 'bbox' in item
                bbox = item['bbox']
                l, t, r, b = self.expand(bbox, data.shape[1], data.shape[0],
                                         1.1)
                crop = cv2.resize(data[t:b, l:r], (224, 224))
                crops.append(crop[np.newaxis, :])
        if len(crops) > 0:
            data = np.concatenate(crops)
            types = self._model.inference_batch(data)
            for _type in types:
                self.out.send(envelopes[0].repack(json.dumps(str(_type))))

    # batch_size == 1
    # def exec(self):
    #     envelope = self.inp.recv()
    #     if envelope is None:
    #         return

    #     data = envelope.msg['data']
    #     items = []
    #     results = []
    #     if 'items' in envelope.msg:
    #         items = envelope.msg['items']
    #     for _, item in enumerate(items):
    #         assert 'bbox' in item
    #         bbox = item['bbox']
    #         l, t, r, b = self.expand(bbox, data.shape[1], data.shape[0], 1.1)
    #         _type = self._model.inference(data[t:b, l:r])
    #         results.append({
    #             "type": str(_type),
    #             "frame_id": str(envelope.partial_id)
    #         })
    #     self.out.send(envelope.repack(json.dumps(results)))
