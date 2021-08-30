# MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
#
# Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

#!/usr/bin/env python
# coding=utf-8
from pyflow import register
from multiprocessing import Process, Pipe

def repeat(n, s, r):
    while True:
        envelope = r.recv()
        if envelope is None:
            break
        for i in range(n):
            msg = {}
            msg['message'] = "a message[{}] repeat {} by process node".format(envelope.msg['message'], i)
            s.send(envelope.repack(msg))


@register(inputs=['inp'], outputs=['out'], exclusive=True)
class RepeatProcess:
    def __init__(self, name, args):
        self.name = name
        s1, r1 = Pipe()
        s2, r2 = Pipe()
        self.send = s1
        self.recv = r2

        self.p = Process(target=repeat, args=(10, s2, r1))
        self.p.start()


    def __del__(self):
        self.send.send(None)
        self.p.join()


    def exec(self):
        envelope = self.inp.recv()

        if envelope is None:
            return

        try:
            self.send.send(envelope)
        except:
            pass

        for i in range(10):
            envelope = self.recv.recv()
            self.out.send(envelope)

