#!/usr/bin/env bash

# Copy shared libraries into the project directory for bundling with `tauri build`

set -e

IMPORTED_PATH=libs

function import-lib {
    local imported_path="${IMPORTED_PATH}/$(basename $1)"
    echo "Copying $1 -> ${imported_path}"
    cp $1 ${imported_path}
}

mkdir -p $IMPORTED_PATH
import-lib ${LIBSWIPL_PATH:-$(ldconfig -p | grep libswipl.so$ | sed 's/^.*=> //')}
