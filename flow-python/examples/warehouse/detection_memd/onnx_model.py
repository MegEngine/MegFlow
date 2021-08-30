# MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
#
# Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

#!/usr/bin/env python
# coding=utf-8

from loguru import logger
import onnxruntime
import cv2 
import numpy as np 

def load_onnx_model(onnx_path): 
    onnx_session = onnxruntime.InferenceSession(onnx_path)
    return onnx_session

def get_output_name(onnx_session):
    output_name = []
    for node in onnx_session.get_outputs():
        output_name.append(node.name)
    return output_name

def transform(image, target_shape=(960, 960)):
    image_height, image_width, _ = image.shape
    ratio_h = target_shape[1] * 1.0 / image_height
    ratio_w = target_shape[0] * 1.0 / image_width
    image = cv2.resize(image, target_shape)
    return image, ratio_h, ratio_w

def is_overlap_v1(rect1, rect2, iou_threshold):
    xx1 = max(rect1[0], rect2[0])
    yy1 = max(rect1[1], rect2[1])
    xx2 = min(rect1[2], rect2[2])
    yy2 = min(rect1[3], rect2[3])
    dx = max(0, xx2 - xx1 + 1)
    dy = max(0, yy2 - yy1 + 1)
    i = dx * dy
    u = (rect1[2] - rect1[0] + 1) * (rect1[3] - rect1[1] + 1) + (
        rect2[2] - rect2[0] + 1) * (rect2[3] - rect2[1] + 1) - i
    ov = i / u
    return ov >= iou_threshold

def raw_nms(boxes, iou_threshold=0.3):
    if 0 == len(boxes):
        return []
    rects = list(boxes)
    for i in range(len(rects)):
        rects[i] = list(rects[i])
        rects[i].append(i)

    rects.sort(key=lambda x: x[4], reverse=True)

    rect_valid = [True for i in range(len(rects))]
    for i in range(len(rects)):
        if rect_valid[i]:
            j = i + 1
            while j < len(rect_valid):
                if is_overlap_v1(rects[i], rects[j], iou_threshold):
                    rect_valid[j] = False
                j = j + 1

    return [x[5] for i, x in enumerate(rects) if rect_valid[i]]

def onnx_inference(onnx_session, num_classes, image, topk_candidates=1000): 

    output_name = get_output_name(onnx_session)

    image, ratio_h, ratio_w = transform(image)
    image = image.astype(np.float32)
    image = np.expand_dims(image.transpose((2, 0, 1)), 0)

    scores, boxes = onnx_session.run(
        output_name, input_feed={"input": image}
    )

    keep = scores.max(axis=1) > 0.1
    scores = scores[keep]
    boxes = boxes[keep]

    scores = scores.flatten()
    # Keep top k top scoring indices only.
    num_topk = min(topk_candidates, len(boxes))
    # torch.sort is actually faster than .topk (at least on GPUs)
    topk_idxs = np.argsort(scores)

    scores = scores[topk_idxs][-num_topk:]
    topk_idxs = topk_idxs[-num_topk:]

    # filter out the proposals with low confidence score
    shift_idxs = topk_idxs // num_classes
    classes = topk_idxs % num_classes
    boxes = boxes[shift_idxs]

    boxes[:, 0] /= ratio_w
    boxes[:, 1] /= ratio_h
    boxes[:, 2] /= ratio_w
    boxes[:, 3] /= ratio_h

    return boxes, scores, classes

def run(onnx_session, image, class_names, score_thrs, nms_thr=0.6):
    num_classes = len(class_names)
    import time
    t0 = time.time()
    boxes, scores, cls_idxs = onnx_inference(onnx_session, num_classes, image)
    cost = time.time() - t0
    logger.info(f'memd inference: {cost}s')

    assert len(boxes) == len(scores) and len(boxes) == len(cls_idxs)

    if isinstance(score_thrs, float):
        keep = scores > max(score_thrs, 0.2)
    else:
        score_thrs = np.asarray(score_thrs)
        keep = scores > np.maximum(score_thrs[cls_idxs], 0.2)

    pred_boxes = np.concatenate(
        [boxes, scores[:, np.newaxis], cls_idxs[:, np.newaxis]], axis=1
    )
    pred_boxes = pred_boxes[keep]

    all_boxes = []
    for cls_idx in range(len(class_names)):
        keep_per_cls = pred_boxes[:, -1] == cls_idx
        if keep_per_cls.sum() > 0:
            pred_boxes_per_cls = pred_boxes[keep_per_cls].astype(np.float32)
            keep_idx = raw_nms(pred_boxes_per_cls[:, :5], nms_thr)
            for idx in keep_idx:
                all_boxes.append(pred_boxes_per_cls[idx])
    return all_boxes
