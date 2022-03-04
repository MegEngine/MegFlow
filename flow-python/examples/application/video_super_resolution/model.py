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
import random
import string
import os
import cv2
import numpy as np
from loguru import logger
from megflow import register
from .lite import PredictorLite


@register(inputs=['inp'], outputs=['out'])
class Model:
    def __init__(self, name, arg):
        self.name = name
        # self.inidx = 0

        # load VSR models
        self._model = PredictorLite(path=arg['dir'],
                                    device=arg['device'],
                                    device_id=arg['device_id'])

    def exec(self):
        # recv 25 frames in 4000ms
        (envelopes, _) = self.inp.batch_recv(100, 4000)
        if len(envelopes) == 0:
            return

        imagelist = []
        for env in envelopes:
            data = env.msg['data']
            # cv2.imwrite("{:08d}.png".format(self.inidx), data)
            # self.inidx+=1

            mat = cv2.cvtColor(data, cv2.COLOR_BGR2RGB)
            mat = np.transpose(mat, (2, 0, 1))
            mat = np.expand_dims(mat, axis=0)
            imagelist.append(mat[...])
        batchdata = np.concatenate(imagelist, axis=0)
        envelopes[-1].msg['batch_result'] = self._model.inference(batchdata)

        for env in envelopes:
            self.out.send(env)


@register(inputs=['inp'], outputs=['out'])
class Save:
    def __init__(self, name, arg):
        print("Save init")
        self.name = name
        salt = ''.join(random.sample(string.ascii_letters + string.digits,
                                     6)) + ".flv"
        self.video_path = os.path.join(arg["path"], salt)
        self.conn = None
        # self.outidx = 0

    def __del__(self):
        if self.conn is not None:
            self.conn.terminate()
            self.conn = None
        logger.info("finish one stream")

    def exec(self):
        envelope = self.inp.recv()
        if envelope is None:
            return

        if 'batch_result' in envelope.msg:
            images = envelope.msg['batch_result']
            assert len(images) > 0

            # build connection
            if self.conn is None:
                (frame_height, frame_width, _) = images[0].shape
                command = [
                    'ffmpeg', '-y', '-f', 'rawvideo', '-vcodec', 'rawvideo',
                    '-pix_fmt', 'bgr24', '-s',
                    "{}x{}".format(frame_width, frame_height), '-r',
                    str(20), '-i', '-', '-c:v', 'libx264', '-pix_fmt',
                    'yuv420p', '-profile', 'high', '-level',
                    str(4.0), '-preset', 'slow', '-f', 'flv', self.video_path
                ]
                self.conn = subprocess.Popen(command, stdin=subprocess.PIPE)

            for image in images:
                # write visualized image
                # cv2.imwrite("out_{:08d}.png".format(self.outidx), image)
                # self.outidx += 1
                try:
                    self.conn.stdin.write(image.tobytes())
                except IOError as e:
                    logger.error(f"write failed: {e}")
                    self.conn.terminate()
                    self.conn = None

        if envelope.partial_id == 0:
            self.out.send(envelope.repack(self.video_path))
        else:
            self.out.send(envelope.repack(""))
