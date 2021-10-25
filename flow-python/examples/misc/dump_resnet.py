# -*- coding: utf-8 -*-
# MegEngine is Licensed under the Apache License, Version 2.0 (the "License")
#
# Copyright (c) 2014-2021 Megvii Inc. All rights reserved.
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

# pylint: skip-file
import argparse
import sys

# pylint: disable=import-error
import resnet.model as resnet_model
# pylint: disable=import-error
import shufflenet.model as snet_model

import numpy as np

import megengine as mge
import megengine.functional as F
from megengine import jit


def dump_static_graph(model, graph_name, shape):
    model.eval()

    data = mge.Tensor(np.ones(shape, dtype=np.uint8))

    @jit.trace(capture_as_const=True)
    def pred_func(data):
        out = data.astype(np.float32)

        output_h, output_w = 224, 224
        # resize
        print(shape)
        M = mge.tensor(np.array([[1,0,0], [0,1,0], [0,0,1]], dtype=np.float32))        
        M_shape = F.concat([data.shape[0],M.shape])
        M = F.broadcast_to(M, M_shape)
        out = F.vision.warp_perspective(out, M, (output_h, output_w), format='NHWC')
        # mean
        _mean = mge.Tensor(np.array([103.530, 116.280, 123.675], dtype=np.float32))
        out = F.sub(out, _mean)
        # div 
        _div = mge.Tensor(np.array([57.375, 57.120, 58.395], dtype=np.float32))
        out = F.div(out, _div)
        # dimshuffile 
        out = F.transpose(out, (0,3,1,2))

        outputs = model(out)
        return outputs

    pred_func(data)
    pred_func.dump(
        graph_name,
        arg_names=["data"],
        optimize_for_inference=True,
        enable_fuse_conv_bias_nonlinearity=True,
    )


def main():
    parser = argparse.ArgumentParser(description="MegEngine Classification Dump .mge")
    parser.add_argument(
        "-a",
        "--arch",
        default="resnet18",
        help="model architecture (default: resnet18)",
    )
    parser.add_argument(
        "-s",
        "--shape",
        type=int,
        nargs='+',
        default="1 3 224 224",
        help="input shape (default: 1 3 224 224)"
    )
    parser.add_argument(
        "-o",
        "--output",
        type=str,
        default="model.mge",
        help="output filename"
    )

    args = parser.parse_args()
    if 'resnet' in args.arch:
        model = getattr(resnet_model, args.arch)(pretrained=True)
    elif 'shufflenet' in args.arch:
        model = getattr(snet_model, args.arch)(pretrained=True)
    else:
        print('unavailable arch {}'.format(args.arch))
        sys.exit()
    print(model)
    dump_static_graph(model, args.output, tuple(args.shape))


if __name__ == "__main__":
    main()
