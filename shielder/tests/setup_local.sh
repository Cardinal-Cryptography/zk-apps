#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)

get_timestamp() {
  date +'%Y-%m-%d %H:%M:%S'
}

log_progress() {
  bold=$(tput bold)
  normal=$(tput sgr0)
  echo "[$(get_timestamp)] [INFO] ${bold}${1}${normal}"
}


function copy_metadata() {
    cp ${SCRIPT_DIR}/../contract/target/ink/shielder.json ${SCRIPT_DIR}/resources/
    cp ${SCRIPT_DIR}/../public_token/target/ink/public_token.json ${SCRIPT_DIR}/resources/
    log_progress "✅ Contracts' metadata was made visible to tests"
}

function copy_addresses() {
    cp ${SCRIPT_DIR}/../deploy/addresses.json ${SCRIPT_DIR}/resources/
    log_progress "✅ addresses.json moved to tests/resources"
}

copy_metadata
copy_addresses