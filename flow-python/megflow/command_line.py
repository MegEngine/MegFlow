#!/usr/bin/env python
# coding=utf-8
import sys
import subprocess
import pkg_resources
from find_libpython import find_libpython


def main():
    bin_exist = pkg_resources.resource_exists('megflow', 'run_with_plugins_inner')
    if not bin_exist:
        print('cannot find run_with_plugins, exit!')
        sys.exit(-1)
    bin_path = pkg_resources.resource_filename('megflow', 'run_with_plugins_inner')

    pylib_dir = os.path.dirname(find_libpython())

    sys.argv[0] = bin_path
    ret = subprocess.Popen(sys.argv, env={**os.environ, 'LD_LIBRARY_PATH': pylib_dir + ':' + os.environ['LD_LIBRARY_PATH']})
    ret.wait()


if __name__ == '__main__':
    main()
