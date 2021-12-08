#!/usr/bin/env python
# coding=utf-8

from megflow import register, Envelope
@register(inputs=["inp"], outputs=["out"])
class Transport:
    def __init__(self, name, args):
        pass

    def exec(self):
        envelope = self.inp.recv()
        if envelope is not None:
            self.out.send(envelope)

