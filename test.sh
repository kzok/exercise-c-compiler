#!/bin/bash

try() {
	expected="$1"
	input="$2"

	./9cc "$input" > tmp.s
	gcc -o tmp tmp.s
	./tmp
	actual="$?"

	if [ "$actual" = "$expected" ]; then
		echo "$input => $actual"
	else
		echo "$input => $expected expected, but go $actual"
		exit 1
	fi
}

try 0 0
try 42 42
try 21 "5+20-4"
# 終了コードの最大値は 255 なのでオーバーフローして 0 になる
try 256 0

echo OK
