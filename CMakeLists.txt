cmake_minimum_required(VERSION 3.0)
project(pcc)
# set(CMAKE_VERBOSE_MAKEFILE ON)

#
# Dependencies
#

# include_directories(deps)

#
# Targets
#

file(GLOB_RECURSE source_files src/*.c)
add_executable(${PROJECT_NAME} ${source_files})

target_compile_options(${PROJECT_NAME} PUBLIC -std=c11 -g -static -Wall)
