# MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
#
# Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

#!/usr/bin/env python
# coding=utf-8

import megengine as mge
import megengine.module as nn
import megengine.module.init as init
import megengine.functional as F
import time
from loguru import logger

from .resnet import resnet50, resnet18
from .process import preprocess


class Model(nn.Module):
    def __init__(self):
        super(Model, self).__init__()
        self.base = resnet50()
        # planes = 2048

    def forward(self, x):
        """
    Returns:
      global_feat: shape [N, C]
      local_feat: shape [N, H, c]
    """
        # shape [N, C, H, W]
        feat = self.base(x)
        feat = F.avg_pool2d(feat, (feat.shape[2], feat.shape[3]))
        # shape [N, C]
        feat = F.flatten(feat, 1)
        feat = F.normalize(feat, axis=1)
        return feat

    def inference(self, mat):
        t0 = time.time()
        img = preprocess(mat,
                         input_size=(224, 224),
                         scale_im=True,
                         mean=[0.486, 0.459, 0.408],
                         std=[0.229, 0.224, 0.225])
        img = F.expand_dims(mge.tensor(img), 0)
        feat = self.forward(img)
        logger.info("AlignedReID infer time: {:.4f}s".format(time.time() - t0))

        return feat[0].numpy()
