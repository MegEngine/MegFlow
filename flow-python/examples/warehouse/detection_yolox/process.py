#!/usr/bin/env python3
# -*- coding:utf-8 -*-
# Copyright (c) 2014-2021 Megvii Inc. All rights reserved.

import cv2
import megengine.functional as F
import numpy as np

__all__ = [
    "preprocess",
    "postprocess",
]


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

def cpu_nms(boxes, scores, iou_threshold=0.3):
    if 0 == len(boxes):
        return []
    rects = list(boxes)
    for i in range(len(rects)):
        rects[i] = list(rects[i])
        rects[i].append(scores[i])
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

def preprocess(image, input_size, mean, std, swap=(2, 0, 1)):
    # print(f'{image.shape}')
    if len(image.shape) == 3:
        padded_img = np.ones((input_size[0], input_size[1], 3)) * 114.0
    else:
        padded_img = np.ones(input_size) * 114.0
    img = np.array(image)
    r = min(input_size[0] / img.shape[0], input_size[1] / img.shape[1])
    resized_img = cv2.resize(
        img,
        (int(img.shape[1] * r), int(img.shape[0] * r)),
        interpolation=cv2.INTER_LINEAR,
    ).astype(np.float32)
    padded_img[: int(img.shape[0] * r), : int(img.shape[1] * r)] = resized_img
    image = padded_img

    image = image.astype(np.float32)
    image = image[:, :, ::-1]
    image /= 255.0
    if mean is not None:
        image -= mean
    if std is not None:
        image /= std
    image = image.transpose(swap)
    image = np.ascontiguousarray(image, dtype=np.float32)
    return image, r

def argmax_keepdims(x, axis):
    output_shape = list(x.shape)
    output_shape[axis] = 1
    return np.argmax(x, axis=axis).reshape(output_shape)

def postprocess(prediction, num_classes, conf_thre=0.7, nms_thre=0.45):
    box_corner = F.zeros_like(prediction)
    box_corner[:, :, 0] = prediction[:, :, 0] - prediction[:, :, 2] / 2
    box_corner[:, :, 1] = prediction[:, :, 1] - prediction[:, :, 3] / 2
    box_corner[:, :, 2] = prediction[:, :, 0] + prediction[:, :, 2] / 2
    box_corner[:, :, 3] = prediction[:, :, 1] + prediction[:, :, 3] / 2
    prediction[:, :, :4] = box_corner[:, :, :4]

    output = [None for _ in range(len(prediction))]
    for i, image_pred in enumerate(prediction):

        # If none are remaining => process next image
        if not image_pred.shape[0]:
            continue
        # Get score and class with highest confidence
        class_conf = F.max(image_pred[:, 5 : 5 + num_classes], 1, keepdims=True)
        class_pred = F.argmax(image_pred[:, 5 : 5 + num_classes], 1, keepdims=True)

        class_conf_squeeze = F.squeeze(class_conf)
        conf_mask = image_pred[:, 4] * class_conf_squeeze >= conf_thre
        detections = F.concat((image_pred[:, :5], class_conf, class_pred), 1)
        detections = detections[conf_mask]
        if not detections.shape[0]:
            continue

        nms_out_index = F.vision.nms(
            detections[:, :4], detections[:, 4] * detections[:, 5], nms_thre,
        )
        detections = detections[nms_out_index]
        if output[i] is None:
            output[i] = detections
        else:
            output[i] = F.concat((output[i], detections))

    if output[0] is not None:
        return output[0].numpy()
    return None
