#!/usr/bin/env python
# coding=utf-8

from megflow.func_op import *
import random

@map_def()
def multiply(inp, context=None):
    return inp * context.c


@reduce_def()
def summation(inp):
    return sum(inp, 0)

@source_def()
def rnd(context=None):
    for i in range(context.n):
        yield random.random()

@sink_def()
def printer(inp):
    print('printer: ', inp)
