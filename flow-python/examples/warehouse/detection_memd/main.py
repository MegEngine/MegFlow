# MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
#
# Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

#!/usr/bin/env python
# coding=utf-8

import sys 
import json
import argparse

import cv2
from loguru import logger

sys.path.append('.')
from .onnx_model import load_onnx_model, run

def inference(detector_path, img, output_path):
    class_names = ["elec_cycle"]
    onnx_session = load_onnx_model(detector_path)

    score_thrs = 0.05
    nms_thrs = 0.6

    score_thrs = max(score_thrs, 0.5)

    res = run(onnx_session, img, class_names, score_thrs, nms_thrs)

    for box in res["boxes"]:
        cv2.rectangle(
            img,
            (int(box["x"]), int(box["y"])),
            (int(box["x"] + box["w"]), int(box["y"] + box["h"])),
            (0, 255, 0),
            2,
        )
        cv2.putText(
            img,
            "{}:{:.2f}".format(box["class_name"], box["score"]),
            (int(box["x"] - 10), int(box["y"] - 10)),
            cv2.FONT_HERSHEY_SIMPLEX,
            1,
            (0, 255, 0),
            2,
        )

    if not output_path.endswith((".jpg", ".png")):
        output_path = f"{output_path}.jpg"

    cv2.imwrite(output_path, img)
    logger.info(f"Inference on an image and write to {output_path}")

    box_num = len(res["boxes"])

    return box_num

def parse_args():
    parser = argparse.ArgumentParser("Electric Moped Detector")
    parser.add_argument(
        "--detector", default="./models/model.onnx", help="The path to onnx detector. "
    )
    parser.add_argument(
        "--input-img", default="./demo/input.jpg", help="The path to demo image to inference. "
    )
    parser.add_argument(
        "--output-path", default="./demo/output.jpg", help="The path of output images. "
    )
    args = parser.parse_args()

    return args

def main():
    args = parse_args()
    img = cv2.imread(args.input_img)
    inference(args.detector, img, args.output_path)


if __name__ == "__main__":
    main()
