# MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
#
# Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

#!/usr/bin/env python
# coding=utf-8

import subprocess
import time

import requests
import numpy as np
import cv2
from loguru import logger
from megflow import register


class BrightnessMemo:
    def __init__(self):
        self.memo = dict()


    def query_by_tid(self, tid):
        now = time.time()
        if tid in self.memo:
            diff = (now - self.memo[tid])
            return int(255 * min(1.0, diff))
        else:
            self.memo[tid] = now
            return int(0)


class Interpolator:
    def __init__(self):
        self.data_queue = []
        self.brightness = BrightnessMemo()

    def push(self, id, msg):
        item = dict()
        item['id'] = id
        item['data'] = msg['data']
        if 'tracks' in msg:
            item['tracks'] = msg['tracks']
        if 'crop' in msg:
            item['crop'] = msg['crop']
        self.data_queue.append(item)

    def interpolate(self, begin, end, step):
        ret = [[] for _ in range(step)]

        end_map = dict()
        # convert end tracks to hashmap
        for item in end["tracks"]:
            end_map[item['tid']] = item['bbox']

        for item in begin['tracks']:
            tid = item['tid']
            if tid in end_map:
                a = item['bbox']
                b = end_map[tid]
                for j in range(step):
                    new_l = a[0] + (b[0] - a[0]) / step * j
                    new_t = a[1] + (b[1] - a[1]) / step * j
                    new_r = a[2] + (b[2] - a[2]) / step * j
                    new_b = a[3] + (b[3] - a[3]) / step * j
                    ret[j].append(
                        np.array((new_l, new_t, new_r, new_b, tid),
                                 dtype=np.float32))
            else:
                bbox = item['bbox']
                ret[0].append(
                    np.array((bbox[0], bbox[1], bbox[2], bbox[3], tid),
                             dtype=np.float32))
        return ret

    def pop(self):
        ret = []
        end_idx = -1
        for i, item in enumerate(self.data_queue):
            if i == 0:
                continue
            if 'tracks' in item:
                end_idx = i

        if end_idx != -1:
            bboxes_list = self.interpolate(self.data_queue[0],
                                           self.data_queue[end_idx], end_idx)
            assert len(bboxes_list) == end_idx

            # prepare crop
            crop = None
            if 'crop' in self.data_queue[end_idx]:
                crop = self.data_queue[end_idx]['crop']
                crop = cv2.resize(crop, (128, 128))

            # real draw
            for i in range(end_idx):
                data = self.data_queue[i]['data']
                bboxes = bboxes_list[i]
                for _, bbox in enumerate(bboxes):
                    print(bbox)
                    brightness = self.brightness.query_by_tid(bbox[4])
                    # cv2.putText(data, str(bbox[4]), (int(bbox[0]), int(bbox[1])), cv2.FONT_HERSHEY_SIMPLEX, 2.0, (255,255,255))
                    cv2.rectangle(data, (int(bbox[0]), int(bbox[1])),
                                  (int(bbox[2]), int(bbox[3])),
                                  (brightness, brightness, brightness), 3)

                if crop is not None:
                    data[0:128, 0:128, :] = crop
                ret.append(data)

            for i in range(end_idx):
                self.data_queue.pop(0)
        return ret


@register(inputs=['inp'], outputs=['out'])
class Visualize:
    def __init__(self, _, args):
        logger.info("connect livego manager...")

        manage_url = args["livego_manger_url"]
        api_result = requests.get(manage_url)
        if not api_result.ok:
            raise Exception(f'request channel failed, from {manage_url}')
        channel = api_result.json()['data']
        self.upload_url = args['livego_upload_url_template'].format(channel)
        logger.info(f"manage url {manage_url} , upload url {self.upload_url}")

        self.interpolator = Interpolator()
        self.conn = None

    def __del__(self):
        if self.conn is not None:
            self.conn.terminate()
            self.conn = None

    def exec(self):
        envelope = self.inp.recv()
        if envelope is None:
            return
        data = envelope.msg['data']

        # build connection
        if self.conn is None:
            (frame_height, frame_width, _) = data.shape
            command = [
                'ffmpeg', '-y', '-f', 'rawvideo', '-vcodec', 'rawvideo',
                '-pix_fmt', 'bgr24', '-s',
                "{}x{}".format(frame_width, frame_height), '-r',
                str(25), '-i', '-', '-c:v', 'libx264', '-pix_fmt', 'yuv420p',
                '-preset', 'ultrafast', '-f', 'flv', self.upload_url
            ]
            self.conn = subprocess.Popen(command, stdin=subprocess.PIPE)

        self.interpolator.push(envelope.partial_id, envelope.msg)
        outputs = self.interpolator.pop()
        for drawed_image in outputs:
            # try write visualized image
            try:
                self.conn.stdin.write(drawed_image.tobytes())
            except IOError as e:
                logger.error(f"write failed: {e}")
                self.conn.terminate()
                self.conn = None
