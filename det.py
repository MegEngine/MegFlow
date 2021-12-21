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
class Detect:
    def __init__(self, name, arg):
        self.name = name
        print('Detect init')

    def exec (self):
        envelope = self.inp.recv()
        if envelope is None:
            return

        data = envelope.msg['data']
        print(data)
        print('Detect')
        self.out.send(envelope)
