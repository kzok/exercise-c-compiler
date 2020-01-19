#!/bin/bash

docker run --rm -it -v "$PWD:/project" -w "/project" gcc:7.5.0
