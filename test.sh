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

assert 0 "0;"
assert 42 "42;"

assert 21 "5+20-4;"
assert 41 " 12 + 34 - 5 ;"

assert 47 '5+6*7;'
assert 15 '5*(9-6);'
assert 4 '(3+5)/2;'

assert 3 '+1+2;'
assert 10 '-10+20;'

assert 0 '0==1;'
assert 1 '42==42;'
assert 1 '0!=1;'
assert 0 '42!=42;'

assert 1 '0<1;'
assert 0 '1<1;'
assert 0 '2<1;'
assert 1 '0<=1;'
assert 1 '1<=1;'
assert 0 '2<=1;'

assert 1 '1>0;'
assert 0 '1>1;'
assert 0 '1>2;'
assert 1 '1>=0;'
assert 1 '1>=1;'
assert 0 '1>=2;'

assert 3 'a=3; a;'
assert 8 'a=3; z=5; a+z;'
assert 6 'a=b=3; a+b;'

assert 6 'foo=1; bar=2+3; foo+bar;'

assert 0 'return 0;'
assert 5 'return 5; return 8;'
assert 21 'return 5+20-4;'

echo -e '\e[32mAll tests passed!\e[0m'
