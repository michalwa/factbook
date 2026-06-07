#!/usr/bin/env bash

if command -v apt; then
  apt-add-repository ppa:swi-prolog/stable
  apt update
  apt install libwebkit2gtk-4.1-dev \
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
  brew install swi-prolog@10.0.2
else
  echo "No supported package manager found"
  exit 1
fi
