#!/usr/bin/env python
# coding=utf-8

from megflow import register, Envelope
import time

@register(inputs=['inp'])
class Sink:
    def __init__(self, name, args):
        self.n = args['n']
        self.count = 0

    def exec(self):
        envelope = self.inp.recv()

        if envelope is not None:
            self.count += 1
            if self.count == self.n:
                start = envelope.msg['time']
                end = time.perf_counter()
                elapsed = end - start
                print('Time elapsed is {} ms for {} messages'.format(elapsed*1000, self.n))

