#!/usr/bin/env python
# coding=utf-8
import os
import sys
import subprocess
import pkg_resources


def main():
    bin_exist = pkg_resources.resource_exists('megflow', 'run_with_plugins_inner')
    if not bin_exist:
        print('cannot find run_with_plugins, exit!')
        sys.exit(-1)
    bin_path = pkg_resources.resource_filename('megflow', 'run_with_plugins_inner')

    sys.argv[0] = bin_path
    ret = subprocess.Popen(sys.argv)
    ret.wait()


if __name__ == '__main__':
    main()
