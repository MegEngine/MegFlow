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
import glob
import re

from setuptools import setup, Extension
from setuptools.command.build_ext import build_ext
from setuptools.command.build_py import build_py
from distutils.file_util import copy_file
from distutils.dir_util import copy_tree, mkpath, remove_tree
import subprocess as sp

import platform
system = platform.system().lower()

dyn_ext = 'so'
if system == 'darwin':
    dyn_ext = 'dylib'
elif system == 'windows':
    dyn_ext = 'dll'

debug = os.environ.get("DEBUG")
target_dir = os.environ.get("CARGO_TARGET_DIR")

if not debug:
    debug = False
if not target_dir:
    target_dir = "../target"


class FirstBuildExt(build_py):
    def run(self):
        self.run_command("build_ext")
        return super().run()


class CargoExtension(Extension):
    def __init__(self,
                 target,
                 src,
                 dst,
                features=[]):
        Extension.__init__(self, target, sources=[])
        self.target = target
        self.src = src
        self.dst = dst
        self.features = features

    def build(self):
        command = ['cargo', 'build', '-p', self.target]
        if len(self.features) != 0:
            command.append('--features')
            command.append(' '.join(self.features))
        if not debug:
            command.append('--release')
        sp.check_call(command)

    def install(self, prefix):
        copy_file('{}/{}'.format(prefix, self.src), 'megflow/{}'.format(self.dst))


class CopyExtension(Extension):
    def __init__(self, pattern, src, dst):
        Extension.__init__(self, '', sources=[])
        self.src = src
        self.dst = dst
        self.pattern = pattern

    def copy(self):
        mkpath('megflow/{}'.format(self.dst))
        paths = glob.glob(self.src)
        paths = [ x for x in paths if self.pattern.fullmatch(x) ]

        for path in paths:
            copy_file(path, 'megflow/{}'.format(self.dst))


class ExtBuild(build_ext):
    def run(self):
        current_dir = os.getcwd()
        repo = os.path.dirname(current_dir)
        
        prefix = target_dir
        if debug:
            prefix += '/debug'
        else:
            prefix += '/release'

        for ext in self.extensions:
            if isinstance(ext, CargoExtension):
                ext.build()
                ext.install(prefix)
            if isinstance(ext, CopyExtension):
                ext.copy()


if __name__ == '__main__':
    ext_modules=[
        CargoExtension("flow-python", f"libflow_python.{dyn_ext}", f"megflow.{dyn_ext}", features=["extension-module"]), 
        CargoExtension("flow-quickstart", "megflow_quickstart", "megflow_quickstart_inner"),
    ]

    ffmpeg_dir = os.getenv('FFMPEG_DIR')
    prebuild = os.getenv('CARGO_FEATURE_DYNAMIC')
    if prebuild is not None and ffmpeg_dir is not None:
        pattern = re.compile(f'.*?{dyn_ext}\.[0-9]*')
        ext_modules.append(CopyExtension(pattern, f"{ffmpeg_dir}/lib/*.{dyn_ext}.*", "lib/"))

    current_dir = os.getcwd()
    with open(current_dir+'/Cargo.toml') as f:
        pattern = re.compile(r'\d+\.(?:\d+\.)*\d+')
        for line in f:
            if line.startswith('version'):
                version = re.search(pattern, line).group()
                break

    setup(
        options={
            'bdist_wheel': {
                'py_limited_api': "cp36",
            }
        },
        name="megflow",
        version=version,
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
            'Programming Language :: Python :: 3',
            'Topic :: Software Development :: Libraries :: Application Frameworks',
            'Topic :: Scientific/Engineering',
            'Topic :: Scientific/Engineering :: Mathematics',
            'Topic :: Scientific/Engineering :: Artificial Intelligence',
            'Topic :: Software Development',
            'Topic :: Software Development :: Libraries',
            'Topic :: Software Development :: Libraries :: Python Modules',
        ],
        ext_modules=ext_modules,
        package_data={"": [f'megflow.{dyn_ext}', 'lib/*', 'megflow_quickstart_inner']},
        entry_points={
            'console_scripts':['megflow_run=megflow.command_line:megflow_run', 'run_with_plugins=megflow.command_line:run_with_plugins', 'megflow_quickstart=megflow.command_line:megflow_quickstart'],
        },
        cmdclass={
            'build_ext': ExtBuild,
            'build_py': FirstBuildExt
        },
        zip_safe=False,
    )
