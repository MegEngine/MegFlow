#!/bin/bash -ex

python3 ci/convert_project_to_github.py

docker build --rm -t megflow -f Dockerfile.github-dev .