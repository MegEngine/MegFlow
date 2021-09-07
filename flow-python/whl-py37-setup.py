# -*- coding: utf-8 -*-
# MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
#
# Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

#!/usr/bin/env python
# coding=utf-8
from setuptools import setup, find_packages

setup(
    name="pyflow",
    version="0.1.0",
    packages=["pyflow"],
    author="Megvii IPU-SDK Team",
    author_email="megengine@megvii.com",
    url="https://github.com/MegEngine/MegFlow",
    include_package_data=True,
    classifiers=[
        'Development Status :: 3 - Alpha',
        'Intended Audience :: Developers',
        'License :: OSI Approved :: Apache Software License',
        'Natural Language :: English',
        'Operating System :: POSIX :: Linux',
        'Programming Language :: C++',
        'Programming Language :: Rust',
        'Programming Language :: Python :: 3.7',
        'Topic :: Software Development :: Libraries :: Application Frameworks',
        'Topic :: Scientific/Engineering',
        'Topic :: Scientific/Engineering :: Mathematics',
        'Topic :: Scientific/Engineering :: Artificial Intelligence',
        'Topic :: Software Development',
        'Topic :: Software Development :: Libraries',
        'Topic :: Software Development :: Libraries :: Python Modules',
    ],
    python_requires='>=3.7,<3.8',
    package_data={
        "":['run_with_plugins']
    } 
)

