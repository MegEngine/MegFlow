#!/usr/bin/env python
# coding=utf-8

from megflow import res_register


@res_register()
class Buffer:
    def __init__(self, name, args):
        self.name = name
        self.n = args['n']
        self.i = 0

    def get(self):
        self.i += 1
        return self.i % self.n
