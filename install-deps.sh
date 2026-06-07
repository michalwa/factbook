#!/usr/bin/env bash

if command -v apt; then
  sudo apt-add-repository ppa:swi-prolog/stable
  sudo apt update
  sudo apt install libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libxdo-dev \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    swi-prolog=10.0.2-*
elif command -v brew; then
  brew install swi-prolog
else
  echo "No supported package manager found"
  exit 1
fi

if [[ -n "$1" ]]; then
  # Ensure `swipl` is in PATH, for some reason not the default on Ubuntu in CI
  if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if ! command -v swipl; then
      path=$(ldconfig -p | grep "libswipl.so$" | sed 's/^.*=> //' | dirname)
      path=${path:-$(pkg-config --libs-only-L $1 | tr -d ' ' | sed 's/-L//')}
      echo "swipl location: $path"
      echo $path >> $1
    fi
  else
    echo "unrecognized OSTYPE: ${OSTYPE}, skip adding swipl to PATH"
  fi
fi
