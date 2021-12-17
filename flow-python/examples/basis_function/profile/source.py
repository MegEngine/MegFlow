#!/usr/bin/env python
# coding=utf-8
from megflow import register, Envelope
import time

@register(outputs=['out'])
class Source:
    def __init__(self, name, args):
        self.n = args['n']

    def exec(self):
        start = time.perf_counter()
        for i in range(self.n - 1):
            envelope = Envelope.pack({})
            self.out.send(envelope)
        
        envelope = Envelope.pack({'time': start})
        self.out.send(envelope)
