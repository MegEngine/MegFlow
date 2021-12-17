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
from loguru import logger
from megflow import register

from .lite import PredictorLite


@register(inputs=['inp'], outputs=['out'])
class Classify:
    def __init__(self, name, arg):
        logger.info("loading Resnet18 Classification...")
        self.name = name

        # load ReID model and warmup
        self._model = PredictorLite(path=arg['path'],
                                    device=arg['device'],
                                    device_id=arg['device_id'])
        warmup_data = np.zeros((224, 224, 3), dtype=np.uint8)
        self._model.inference(warmup_data)
        logger.info("Resnet18  loaded.")

    def exec(self):
        envelope = self.inp.recv()
        if envelope is None:
            return

        data = envelope.msg['data']
        result = self._model.inference(data)
        self.out.send(envelope.repack(json.dumps(str(result))))
