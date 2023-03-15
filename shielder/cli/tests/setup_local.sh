#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
BASE_DIR=${SCRIPT_DIR}/../..

get_timestamp() {
  date +'%Y-%m-%d %H:%M:%S'
}

log_progress() {
  bold=$(tput bold)
  normal=$(tput sgr0)
  echo "[$(get_timestamp)] [INFO] ${bold}${1}${normal}"
}


function copy_metadata() {
    cp ${BASE_DIR}/contract/target/ink/shielder.json ${SCRIPT_DIR}/resources/
    cp ${BASE_DIR}/public_token/target/ink/public_token.json ${SCRIPT_DIR}/resources/
    log_progress "✅ Contracts' metadata copied to tests/resources"
}

function copy_addresses() {
    cp ${BASE_DIR}/deploy/addresses.json ${SCRIPT_DIR}/resources/
    log_progress "✅ addresses.json copied to tests/resources"
}

function copy_verification_keys() {
  cp ${BASE_DIR}/deploy/docker/keys/* ${SCRIPT_DIR}/resources/
  log_progress "✅ Verification keys copied to tests/resources"
}

copy_metadata
copy_addresses
copy_verification_keys