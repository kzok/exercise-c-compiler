#!/bin/bash -eu

cd $(dirname $0)

cargo test
cargo build

assert() {
  expected="$1"
  input="$2"

  ./target/debug/pcc "$input" > tmp.s
  cc -o tmp tmp.s
  set +e
  ./tmp
  actual="$?"
  set -e

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected expected, but got $actual"
    exit 1
  fi
}

echo "===== E2E testing ====="

assert 0 0
assert 42 42

assert 21 "5+20-4"
assert 41 " 12 + 34 - 5 "

echo -e '\e[32mAll tests passed!\e[0m'
