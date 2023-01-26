#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR=$(pwd)

# Actors
CONTRACTS_ADMIN=//Alice
DAMIAN=//0
HANS=//1
DAMIAN_ACCOUNT=5D34dL5prEUaGNQtPPZ3yN5Y6BnkfXunKXXz6fo7ZJbLwRRH
HANS_ACCOUNT=5GBNeWRhZc2jXu7D55rBimKYDk8PGk8itRYFTPfC8RJLKG5o

# Token economics
TOTAL_TOKEN_ISSUANCE_PER_CONTRACT=2000
TOKEN_PER_PERSON=1000
TOKEN_ALLOWANCE=500

DEPOSIT_VK_BYTES="0x$(cat deposit.vk.bytes | xxd -ps | tr -d '\n')"
DEPOSIT_AND_MERGE_VK_BYTES="0x$(cat deposit_and_merge.vk.bytes | xxd -ps | tr -d '\n')"
WITHDRAW_VK_BYTES="0x$(cat withdraw.vk.bytes | xxd -ps | tr -d '\n')"

MERKLE_LEAVES=65536

usage() {
  cat << EOF
Sets up the environment for testing Shielder application. Precisely:
 - we build and deploy token contracts (each with 2000 tokens of initial supply) and the Shielder contract
 - we endow //0 and //1 with 1000 tokens each (of both types)
 - for both tokens, for both actors, we set allowance for Shielder to spend up to 500 tokens
 - we register verifying key for both 'deposit' and 'withdraw' relation
 - we register both token contracts

Make sure to have "cargo contract" installed (version 2.0.0-beta.1).
EOF
}

while getopts n:k: flag
do
  case "${flag}" in
    n) NODE=${OPTARG};;
    *)
      usage
      exit
      ;;
  esac
done

# defaults

NODE="${NODE:-ws://127.0.0.1:9944}"

# Command shortcuts
INSTANTIATE_CMD="cargo contract instantiate --skip-confirm --url ${NODE} --suri ${CONTRACTS_ADMIN} --output-json"
CALL_CMD="cargo contract call --quiet --skip-confirm  --url ${NODE}"

# Contract addresses
TOKEN_A_ADDRESS=""
TOKEN_B_ADDRESS=""
SHIELDER_ADDRESS=""

get_timestamp() {
  echo "$(date +'%Y-%m-%d %H:%M:%S')"
}

error() {
  echo -e "[$(get_timestamp)] [ERROR] $*"
  exit 1
}

log_progress() {
  bold=$(tput bold)
  normal=$(tput sgr0)
  echo "[$(get_timestamp)] [INFO] ${bold}${1}${normal}"
}

random_salt() {
  hexdump -vn16 -e'4/4 "%08X" 1 "\n"' /dev/urandom
}

build_token_contract() {
  cd "${ROOT_DIR}"/public_token/
  cargo contract build --quiet --release 1> /dev/null 2> /dev/null
}

deploy_token_contracts() {
  cd "${ROOT_DIR}"/public_token/

  TOKEN_A_ADDRESS=$($INSTANTIATE_CMD --args "${TOTAL_TOKEN_ISSUANCE_PER_CONTRACT}" --salt "0x$(random_salt)" | jq -r '.contract')
  echo "Token A address: ${TOKEN_A_ADDRESS}"

  TOKEN_B_ADDRESS=$($INSTANTIATE_CMD --args "${TOTAL_TOKEN_ISSUANCE_PER_CONTRACT}" --salt "0x$(random_salt)" | jq -r '.contract')
  echo "Token B address: ${TOKEN_B_ADDRESS}"
}

distribute_tokens() {
  cd "${ROOT_DIR}"/public_token/
  $CALL_CMD --contract "${TOKEN_A_ADDRESS}" --message "PSP22::transfer" --args "${DAMIAN_ACCOUNT}" "${TOKEN_PER_PERSON}" "0x00" --suri "${CONTRACTS_ADMIN}" | grep "Success"
  $CALL_CMD --contract "${TOKEN_A_ADDRESS}" --message "PSP22::transfer" --args "${HANS_ACCOUNT}" "${TOKEN_PER_PERSON}" "0x00" --suri "${CONTRACTS_ADMIN}" | grep "Success"

  $CALL_CMD --contract "${TOKEN_B_ADDRESS}" --message "PSP22::transfer" --args "${DAMIAN_ACCOUNT}" "${TOKEN_PER_PERSON}" "0x00" --suri "${CONTRACTS_ADMIN}" | grep "Success"
  $CALL_CMD --contract "${TOKEN_B_ADDRESS}" --message "PSP22::transfer" --args "${HANS_ACCOUNT}" "${TOKEN_PER_PERSON}" "0x00" --suri "${CONTRACTS_ADMIN}" | grep "Success"
}

set_allowances() {
  cd "${ROOT_DIR}"/public_token/
  $CALL_CMD --contract "${TOKEN_A_ADDRESS}" --message "PSP22::approve" --args "${SHIELDER_ADDRESS}" "${TOKEN_ALLOWANCE}" --suri "${DAMIAN}" | grep "Success"
  $CALL_CMD --contract "${TOKEN_B_ADDRESS}" --message "PSP22::approve" --args "${SHIELDER_ADDRESS}" "${TOKEN_ALLOWANCE}" --suri "${DAMIAN}" | grep "Success"

  $CALL_CMD --contract "${TOKEN_A_ADDRESS}" --message "PSP22::approve" --args "${SHIELDER_ADDRESS}" "${TOKEN_ALLOWANCE}" --suri "${HANS}" | grep "Success"
  $CALL_CMD --contract "${TOKEN_B_ADDRESS}" --message "PSP22::approve" --args "${SHIELDER_ADDRESS}" "${TOKEN_ALLOWANCE}" --suri "${HANS}" | grep "Success"
}

build_shielder_contract() {
  cd "${ROOT_DIR}"/contract/
  cargo contract build --quiet --release 1> /dev/null 2> /dev/null
}

deploy_shielder_contract() {
  cd "${ROOT_DIR}"/contract/
  SHIELDER_ADDRESS=$($INSTANTIATE_CMD --args "${MERKLE_LEAVES}" --salt "0x$(random_salt)" | jq -r '.contract')
  echo "Shielder address: ${SHIELDER_ADDRESS}"
  cp "${ROOT_DIR}"/contract/target/ink/metadata.json "${ROOT_DIR}"/cli/shielder-metadata.json
}

register_vk() {
  cd "${ROOT_DIR}"/contract/
  $CALL_CMD --contract "${SHIELDER_ADDRESS}" --message "register_vk" --args Deposit "${DEPOSIT_VK_BYTES}" --suri "${CONTRACTS_ADMIN}" | grep "Success"
  $CALL_CMD --contract "${SHIELDER_ADDRESS}" --message "register_vk" --args DepositAndMerge "${DEPOSIT_AND_MERGE_VK_BYTES}" --suri "${CONTRACTS_ADMIN}" | grep "Success"
  $CALL_CMD --contract "${SHIELDER_ADDRESS}" --message "register_vk" --args Withdraw "${WITHDRAW_VK_BYTES}" --suri "${CONTRACTS_ADMIN}" | grep "Success"
}

register_tokens() {
  cd "${ROOT_DIR}"/contract/
  $CALL_CMD --contract "${SHIELDER_ADDRESS}" --message "register_new_token" --args 0 "${TOKEN_A_ADDRESS}" --suri "${CONTRACTS_ADMIN}" | grep "Success"
  $CALL_CMD --contract "${SHIELDER_ADDRESS}" --message "register_new_token" --args 1 "${TOKEN_B_ADDRESS}" --suri "${CONTRACTS_ADMIN}" | grep "Success"
}

set_up_shielding() {

  log_progress "Building token contract..."
  build_token_contract || error "Failed to build token contract"

  log_progress "Deploying token contracts..."
  deploy_token_contracts || error "Failed to deploy token contracts"

  log_progress "Distributing tokens..."
  distribute_tokens || error "Failed to distribute tokens"

  log_progress "Building Shielder contract..."
  build_shielder_contract || error "Failed to build Shielder contract"

  log_progress "Deploying Shielder contract..."
  deploy_shielder_contract || error "Failed to deploy Shielder contract"

  log_progress "Setting allowances for Shielder..."
  set_allowances || error "Failed to set allowances"

  log_progress "Registering verifying keys..."
  register_vk || error "Failed to register verifying keys"

  log_progress "Registering token contracts..."
  register_tokens || error "Failed to register token contracts"
}

set_up_shielding
