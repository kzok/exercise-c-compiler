#!/bin/bash -eu

cd $(dirname $0)
mkdir -p ./out
cd ./out
cmake ..
make depend
make pcc
