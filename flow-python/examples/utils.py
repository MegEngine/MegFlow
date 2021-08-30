# MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
#
# Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

#!/usr/bin/env python
# coding=utf-8
import numpy as np


def sigmoid(x: float) -> float:
    return 1.0 / 1.0 + np.exp(-x)


def iou(r0: np.array, r1: np.array) -> float:
    xx1 = max(r0[1], r1[1])
    yy1 = max(r0[2], r1[2])
    xx2 = min(r0[3], r1[3])
    yy2 = min(r0[4], r1[4])
    dx = max(0., xx2 - xx1)
    dy = max(0, yy2 - yy1)
    i = dx * dy
    u = (r0[3] - r0[1]) * (r0[4] - r0[2]) + (r1[3] - r1[1]) * (r1[4] -
                                                               r1[2]) - i
    iou = i / u
    return iou


def is_overlap(rect1: np.array, rect2: np.array, iou_thr: float) -> bool:
    ov = iou(rect1, rect2)
    return ov >= iou_thr

def shrink_rect(rect):
    x1, y1, x2, y2 = rect

    # shrink person to center
    '''
    x1_new = (x2 - x1) / 4 + x1
    x2_new = x2 - (x2 - x1) / 4
    y1_new = (y2 - y1) / 4 + y1
    y2_new = y2 - (y2 - y1) / 4
    '''

    # shrink person to head by stat
    w = x2 - x1 + 1
    h = y2 - y1 + 1

    head_ratio_h2w = 1.3

    min_head_width = 40  # set min_head if person is too small
    min_head_height = min_head_width * head_ratio_h2w

    ratio_w = 0.3478  # 0.3478 is the width of head relative to person
    ratio_h = 0.1709  # 0.1709 is the height of head relative to person

    w_head = max(w * ratio_w, min_head_width)
    h_head = max(h * ratio_h, min_head_height)

    x1_new = x1 + (
        w -
        w_head) * 0.5  # put the center of head on x at the center of person
    y1_new = y1 + h * 0.0931 - h_head * 0.5  # 0.0931 if the y_center of head relative to person
    x2_new = x1_new + w_head - 1
    y2_new = y1_new + h_head - 1

    return x1_new, y1_new, x2_new, y2_new


def filter_rect_by_score_and_size(scores, rects, idx, size):
    if scores[idx] == -1:
        return np.array([])
    elif rects[idx][2] - rects[idx][0] + 1 <= size or rects[idx][3] - rects[
            idx][1] + 1 <= size:
        return np.array([])
    else:
        return np.concatenate((rects[idx], scores[idx:idx + 1]))


def is_overlap_v2(rect1, rect2, iou_threshold):
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
    return rect1[4] * rect2[4] < ov or ov >= iou_threshold


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


def IoB(rect1, rect2, iob_threshold):
    b = (rect2[:, 2] - rect2[:, 0] + 1) * (rect2[:, 3] - rect2[:, 1] + 1)
    # from IPython import embed; embed() $TODO
    xx1 = np.maximum(rect1[:, 0], rect2[:, 0])
    yy1 = np.maximum(rect1[:, 1], rect2[:, 1])
    xx2 = np.minimum(rect1[:, 2], rect2[:, 2])
    yy2 = np.minimum(rect1[:, 3], rect2[:, 3])
    dx = np.maximum(0, xx2 - xx1 + 1)
    dy = np.maximum(0, yy2 - yy1 + 1)
    i = dx * dy
    ov = i / b
    ov[b < 0] = 0
    return ov


def IoU(rect1, rect2, iob_threshold):
    a1 = (rect1[:, 2] - rect1[:, 0] + 1) * (rect1[:, 3] - rect1[:, 1] + 1)
    a2 = (rect2[:, 2] - rect2[:, 0] + 1) * (rect2[:, 3] - rect2[:, 1] + 1)
    xx1 = np.maximum(rect1[:, 0], rect2[:, 0])
    yy1 = np.maximum(rect1[:, 1], rect2[:, 1])
    xx2 = np.minimum(rect1[:, 2], rect2[:, 2])
    yy2 = np.minimum(rect1[:, 3], rect2[:, 3])
    dx = np.maximum(0, xx2 - xx1 + 1)
    dy = np.maximum(0, yy2 - yy1 + 1)
    i = dx * dy

    ov = i / (a1 + a2 - i)
    ov[a1 + a2 - i < 0] = 0
    return ov


def simple_merge(rect1, rect2):
    if rect2[4] > rect1[4]:
        return rect2
    else:
        return rect1


def raw_nms(boxes, scores, iou_threshold=0.3):
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


def nms(boxes, scores, iou_threshold=0.3):
    if 0 == len(boxes):
        return []
    rects = list(boxes)
    for i in range(len(rects)):
        rects[i] = list(rects[i])
        rects[i].append(scores[i])
        rects[i].append(i)
    rects.sort(key=lambda x: x[1])
    idx = 0
    for i in range(len(rects)):
        if is_overlap_v2(rects[i], rects[idx], iou_threshold):
            rects[idx] = simple_merge(rects[idx], rects[i])
        else:
            idx += 1
            if idx != i:
                rects[idx] = rects[i]
    rects = rects[:idx + 1]
    rects.sort(key=lambda x: x[0])
    idx = 0
    for i in range(len(rects)):
        if is_overlap_v2(rects[i], rects[idx], iou_threshold):
            rects[idx] = simple_merge(rects[idx], rects[i])
        else:
            idx += 1
            if idx != i:
                rects[idx] = rects[i]
    rects = rects[:idx + 1]
    idx = 0
    while idx < len(rects):
        left = idx + 1
        right = len(rects) - 1
        while left <= right:
            if is_overlap_v2(rects[idx], rects[left], iou_threshold):
                rects[idx] = simple_merge(rects[idx], rects[left])
                rects[left] = rects[right]
                right -= 1
            else:
                left += 1
        if len(rects) != right + 1:
            rects = rects[:right + 1]
        idx += 1
    return [x[5] for x in rects]