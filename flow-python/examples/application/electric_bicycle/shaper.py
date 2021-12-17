# MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
#
# Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

#!/usr/bin/env python
# coding=utf-8

from megflow import register


@register(inputs=['inp'], outputs=['out'])
class Shaper:
    def __init__(self, name, args):
        self.name = name
        self._mode = args['mode']
        self._map = dict()

        self.idx = 0

    def expand(self, box, max_w, max_h, ratio):
        l = box[0]
        r = box[2]
        t = box[1]
        b = box[3]
        center_x = (l + r) / 2
        center_y = (t + b) / 2
        w_side = (r - l) * ratio / 2
        h_side = (b - t) * ratio / 2

        l = center_x - w_side
        r = center_x + w_side
        t = center_y - h_side
        b = center_y + h_side
        l = max(0, l)
        t = max(0, t)
        r = min(max_w, r)
        b = min(max_h, b)
        return int(l), int(t), int(r), int(b)

    def exec(self):
        envelope = self.inp.recv()
        if envelope is None:
            self._map.clear()
            return

        msg = envelope.msg

        # push the first
        msg['shaper'] = []
        for track in msg['tracks']:
            tid = track['tid']
            box = track['bbox']
            if tid not in self._map:
                self._map[tid] = dict()

                data = msg['data']
                l, t, r, b = self.expand(box, data.shape[1], data.shape[0],
                                         1.1)
                crop = data[t:b, l:r]
                msg['shaper'].append(crop)
                # cv2.imwrite(f'shaper_{envelope.partial_id}.jpg', crop)

        self.out.send(envelope)
