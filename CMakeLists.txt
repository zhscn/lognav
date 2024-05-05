cmake_minimum_required(VERSION 3.14.0)
project(lognav VERSION 0.1.0 LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 23)

add_executable(lognav)
target_sources(lognav PRIVATE main.cc)
