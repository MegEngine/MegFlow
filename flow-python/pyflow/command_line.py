import os
import sys
import subprocess
import pkg_resources
def main():
    bin_exist = pkg_resources.resource_exists('pyflow', 'run_with_plugins')
    if not bin_exist:
        print('cannot find run_with_plugins, exit!')
        sys.exit(-1)
    bin_path = pkg_resources.resource_filename('pyflow', 'run_with_plugins')

    sys.argv[0] = bin_path
    ret = subprocess.Popen(sys.argv)
    ret.wait()

if __name__ == '__main__':
    main()


