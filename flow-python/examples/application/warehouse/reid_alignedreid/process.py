# MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
#
# Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

#!/usr/bin/env python3
# -*- coding:utf-8 -*-

import cv2
import megengine.functional as F
import numpy as np
from loguru import logger

__all__ = ["preprocess"]


def preprocess(image, input_size, scale_im, mean, std, swap=(2, 0, 1)):
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
