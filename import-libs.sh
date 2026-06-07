#!/usr/bin/env bash

# Copy shared libraries into the project directory for bundling with `tauri build`

set -e

IMPORTED_LIBS_PATH=libs
IMPORTED_LIBS_TARGET=x86_64-unknown-linux-gnu

# import-lib <pkg-name> <lib-name> <lib-path>
function import-lib {
  local path="$3"
  path=${path:-$(ldconfig -p | grep "$2$" | sed 's/^.*=> //')}
  path=${path:-$(pkg-config --libs-only-L $1 | tr -d ' ' | sed 's/-L//')/$2}

  if ! [[ -f "$path" ]]; then
    echo "Shared library missing: $1"
    echo "ldconfig -p:"
    ldconfig -p
    exit 1
  fi

  local imported_path="${IMPORTED_LIBS_PATH}/$(basename $path)-${IMPORTED_LIBS_TARGET}"
  echo "Copying $path -> ${imported_path}"
  cp $path ${imported_path}
}

mkdir -p ${IMPORTED_LIBS_PATH}
import-lib swipl libswipl.so ${LIBSWIPL_PATH}
