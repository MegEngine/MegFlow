# MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
#
# Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

#!/usr/bin/env python
# coding=utf-8
from pyflow import register, Envelope, sleep

@register(outputs=['out'])
class Source:
    def __init__(self, name, args):
        self.n = args['n']

    def exec(self):
        for i in range(self.n):
            msg = {}
            msg['message'] = 'send to {}'.format(i)
            envelope = Envelope.pack(msg)
            envelope.to_addr = i
            self.out.send(envelope)

        sleep(5)

        for i in range(self.n):
            envelope = Envelope.pack(None)
            envelope.to_addr = i
            self.out.send(envelope)

