#!/bin/bash
set -eu

cd $(dirname $0)
./run_build.sh

#
# End to end tests
#

cd ./build

try() {
	expected="$1"
	input="$2"

	set +e
	./pcc "$input" > tmp.s
	gcc -o tmp tmp.s
	./tmp
	actual="$?"
	set -e

	if [ "$actual" = "$expected" ]; then
		echo "$input => $actual"
	else
		echo "$input => $expected expected, but go $actual"
		exit 1
	fi
}

try 0 "0;"
try 42 "42;"
try 21 "5+20-4;"
try 41 " 12 + 34    - 5;"
try 47 "5+6*7;"
try 15 "5 * (9 - 6);"
try 4 "(3   + 5) / 2;"
try 10 "-  10 +20;"
try 0 "1 > 2;"
try 1 "1 < 2;"
try 0 "2 < 2;"
try 1 "2 <= 2;"
try 0 "2 <= 1;"
try 3 "a = 1; b = a + 2; b;"
try 6 "foo = 1; bar = 2 + 3; foo + bar;"
try 5 "return 5; return 8;"


echo OK
