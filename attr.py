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
class Attribute:
    def __init__(self, name, arg):
        self.name = name
        print('Attribute init')

    def exec(self):
        envelope = self.inp.recv()
        if envelope is None:
            return

        print("Attribute")
        self.out.send(envelope)
        # self.out.send(envelope.repack("json string"))
