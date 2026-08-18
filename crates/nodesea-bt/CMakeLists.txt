# SPDX-License-Identifier: MIT
# Copyright (c) 2026 HualiNox

cmake_minimum_required(VERSION 3.24)

project(nodesea_bt LANGUAGES CXX)

include(FetchContent)

# Build configuration
set(CMAKE_BUILD_TYPE Release CACHE STRING "" FORCE)
set(BUILD_SHARED_LIBS OFF CACHE BOOL "" FORCE)

set(build_tests OFF CACHE BOOL "" FORCE)
set(build_examples OFF CACHE BOOL "" FORCE)
set(build_tools OFF CACHE BOOL "" FORCE)
set(python-bindings OFF CACHE BOOL "" FORCE)

# The Rust build script compiles the FFI wrapper. CMake only builds libtorrent.
set(
    CMAKE_CXX_STANDARD 20
    CACHE STRING
    ""
    FORCE
)

# boost - don't have source
set(BOOST_VERSION 1.92.0)
set(BOOST_RELEASE_BASE "https://github.com/boostorg/boost/releases/download")
set(BOOST_ARCHIVE "boost-${BOOST_VERSION}-cmake.tar.xz")
string(
    CONCAT BOOST_URL
    "${BOOST_RELEASE_BASE}"
    "/boost-${BOOST_VERSION}"
    "/${BOOST_ARCHIVE}"
)

FetchContent_Declare(
    boost_src
    URL "${BOOST_URL}"
)

set(BOOST_ENABLE_COMPATIBILITY_TARGETS ON CACHE BOOL "" FORCE)
set(BOOST_SKIP_INSTALL_RULES OFF CACHE BOOL "" FORCE)

FetchContent_MakeAvailable(boost_src)

# libtorrent
FetchContent_Declare(
    libtorrent
    GIT_REPOSITORY https://github.com/arvidn/libtorrent.git

    GIT_TAG v2.1.1
)

FetchContent_MakeAvailable(libtorrent)

# Install boost headers
file(
    GLOB BOOST_INCLUDE_DIRS
    LIST_DIRECTORIES true
    "${boost_src_SOURCE_DIR}/libs/*/include"
)

foreach(BOOST_INCLUDE_DIR IN LISTS BOOST_INCLUDE_DIRS)
  install(
        DIRECTORY "${BOOST_INCLUDE_DIR}/"
        DESTINATION include
    )
endforeach()

# Enable interprocedural optimization
include(CheckIPOSupported)

check_ipo_supported(
    RESULT IPO_SUPPORTED
    OUTPUT IPO_ERROR
)

if(IPO_SUPPORTED)
  set_property(
        TARGET torrent-rasterbar
        PROPERTY INTERPROCEDURAL_OPTIMIZATION TRUE
    )
endif()
