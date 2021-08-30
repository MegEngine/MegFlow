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

class Quality:
    
    @staticmethod
    def clearness(mat):
        return cv2.Laplacian(mat, cv2.CV_64F).var()

    @staticmethod
    def area(mat):
        return mat.shape[0] * mat.shape[1]

if __name__ == "__main__":
    import sys
    mat = cv2.imread(sys.argv[1])
    if mat is None:
        print(f'load {sys.argv[1]} failed')
        sys.exit(-1)
    print(f'brightness: {Quality.clearness(mat)}')

