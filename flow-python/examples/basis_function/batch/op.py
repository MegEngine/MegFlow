#!/usr/bin/env python
# coding=utf-8

from megflow.func_op import *

@batch_def()
def group(inp, context=None):
    for x in inp:
        x['group_id'] = context.id
    context.id += 1

@source_def()
def gen(context=None):
    for i in range(context.n):
        yield { 'id': i }

@sink_def()
def printer(inp):
    print(inp)
