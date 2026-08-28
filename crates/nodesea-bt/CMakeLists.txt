# SPDX-License-Identifier: MIT
# Copyright (c) 2026 HualiNox

cmake_minimum_required(VERSION 3.24)

project(nodesea_bt LANGUAGES CXX)

include(FetchContent)

# Libtorrent build policy. CMake derives the build type from Cargo's profile.
set(BUILD_SHARED_LIBS OFF CACHE BOOL "" FORCE)

set(build_tests OFF CACHE BOOL "" FORCE)
set(build_examples OFF CACHE BOOL "" FORCE)
set(build_tools OFF CACHE BOOL "" FORCE)
set(python-bindings OFF CACHE BOOL "" FORCE)

# Keep the protocol feature set identical for every Cargo profile. The Rust
# build script owns the C++ wrapper language standard and matching header
# definitions; libtorrent declares its own C++17 requirement.
set(dht ON CACHE BOOL "" FORCE)
set(extensions ON CACHE BOOL "" FORCE)
set(deprecated-functions ON CACHE BOOL "" FORCE)
set(encryption ON CACHE BOOL "" FORCE)
set(exceptions ON CACHE BOOL "" FORCE)
set(webtorrent OFF CACHE BOOL "" FORCE)
set(i2p OFF CACHE BOOL "" FORCE)

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

# Help libtorrent's FindBoost locate the headers fetched above instead of
# relying on host-specific search paths such as /usr/include.
set(BOOST_ROOT "${boost_src_SOURCE_DIR}" CACHE PATH "" FORCE)
set(Boost_ROOT "${boost_src_SOURCE_DIR}" CACHE PATH "" FORCE)
set(Boost_INCLUDE_DIR "${boost_src_SOURCE_DIR}" CACHE PATH "" FORCE)
set(Boost_NO_SYSTEM_PATHS ON CACHE BOOL "" FORCE)

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

# Use IPO only for CMake's Release configuration. Debug stays free of IPO and
# extra optimization policy for native build and incremental-development speed.
include(CheckIPOSupported)

check_ipo_supported(
    RESULT IPO_SUPPORTED
    OUTPUT IPO_ERROR
    LANGUAGES CXX
)

if(IPO_SUPPORTED)
  set_property(
        TARGET torrent-rasterbar
        PROPERTY INTERPROCEDURAL_OPTIMIZATION_RELEASE TRUE
    )
else()
  message(STATUS "Release IPO is unavailable for libtorrent: ${IPO_ERROR}")
endif()
