#!/usr/bin/env bash

# Copy shared libraries into the project directory for bundling with `tauri build`

set -e

IMPORTED_LIBS_PATH=libs
IMPORTED_LIBS_TARGET=x86_64-unknown-linux-gnu

function import-lib {
    local imported_path="${IMPORTED_LIBS_PATH}/$(basename $1)-${IMPORTED_LIBS_TARGET}"
    echo "Copying $1 -> ${imported_path}"
    cp $1 ${imported_path}
}

mkdir -p $IMPORTED_LIBS_PATH
import-lib ${LIBSWIPL_PATH:-$(ldconfig -p | grep libswipl.so$ | sed 's/^.*=> //')}
