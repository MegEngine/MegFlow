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

from warehouse.reid_alignedreid import PredictorLite


@register(inputs=['inp'], outputs=['out'])
class ReIDImage:
    def __init__(self, name, args):
        logger.info("loading Image Reid...")
        self.name = name

        # load ReID model and warmup
        self._model = PredictorLite(path=args['path'],
                                    device=args['device'],
                                    device_id=args['device_id'])
        warmup_data = np.zeros((224, 224, 3), dtype=np.uint8)
        self._model.inference(warmup_data)
        logger.info("Image Reid loaded.")

    def exec(self):
        envelope = self.inp.recv()
        if envelope is None:
            return
        image = envelope.msg

        data = image['data']
        items = image['items']
        assert isinstance(items, list)

        for item in items:
            bbox = item['bbox']
            crop = data[round(bbox[1]):round(bbox[3]),
                        round(bbox[0]):round(bbox[2])]

            # cv2.imwrite(f'reid_image_{envelope.partial_id}.jpg', crop)
            feature = self._model.inference(crop)
            item['feature'] = feature
            logger.info(f'feature {feature}')
        self.out.send(envelope)
