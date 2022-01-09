#!/usr/bin/env python
# coding=utf-8

from megflow.func_op import *
from megflow import create_future, join
import random


@sink_def()
def rnd(inp):
    inp.wake(random.random())


@source_def()
def printer():
    (fut1, waker1) = create_future()
    yield waker1
    (fut2, waker2) = create_future()
    yield waker2

    [r1, r2] = join([lambda: fut1.wait(), lambda: fut2.wait()])
    print('printer: ', r1, r2)

