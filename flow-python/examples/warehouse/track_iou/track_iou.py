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
import numpy as np
from scipy.optimize import linear_sum_assignment, leastsq

class CostGenerator:
    def __init__(self):
        self.linear_pred_funcs = []
        for _ in range(4):
            self.linear_pred_funcs.append(LinearFunc())

    def update(self, x, boxes):
        y = np.array([bb['bbox'] for bb in boxes])
        for i in range(4):
            self.linear_pred_funcs[i].update(x, y[:, i])

    def predict(self, x):
        linear_pred = np.zeros(4)
        for i in range(4):
            linear_pred[i] = self.linear_pred_funcs[i].predict(x)
        return linear_pred

    def get_cost(self, x, det_box):
        pred_box = self.predict(x)
        return 1 - get_iou(pred_box, det_box['bbox'])


class Tracker:
    '''
    Args:
        detections (list): list of detections per frame
        sigma_h (float): high detection threshold.
        sigma_iou (float): IOU threshold.
        t_min (float): minimum track length in frames.
        m_tolerance (float):
           * If the confidence of a track is below a threshold, the tracker will not be
           * deleted at once, but hidden temporarily in the internal state of the
           * tracker. Only after N consecutive frames in which the confidence scores are
           * all below the threshold, will the track be deleted. Otherwise, the track
           * will be resumed.
    '''
    def __init__(self):
        self.sigma_h = 0.05
        self.threshold = 0.8
        self.t_min_item = 1
        self.miss_tolerance = 4
        self.max_history_len = 6
        self.smooth_factor = 0.3
        self.frame_count = 0
        self.tracker_count = 0
        '''
        tracker:
            keys = [boxes, frame_ids, cost_generator,
                    max_score, num_miss, num_valid]
            boxes: list of boxes, in shape of [x1, y1, x2, y2, score]
        '''
        self.trackers = dict()

    def set_args(self, **kwargs):
        if "sigma_h" in kwargs:
            self.sigma_h = kwargs["sigma_h"]
        if "sigma_iou" in kwargs:
            self.threshold = 1 - kwargs["sigma_iou"]
        if "t_min_item" in kwargs:
            self.t_min_item = kwargs["t_min_item"]
        if "miss_tolerance" in kwargs:
            self.miss_tolerance = kwargs["miss_tolerance"]
        if "max_history_len" in kwargs:
            self.max_history_len = kwargs["max_history_len"]
        if "smooth_factor" in kwargs:
            self.smooth_factor = kwargs["smooth_factor"]

        self.frame_count = 0
        self.tracker_count = 0
        self.trackers = dict()

    def _get_next_track_id(self):
        self.tracker_count += 1
        return self.tracker_count

    def _get_results(self, use_smooth=True):
        ret = []
        # logger.debug(f'track.internal: {self.trackers}')
        for tracker_id, tracker in self.trackers.items():
            if tracker['num_valid'] < self.t_min_item or \
                tracker['max_score'] < self.sigma_h or \
                tracker['num_miss'] > 0:
                continue

            if use_smooth and \
                len(tracker['frame_ids']) > 1 and \
                (tracker['frame_ids'][-1] - tracker['frame_ids'][-2] == 1):
                boxes = np.array([bb['bbox'] for bb in tracker['boxes']])
                bbox = smooth_boxes(boxes, self.smooth_factor)
            else:
                bbox = tracker['boxes'][-1]['bbox']

            ret.append(
                dict(tid=tracker_id,
                     bbox=bbox))

        return ret

    def _update_trackers(self, det_boxes):
        failed_ids = []
        self.frame_count += 1

        updated_tracker_ids = set()
        matched_det_ids = set()
        # Step 1: match tracker and det_boxes
        if len(self.trackers) > 0:
            # Step 1.1: generate cost matrix
            list_tracker_id = list(self.trackers.keys())
            mat_cost = np.zeros([len(list_tracker_id), len(det_boxes)])
            for i, tracker_id in enumerate(list_tracker_id):
                tracker = self.trackers[tracker_id]
                min_frame_id = tracker['frame_ids'][0]
                for j, det_box in enumerate(det_boxes):
                    mat_cost[i, j] = tracker['cost_generator'].get_cost(
                        self.frame_count - min_frame_id, det_box)
            # Step 1.2: match cost with linear_sum_assignment
            matched_row_idxes, matched_col_idxes = linear_sum_assignment(
                mat_cost)

            for r, c in zip(matched_row_idxes, matched_col_idxes):
                tracker_id = list_tracker_id[r]
                matched_det_box = det_boxes[c]

                # Step 2: update matched trackers
                if mat_cost[r, c] < self.threshold:
                    self.trackers[tracker_id]['boxes'].append(matched_det_box)
                    if len(self.trackers[tracker_id]
                           ['boxes']) > self.max_history_len:
                        self.trackers[tracker_id]['boxes'] = \
                            self.trackers[tracker_id]['boxes'][-self.max_history_len:]
                    self.trackers[tracker_id]['frame_ids'].append(
                        self.frame_count)
                    if len(self.trackers[tracker_id]
                           ['frame_ids']) > self.max_history_len:
                        self.trackers[tracker_id]['frame_ids'] = \
                            self.trackers[tracker_id]['frame_ids'][-self.max_history_len:]

                    min_frame_id = self.trackers[tracker_id]['frame_ids'][0]
                    self.trackers[tracker_id]['cost_generator'].update(
                        np.array(self.trackers[tracker_id]['frame_ids']) -
                        min_frame_id,
                        np.array(self.trackers[tracker_id]['boxes']))

                    self.trackers[tracker_id]['num_miss'] = 0
                    self.trackers[tracker_id]['max_score'] = \
                        max(self.trackers[tracker_id]['max_score'],
                            matched_det_box['score'])
                    self.trackers[tracker_id]['num_valid'] += 1
                    self.trackers[tracker_id]['num_item'] += 1

                    updated_tracker_ids.add(tracker_id)
                    matched_det_ids.add(c)

            # Step 2: update failed trackers
            for tracker_id in list_tracker_id:
                if tracker_id in updated_tracker_ids:
                    continue
                self.trackers[tracker_id]['num_miss'] += 1

                if self.trackers[tracker_id]['num_miss'] > self.miss_tolerance:
                    self.trackers.pop(tracker_id)
                    failed_ids.append(tracker_id)

        # Step 3: start new tracker
        for i, det_box in enumerate(det_boxes):
            if i in matched_det_ids:
                continue
            tracker_id = self._get_next_track_id()
            self.trackers[tracker_id] = dict()
            self.trackers[tracker_id]['boxes'] = []
            self.trackers[tracker_id]['boxes'].append(det_box)
            self.trackers[tracker_id]['frame_ids'] = []
            self.trackers[tracker_id]['frame_ids'].append(self.frame_count)
            self.trackers[tracker_id]['max_score'] = det_box['score']
            self.trackers[tracker_id]['num_miss'] = 0
            self.trackers[tracker_id]['num_valid'] = 1
            self.trackers[tracker_id]['num_item'] = 1
            self.trackers[tracker_id]['cost_generator'] = CostGenerator()
            self.trackers[tracker_id]['cost_generator'].update(
                np.array([0]), [
                    det_box,
                ])
        
        # return finished track id
        return failed_ids

    def track(self, det_boxes, use_smooth=True):
        failed_ids = self._update_trackers(det_boxes)
        return self._get_results(use_smooth), failed_ids


def get_iou(bbox1, bbox2):
    """
    Calculates the intersection-over-union of two bounding boxes.
    Args:
        bbox1 (numpy.array, list of floats): bounding box in format x1, y1, x2, y2.
        bbox2 (numpy.array, list of floats): bounding box in format x1, y1, x2, y2.
    Returns:
        int: intersection-over-onion of bbox1, bbox2
    """

    x1_1, y1_1, x1_2, y1_2 = bbox1[:4]
    x2_1, y2_1, x2_2, y2_2 = bbox2[:4]

    # get the overlap rectangle
    overlap_x1 = max(x1_1, x2_1)
    overlap_y1 = max(y1_1, y2_1)
    overlap_x2 = min(x1_2, x2_2)
    overlap_y2 = min(y1_2, y2_2)

    # check if there is an overlap
    if overlap_x2 - overlap_x1 <= 0 or overlap_y2 - overlap_y1 <= 0:
        return 0

    # if yes, calculate the ratio of the overlap to each ROI size and the unified size
    size_1 = (x1_2 - x1_1 + 1) * (y1_2 - y1_1 + 1)
    size_2 = (x2_2 - x2_1 + 1) * (y2_2 - y2_1 + 1)
    size_intersection = (overlap_x2 - overlap_x1 + 1) * (overlap_y2 -
                                                         overlap_y1 + 1)
    size_union = size_1 + size_2 - size_intersection

    return size_intersection / size_union


def smooth_boxes(boxes, smooth_fatcor=0.3):
    if len(boxes) > 1:
        smoothed_box = [
            boxes[-2][0] * smooth_fatcor + boxes[-1][0] * (1 - smooth_fatcor),
            boxes[-2][1] * smooth_fatcor + boxes[-1][1] * (1 - smooth_fatcor),
            boxes[-2][2] * smooth_fatcor + boxes[-1][2] * (1 - smooth_fatcor),
            boxes[-2][3] * smooth_fatcor + boxes[-1][3] * (1 - smooth_fatcor),
            boxes[-1][-1],
        ]
    else:
        smoothed_box = boxes[-1]
    return smoothed_box


def linear_func(p, x):
    k, b = p
    return k * x + b


def linear_error(p, x, y):
    return linear_func(p, x) - y


class LinearFunc:
    def __init__(self):
        self.param = (np.array([0, 0]), 0)

    def update(self, x, y):
        if len(x) == 1:
            self.param = (np.array([0, y[0]]), 0)
        else:
            self.param = leastsq(linear_error, self.param[0], args=(x, y))

    def predict(self, x):
        return linear_func(self.param[0], x)

if __name__ == "__main__":

    def build_test_input1():
        item1 = dict()
        item1["bbox"] = np.array([10, 20,  300, 300])
        item1["score"] = 0.7
        item1["cls"] = int(15)

        item2 = dict()
        item2["bbox"] = np.array([300, 300,  600, 600])
        item2["score"] = 0.8
        item2["cls"] = int(15)

        return [item1, item2]

    def build_test_input2():
        item1 = dict()
        item1["bbox"] = np.array([60, 60,  360, 360])
        item1["score"] = 0.77
        item1["cls"] = int(15)

        item2 = dict()
        item2["bbox"] = np.array([310, 310,  660, 660])
        item2["score"] = 0.88
        item2["cls"] = int(15)

        return [item1, item2]

    def build_test_input3():
        item1 = dict()
        item1["bbox"] = np.array([60, 60,  360, 360])
        item1["score"] = 0.77
        item1["cls"] = int(15)

        return [item1]

    t = Tracker()

    set1 = build_test_input1()
    for i in range(6):
        t.track(set1)

    set6 = build_test_input2()
    print(f'set6 result {t.track(set6)}')

    set7 = build_test_input2()
    print(f'set7 result {t.track(set7)}')

    set8 = build_test_input3()

    #  give failed_ids
    for i in range(9):
        print(f'set8 result {t.track(set8)}')

    # clean all
    for i in range(10):
        print(f'{t.track([])}')