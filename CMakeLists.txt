cmake_minimum_required(VERSION 3.22.1)

project(watchcheckcode LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

add_library(watchcheckcode SHARED
    src/watchcheckcode.cpp)

target_compile_options(watchcheckcode PRIVATE
    -fvisibility=hidden
    -fvisibility-inlines-hidden
    -fno-exceptions
    -fno-rtti
    -Wall
    -Wextra)

target_link_options(watchcheckcode PRIVATE
    "LINKER:--exclude-libs,ALL")

find_library(android-lib android)
find_library(log-lib log)
find_library(m-lib m)
find_library(dl-lib dl)

target_link_libraries(watchcheckcode PRIVATE
    ${android-lib}
    ${log-lib}
    ${m-lib}
    ${dl-lib})
