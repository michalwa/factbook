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
fi

mkdir -p libs

function copy-lib {
  local source_path="$1/$2"
  local target_path="libs/$2"
  echo "Copying ${source_path} -> ${target_path}"
  cp "$source_path" "$target_path"
}

libswipl_dir=${libswipl_dir:-$(ldconfig -p | grep libswipl | sed 's/^.*=> //' | xargs dirname)}
libswipl_dir=${libswipl_dir:-$(pkg-config --libs-only-L swipl | tr -d ' ' | sed 's/-L//')}

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
  copy-lib "$libswipl_dir" libswipl.so.10
elif [[ "$OSTYPE" == "darwin"* ]]; then
  copy-lib "$libswipl_dir" libswipl.10.dylib
fi
