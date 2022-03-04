#!/usr/bin/env python
# coding=utf-8

from megflow.func_op import *
from megflow import register, Envelope

@register(outputs=['out'])
class Source:
    def __init__(self, name, args):
        self.n = args['n']

    def exec(self):
        for i in range(self.n):
            envelope = Envelope.pack(i)
            envelope.to_addr = i % 2
            self.out.send(envelope)

@sink_def()
def sink(inp, context=None):
    print(context.name, 'get', inp)

