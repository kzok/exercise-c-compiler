#!/bin/bash
set -eu

cd $(dirname $0)
./build.sh

#
# End-to-end tests
#

cd ./out

cat <<EOF | gcc -xc -c -o tmp2.o -
int add(int a, int b) {return a + b;}
int sub(int a, int b) {return a - b;}
int sum6(int a, int b, int c, int d, int e, int f) {return a + b + c + d + e + f;}
EOF

try() {
	expected="$1"
	input="$2"

	set +e
	./pcc "$input" > tmp.s
	gcc -o tmp tmp.s tmp2.o
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
try 0 "main() { return 0; }"
try 42 "main() { return 42; }"

# calculation
try 21 "main() { return 5+20-4; }"
try 41 "main() { return  12 + 34    - 5; }"
try 47 "main() { return 5+6*7; }"
try 15 "main() { return 5 * (9 - 6); }"
try 4 "main() { return (3   + 5) / 2; }"
try 10 "main() { return -  10 +20; }"

# comparison
try 0 "main() { return 1 > 2; }"
try 1 "main() { return 1 < 2; }"
try 0 "main() { return 2 < 2; }"
try 1 "main() { return 2 <= 2; }"
try 0 "main() { return 2 <= 1; }"

# variable
try 3 "main() { a = 1; b = a + 2; return b; }"
try 6 "main() { foo = 1; bar = 2 + 3; return foo + bar; }"

# return
try 3 "main() { return 3; }"
try 5 "main() { return 5; return 8; }"
try 1 "main() { return 1; 2; 3; }"
try 2 "main() { 1; return 2; 3; }"
try 3 "main() { 1; 2; return 3; }"

# if, else
try 2 "main() { if ( 1 ) return 2 ; return 3 ; }"
try 3 "main() { if ( 0 ) return 2 ; return 3 ; }"
try 3 "main() { a=1; if(1)a=a+2; if(0)a=a+3; return a; }"
try 1 "main() { a=0; if ( 1 ) a=1; else a=2; return a; }"
try 2 "main() { a=0; if ( 0 ) a=1; else a=2; return a; }"

# while
try 10 "main() { a=0; while(a<10)a=a+1; return a; }"

# for
try 10 "main() { a=11; for(a=0;a<10;a=a+1) a; }"
try 11 "main() { a=11; for(;a<10;a=a+1) a; a; }"
try 10 "main() { a=11; for(a=0;a<10;) a=a+1; a; }"
try 5 "main() { a=0; for(a=9;a>5;a=a-1) a; }"

# block
try 3 "main() { a=0; if(1){a=a+1; a=a+2;} else a=9; return a; }"

# external function call
try 3 "main() { return add(1, 2); }"
try 5 "main() { return sub(8, 3); }"
try 21 "main() { return sum6(1,2,3,4,5,6); }"

# argument-less internal function definition
try 32 "main() { return ret32(); } ret32() { 32; }"

echo -e '\e[32mAll tests passed!\e[0m'
