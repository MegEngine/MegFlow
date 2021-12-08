#!/bin/bash -e

python -m pip install pylint==2.5.2

CHECK_DIR="flow-python/examples/application/simple_classification"
CHECK_DIR+=" flow-python/examples/application/simple_det_classify"
CHECK_DIR+=" flow-python/examples/application/cat_finder"
CHECK_DIR+=" flow-python/examples/application/electric_bicycle"
CHECK_DIR+=" flow-python/examples/application/misc"

pylint $CHECK_DIR 
