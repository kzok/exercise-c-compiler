#!/bin/bash -eu

cd $(dirname $0)

pushd ../..

cargo test
cargo build

popd

../../target/debug/pcc "$(cat ./main.c)" > tmp.s
cc -no-pie -o tmp tmp.s
./tmp
echo
