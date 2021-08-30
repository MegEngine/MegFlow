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

instance_id = 0

@register('Printer', inputs=['inp'])
class Node:
    def __init__(self, name, args):
        global instance_id
        self.id = instance_id
        instance_id += 1
        
    def exec(self):
        envelope = self.inp.recv()
        if envelope is not None:
            gbuf = self.global_buf.get()
            pbuf = self.parent_buf.get()
            buf = self.buf.get()

            print('Printer[{}] get msg: {}, buf(global, parent, local): ({}, {}, {})'.format(self.id, envelope.msg['message'], gbuf, pbuf, buf))

