#!/usr/bin/env python
# coding=utf-8
import os
import sys
import subprocess
import pkg_resources
from find_libpython import find_libpython

def exec(cmd_inner):
    bin_exist = pkg_resources.resource_exists('megflow', cmd_inner)
    if not bin_exist:
        print(f'cannot find {cmd_inner}, exit!')
        sys.exit(-1)
    bin_path = pkg_resources.resource_filename('megflow', cmd_inner)

    pylib_dir = os.path.dirname(find_libpython())
    sys.argv[0] = bin_path

    if 'LD_LIBRARY_PATH' in os.environ:
        ret = subprocess.Popen(sys.argv, env={**os.environ, 'LD_LIBRARY_PATH': pylib_dir + ':' + os.environ['LD_LIBRARY_PATH']})
    else:
        ret = subprocess.Popen(sys.argv)
    ret.wait()


def run():
    exec('megflow_run_inner')

def quickstart():
    exec('megflow_quickstart_inner')

def main():
    print('run_with_plugins has been renamed to megflow_run.')

if __name__ == '__main__':
    run()