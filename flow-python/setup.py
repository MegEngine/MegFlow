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
import sys
import os

from setuptools import setup

devel_version = os.environ.get("DEVEL_VERSION")
if not devel_version:
    devel_version = "0.1.0"  # fall back

py_version = sys.version_info

if __name__ == '__main__':
    setup(
        options={
            'bdist_wheel': {
                'python_tag': "py{}.{}".format(py_version.major,
                                              py_version.minor),
            }
        },
        name="megflow",
        version=devel_version,
        packages=["megflow"],
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
            'Programming Language :: Rust',
            'Programming Language :: Python :: {}.{}'.format(
                py_version.major, py_version.minor),
            'Topic :: Software Development :: Libraries :: Application Frameworks',
            'Topic :: Scientific/Engineering',
            'Topic :: Scientific/Engineering :: Mathematics',
            'Topic :: Scientific/Engineering :: Artificial Intelligence',
            'Topic :: Software Development',
            'Topic :: Software Development :: Libraries',
            'Topic :: Software Development :: Libraries :: Python Modules',
        ],
        python_requires='>={}.{},<{}.{}'.format(py_version.major,
                                                py_version.minor,
                                                py_version.major,
                                                py_version.minor + 1),
        package_data={"": ['run_with_plugins_inner']},
        entry_points={
            'console_scripts':['run_with_plugins=megflow.command_line:main'],
        },
    )
