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
    unixodbc-dev \
    zlib1g-dev \
    libarchive-dev \
    libossp-uuid-dev \
    libdb-dev \
    libpcre2-dev \
    libyaml-dev \
    libutf8proc-dev \
    python3 \
    libpython3-dev \
    default-jdk \
    junit4
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
cmake -DCMAKE_BUILD_TYPE=PGO -DSWIPL_STATIC_LIB=ON -G Ninja ..
ninja
popd
popd
