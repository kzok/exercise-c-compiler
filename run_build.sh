#!/bin/bash -eu

cd $(dirname $0)
mkdir -p ./build
cd ./build
cmake ..
make pcc
