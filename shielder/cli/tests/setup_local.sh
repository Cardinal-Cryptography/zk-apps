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

function setup_testdir() {
    mkdir -p ${SCRIPT_DIR}/resources/
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

function copy_proving_keys() {
  cp ${BASE_DIR}/cli/deposit.pk.bytes ${SCRIPT_DIR}/resources/
  cp ${BASE_DIR}/cli/deposit_and_merge.pk.bytes ${SCRIPT_DIR}/resources/
  cp ${BASE_DIR}/cli/merge.pk.bytes ${SCRIPT_DIR}/resources/
  cp ${BASE_DIR}/cli/withdraw.pk.bytes ${SCRIPT_DIR}/resources/
  log_progress "✅ Proving keys copied to tests/resources"
}

setup_testdir
copy_metadata
copy_addresses
copy_proving_keys
