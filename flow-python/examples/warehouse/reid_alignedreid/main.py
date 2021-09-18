# MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
#
# Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

#!/usr/bin/env python
# coding=utf-8

from math import log
from loguru import logger
import megengine as mge
import cv2
import megengine.functional as F
import numpy as np

from .model import Model

if __name__ == "__main__":
    import sys
    if len(sys.argv) < 5:
        print(
            "usage: python3 -m reid_alignedreid/demo  reid.pkl  positive1.png  positive2.png  negtive.jpg"
        )
        sys.exit(0)
    model = Model()
    sd = mge.load(sys.argv[1])
    model.load_state_dict(sd, strict=False)
    model.eval()
    feat1 = model.inference(cv2.imread(sys.argv[2]))
    logger.info(f'{feat1}')
    feat2 = model.inference(cv2.imread(sys.argv[3]))
    logger.info(f'{feat2}')

    feat3 = model.inference(cv2.imread(sys.argv[4]))
    logger.info(f'{feat3}')

    positive = np.linalg.norm(feat1 - feat2)
    print(f'distance_positive: {positive}')

    negtive = np.linalg.norm(feat3 - feat2)
    print(f'distance_negtive: {negtive}')
