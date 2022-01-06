#!/usr/bin/env python
# coding=utf-8

from megflow import register, Envelope
import time

@register(inputs=['inp'])
class Sink:
    def __init__(self, name, args):
        self.data_n = args['data_n']
        self.node_n = args['node_n']
        self.count = 0

    def exec(self):
        envelope = self.inp.recv()

        if envelope is not None:
            self.count += 1
            if self.count == self.data_n:
                start = envelope.msg['time']
                end = time.perf_counter()
                elapsed = end - start
                print('Time elapsed is {} ms for {} messages, {} nodes '.format(elapsed*1000, self.data_n, self.node_n))

