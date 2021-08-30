#!/bin/bash
let "WID = $1 % 4 + 1"
cache=/mnt/data/gitlab-runner/.megflow.$WID

if [ ! -d "$cache" ]; then
  mkdir $cache
fi

rm -rf target
ln -s $cache ./target
