#!/bin/bash
set -eu

cd $(dirname $0)
./build.sh

#
# End-to-end tests
#

cd ./out

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

# statement
try 0 "0;"
try 42 "42;"

# calculation
try 21 "5+20-4;"
try 41 " 12 + 34    - 5;"
try 47 "5+6*7;"
try 15 "5 * (9 - 6);"
try 4 "(3   + 5) / 2;"
try 10 "-  10 +20;"

# comparison
try 0 "1 > 2;"
try 1 "1 < 2;"
try 0 "2 < 2;"
try 1 "2 <= 2;"
try 0 "2 <= 1;"

# variable
try 3 "a = 1; b = a + 2; b;"
try 6 "foo = 1; bar = 2 + 3; foo + bar;"

# return
try 3 "return 3;"
try 5 "return 5; return 8;"

# if, else
try 2 "if ( 1 ) return 2 ; return 3 ;"
try 3 "if ( 0 ) return 2 ; return 3 ;"
try 3 "a=1; if(1)a=a+2; if(0)a=a+3; a;"
try 1 "a=0; if ( 1 ) a=1; else a=2; a;"
try 2 "a=0; if ( 0 ) a=1; else a=2; a;"

# while
try 10 "a=0; while(a<10)a=a+1; a;"

echo -e '\e[32mAll tests passed!\e[0m'
