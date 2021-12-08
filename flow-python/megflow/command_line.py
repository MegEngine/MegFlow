#!/usr/bin/env python
# coding=utf-8

import sys
import os
import subprocess
import pkg_resources

def megflow_run():
    if pkg_resources.resource_exists('megflow', 'lib') and pkg_resources.resource_isdir('megflow', 'lib'):
        if 'MGF_CMDLINE_FLAG' not in os.environ:
            os.environ['MGF_CMDLINE_FLAG'] = '1'
            if 'LD_LIBRARY_PATH' in os.environ:
                os.environ['LD_LIBRARY_PATH'] = pkg_resources.resource_filename('megflow', 'lib') + ':' + os.environ['LD_LIBRARY_PATH']
            else:
                os.environ['LD_LIBRARY_PATH'] = pkg_resources.resource_filename('megflow', 'lib')
	    
            try:
                os.execv(sys.argv[0], sys.argv)
                return
            except Exception as exc:
                print('Failed re-exec:', exc)
                sys.exit(1)

    from .megflow import RunConfig, run
    import argparse

    parser = argparse.ArgumentParser(prog='megflow_run', description='run a pipeline with plugins.')
    parser.add_argument('--dump', help='the path to dump graph', action='store_true')
    parser.add_argument('--debug', metavar='PORT', type=str, help='debug server port')
    parser.add_argument('-p', '--plugin', required=True, type=str, help='plugin path')
    parser.add_argument('-m', '--module', type=str, help='module path')
    parser.add_argument('-c', '--config', type=str, help='config path')

    args = parser.parse_args()

    run_cfg = RunConfig()
    run_cfg.dump = args.dump
    run_cfg.debug = args.debug
    run_cfg.plugin_path = args.plugin
    run_cfg.module_path = args.module
    run_cfg.config_path = args.config

    run(run_cfg)


def run_with_plugins():
    print('run_with_plugins has been renamed to megflow_run.')


def megflow_quickstart():
    bin_path = pkg_resources.resource_filename('megflow', 'megflow_quickstart_inner')
    sys.argv[0] = bin_path
    ret = subprocess.Popen(sys.argv)
    ret.wait()


if __name__ == '__main__':
    megflow_run()
