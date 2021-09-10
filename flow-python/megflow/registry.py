# MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
#
# Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

#!/usr/bin/env python
# coding=utf-8
import inspect
import ast
import types
import os
from collections import Iterable

def __register(name, inputs, outputs, exclusive, func):
    params = {}

    params['name'] = name
    params['code'] = func
    params['inputs'] = inputs
    params['outputs'] = outputs
    params['exclusive'] = exclusive

    return params


def __res_register(name, func):
    params = {}
    params['name'] = name
    params['code'] = func

    return params


__NODES_PLUGINS = 'nodes'
__RESOURCE_PLUGINS = 'resources'
__PLUGINS_DEFAULT = {
    __NODES_PLUGINS: [],
    __RESOURCE_PLUGINS: [],
}
_PLUGINS_REGISTRY = __PLUGINS_DEFAULT.copy()

def register(name=None, inputs=[], outputs=[], exclusive=False):
    def decorator(func):
        nonlocal name
        global _PLUGINS_REGISTRY
        if name is None:
            name = func.__name__
        _PLUGINS_REGISTRY[__NODES_PLUGINS].append(__register(name, inputs, outputs, exclusive, func))
        return func

    return decorator


def res_register(name=None):
    def decorator(func):
        nonlocal name
        global _PLUGINS_REGISTRY
        if name is None:
            name = func.__name__
        _PLUGINS_REGISTRY[__RESOURCE_PLUGINS].append(__res_register(name, func))
        return func

    return decorator


def collect():
    global _PLUGINS_REGISTRY
    plugins = _PLUGINS_REGISTRY.copy()
    _PLUGINS_REGISTRY = __PLUGINS_DEFAULT.copy()
    return plugins
