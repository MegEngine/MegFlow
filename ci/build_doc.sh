#!/bin/bash -ex

rm -rf $1

: ${TARGET_DIR:=/target/${CI_COMMIT_SHA}}

cargo doc --no-deps

echo '<meta http-equiv=refresh content=0;url=flow_rs/index.html>' > ${TARGET_DIR}/doc/index.html 

mv ${TARGET_DIR}/doc $1
