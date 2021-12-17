# MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
#
# Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

#!/usr/bin/env python
# coding=utf-8

import argparse
import megengine as mge
import numpy as np
from megengine import jit
from .model import Model


def make_parser():
    parser = argparse.ArgumentParser("Resnet50 Dump")
    parser.add_argument("-c",
                        "--ckpt",
                        default=None,
                        type=str,
                        help="ckpt for eval")
    parser.add_argument("--dump_path",
                        default="model.mge",
                        help="path to save the dumped model")
    return parser


def dump_static_graph(model, graph_name="model.mge"):
    model.eval()

    data = mge.Tensor(np.random.random((1, 3, 224, 224)))

    @jit.trace(capture_as_const=True)
    def pred_func(data):
        outputs = model(data)
        return outputs

    pred_func(data)
    pred_func.dump(
        graph_name,
        arg_names=["data"],
        optimize_for_inference=True,
        enable_fuse_conv_bias_nonlinearity=True,
    )


def main(args):
    model = Model()
    sd = mge.load(args.ckpt)
    model.load_state_dict(sd, strict=False)
    model.eval()

    dump_static_graph(model, args.dump_path)


if __name__ == "__main__":
    args = make_parser().parse_args()
    main(args)
