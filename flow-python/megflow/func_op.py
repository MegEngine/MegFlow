#!/usr/bin/env python
# coding=utf-8

from .registry import register
import inspect
from functools import partial
import re

class Context:
    def __init__(self, **entries):
        self.__dict__.update(entries)

def name_convert_to_camel(name):
    contents = re.findall('_[a-z]+', name)
    for content in set(contents):
        name = name.replace(content, content[1:].title())
    return name.title()


def _with_context(func):
    sig = inspect.signature(func)
    params = sig.parameters
    return 'context' in params


def _common_def(inputs=[], outputs=[]):
    def common_def(plugin_def):
        def decorator(name=None, exclusive=False):
            def _decorator(func):
                nonlocal name
                if name is None:
                    name = func.__name__
                name = name_convert_to_camel(name)
                with_context = _with_context(func)
    
                @register(name=name, inputs=inputs, outputs=outputs, exclusive=exclusive)
                class Node:
                    def __init__(self, name, args):
                        self.context = Context(**args)
                        if with_context:
                            self.impl = partial(func, context = self.context) 
                        else:
                            self.impl = func
    
                    def exec(self):
                        plugin_def(self, self.impl)
    
                return Node
    
            return _decorator
        return decorator
    return common_def


@_common_def(inputs=["inp"], outputs=["out"])
def map_def(self, func):
    envelope = self.inp.recv()
    if envelope is None:
        return
    envelope.msg = func(envelope.msg)
    self.out.send(envelope)


@_common_def(inputs=["inp:[]"], outputs=["out"])
def reduce_def(self, func):
    ret = []
    for inp in self.inp:
        ret.append(inp.recv())
    
    all_empty = True
    for envelope in ret:
        all_empty = all_empty and envelope is None
    if all_empty:
        return

    for envelope in ret:
        assert envelope is not None

    msgs = [ envelope.msg for envelope in ret ]

    self.out.send(ret[0].repack(func(msgs)))


@_common_def(inputs=["inp"])
def sink_def(self, func):
    envelope = self.inp.recv()

    if envelope is None:
        return

    func(envelope.msg)


@_common_def(outputs=["out"])
def source_def(self, func):
    from megflow import Envelope
    i = 0
    for msg in func():
        self.out.send(Envelope.pack(msg, info={'partial_id':i}))
        i += 1


@_common_def(inputs=["inp"], outputs=["out"])
def batch_def(self, func):
    (envelopes, is_closed) = self.inp.batch_recv(self.context.batch_size, self.context.timeout)
    if is_closed or len(envelopes) == 0:
        return

    func([envelope.msg for envelope in envelopes])

    for envelope in envelopes:
        self.out.send(envelope)
