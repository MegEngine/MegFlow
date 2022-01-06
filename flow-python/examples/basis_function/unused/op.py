#!/usr/bin/env python
# coding=utf-8
from megflow.func_op import *

@source_def()
def gen(context=None):
    for i in range(context.n):
        print('gen', i)
        yield { 'id': i }
