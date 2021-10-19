#!/bin/bash -ex

cd flow-python

python3 setup.py install

cd examples

cargo run --example run_with_plugins -- -p logical_test
