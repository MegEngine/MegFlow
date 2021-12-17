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
from megflow import register
from warehouse.quality_naive import Quality


@register(inputs=['inp'], outputs=['out', 'visualize'])
class ShaperVisualize:
    def __init__(self, name, args):
        self.name = name
        self._mode = args['mode']
        self._map = dict()

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
            logger.info('stream shaper finish')
            # last frame, throw out all cropped image
            for id in self._map:
                self.out.send(self._map[id])
            self._map.clear()
            return

        msg = envelope.msg

        # update the BEST
        if 'tracks' in msg:
            for track in msg['tracks']:
                tid = track['tid']
                box = track['bbox']
                if tid not in self._map:
                    self._map[tid] = envelope.repack(msg)

                data = msg['data']
                l, t, r, b = self.expand(box, data.shape[1], data.shape[0],
                                         1.1)
                crop = data[t:b, l:r]
                assert crop is not None
                quality = Quality.area(crop)

                if self._mode == 'BEST':
                    tid_msg = self._map[tid].msg
                    # save best image
                    if 'quality' not in tid_msg:
                        tid_msg['quality'] = -1

                    old_quality = tid_msg['quality']
                    if quality > old_quality:
                        tid_msg['quality'] = quality
                        tid_msg['crop'] = crop

            ids = msg['failed_ids']
            if len(ids) > 0:
                logger.debug(f'shaper recv failed_ids {ids}')

                for id in ids:
                    if id in self._map:
                        msg['crop'] = self._map[id].msg['crop']
                        self.out.send(self._map[id])
                        self._map.pop(id)

        self.visualize.send(envelope)
