#!/usr/bin/env python
# coding=utf-8
from megflow.func_op import *

@map_def(exclusive=True)
def noop(x):
    # print('send', x)
    return x


@source_def()
def gen(context=None):
    for i in range(context.n):
        # print('first', i)
        yield i


@sink_def()
def printer(x):
    pass
    # print('last', x)
