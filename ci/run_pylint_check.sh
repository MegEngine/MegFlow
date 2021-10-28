python -m pip install pylint==2.5.2
CHECK_DIR="flow-python/examples/simple_classification flow-python/examples/simple_det_classify flow-python/examples/cat_finder flow-python/examples/electric_bicycle  flow-python/examples/misc"
pylint $CHECK_DIR || pylint_ret=$?
if [ "$pylint_ret" ]; then
    exit $pylint_ret
fi
