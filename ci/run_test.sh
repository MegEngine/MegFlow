#!/bin/bash -ex

cd flow-python

python3 setup.py install

cd examples

cargo run --example megflow_run -- -p logical_test
