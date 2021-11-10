#!/usr/bin/env python
# coding=utf-8
import os
import sys
import subprocess
import pkg_resources
from find_libpython import find_libpython

def run():
    bin_exist = pkg_resources.resource_exists('megflow', 'megflow_run_inner')
    if not bin_exist:
        print('cannot find megflow_run, exit!')
        sys.exit(-1)
    bin_path = pkg_resources.resource_filename('megflow', 'megflow_run_inner')

    pylib_dir = os.path.dirname(find_libpython())

    sys.argv[0] = bin_path
    ret = subprocess.Popen(sys.argv, env={**os.environ, 'LD_LIBRARY_PATH': pylib_dir + ':' + os.environ['LD_LIBRARY_PATH']})
    ret.wait()

def quickstart():
    bin_exist = pkg_resources.resource_exists('megflow', 'megflow_quickstart_inner')
    if not bin_exist:
        print('cannot find megflow_quickstart, exit!')
        sys.exit(-1)
    bin_path = pkg_resources.resource_filename('megflow', 'megflow_quickstart_inner')

    pylib_dir = os.path.dirname(find_libpython())

    sys.argv[0] = bin_path
    ret = subprocess.Popen(sys.argv, env={**os.environ, 'LD_LIBRARY_PATH': pylib_dir + ':' + os.environ['LD_LIBRARY_PATH']})
    ret.wait()

def main():
    print('run_with_plugins has been renamed to megflow_run.')

if __name__ == '__main__':
    run()
