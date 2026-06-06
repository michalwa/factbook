#!/usr/bin/env bash

set -e

SWIPL_VERSION=10.0.2

if [[ -n "$CI" ]]; then
  apt update
  apt install \
    build-essential \
    cmake \
    ninja-build \
    pkg-config \
    ncurses-dev \
    libedit-dev \
    libgoogle-perftools-dev \
    libgmp-dev \
    libssl-dev \
    zlib1g-dev \
    libarchive-dev \
    libossp-uuid-dev \
    libdb-dev \
    libpcre2-dev \
    libyaml-dev \
    libutf8proc-dev
fi

mkdir -p deps
pushd deps

if ! [[ -d deps/swipl ]]; then
  if ! [[ -f swipl-$SWIPL_VERSION.tar.gz ]]; then
    wget https://www.swi-prolog.org/download/stable/src/swipl-$SWIPL_VERSION.tar.gz
  fi

  mkdir -p swipl
  tar -xzf swipl-$SWIPL_VERSION.tar.gz --strip-components=1 -C swipl
fi

pushd swipl
rm -rf build
mkdir build
pushd build
cmake -DCMAKE_BUILD_TYPE=PGO \
  -DSWIPL_STATIC_LIB=ON \
  -DBUILD_TESTING=OFF \
  -DBUILD_SWIPL_LD=OFF \
  -DINSTALL_DOCUMENTATION=OFF \
  -G Ninja ..
ninja
popd
popd
