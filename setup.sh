#!/usr/bin/env bash

ENV_OUTPUT=${ENV_OUTPUT:-/dev/null}
TARGET=${TARGET:-x86_64-unknown-linux-gnu}

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

mkdir -p libs

function copy-lib {
  local source_path="$1/$2"
  local target_path="libs/$2-${TARGET}"
  echo "Copying ${source_path} -> ${target_path}"
  cp "$source_path" "$target_path"
}

# Echo `SWIPL` env var for the `swipl-info` crate. On some platforms there seems
# to be an issue with `swipl` not being added to `PATH`.
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
  if ! command -v swipl; then
    swipl_path=$(ldconfig -p | grep "libswipl.so$" | sed 's/^.*=> //' | dirname)
    swipl_path=${swipl_path:-$(pkg-config --libs-only-L swipl | tr -d ' ' | sed 's/-L//')}

    if [[ -f "$swipl_path" ]]; then
      echo "SWIPL=${swipl_path}/swipl" | tee "$ENV_OUTPUT"
      copy-lib "$swipl_path" libswipl.so
    else
      echo "Searched ldconfig and pkg-config, but did not find swipl"
      echo "ldconfig -p:"
      ldconfig -p
      echo "pkg-config --libs-only-L ${ENV_OUTPUT}:"
      pkg-config --libs-only-L swipl
      exit 1
    fi
  fi
else
  echo "Unrecognized OSTYPE: ${OSTYPE}, skipped searching for swipl"
fi
