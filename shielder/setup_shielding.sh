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
TOTAL_TOKEN_ISSUANCE_PER_CONTRACT=4000
TOKEN_PER_PERSON=1000
TOKEN_ALLOWANCE=500

DEPOSIT_VK_BYTES="0x$(cat deposit.vk.bytes | xxd -ps | tr -d '\n')"
WITHDRAW_VK_BYTES="0x$(cat withdraw.vk.bytes | xxd -ps | tr -d '\n')"

MERKLE_LEAVES=65536

usage() {
  cat << EOF
Sets up the environment for testing Shielder application. Precisely:
 - we start local chain with "./scripts/run_nodes.sh -b false", so make sure that you have your binary already built in release mode,
 - we build and deploy token contracts (each with 2000 tokens of initial supply) and the Shielder contract
 - we endow //0 and //1 with 1000 tokens each (of both types)
 - for both tokens, for both actors, we set allowance for Shielder to spend up to 500 tokens
 - we register (dummy) verifying key for both 'deposit' and 'withdraw' relation
 - we register both token contracts

Make sure to have "cargo contract" installed (version 1.5).
EOF
}

while getopts r:n:k: flag
do
  case "${flag}" in
    n) NODE=${OPTARG};;
    k) REGISTER_KEYS=${OPTARG};;
    *)
      usage
      exit
      ;;
  esac
done

# defaults

REGISTER_KEYS="${REGISTER_KEYS:-false}"
NODE="${NODE:-ws://127.0.0.1:9944}"

# Command shortcuts
INSTANTIATE_CMD="cargo contract instantiate --skip-confirm --url ${NODE} --suri ${CONTRACTS_ADMIN}"
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
  result=$($INSTANTIATE_CMD --args "${TOTAL_TOKEN_ISSUANCE_PER_CONTRACT}" --salt 0x$(random_salt))
  TOKEN_A_ADDRESS=$(echo "$result" | grep Contract | tail -1 | cut -c 14-)
  echo "Token A address: ${TOKEN_A_ADDRESS}"

  result=$($INSTANTIATE_CMD --args "${TOTAL_TOKEN_ISSUANCE_PER_CONTRACT}" --salt 0x$(random_salt))
  TOKEN_B_ADDRESS=$(echo "$result" | grep Contract | tail -1 | cut -c 14-)
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
  result=$($INSTANTIATE_CMD --args ${MERKLE_LEAVES} --salt 0x$(random_salt))
  SHIELDER_ADDRESS=$(echo "$result" | grep Contract | tail -1 | cut -c 14-)
  echo "Shielder address: ${SHIELDER_ADDRESS}"
}

register_vk() {
  cd "${ROOT_DIR}"/contract/
  $CALL_CMD --contract "${SHIELDER_ADDRESS}" --message "register_vk" --args Deposit "${DEPOSIT_VK_BYTES}" --suri "${CONTRACTS_ADMIN}" | grep "Success"
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

  if [ $REGISTER_KEYS = true ]; then
    log_progress "Registering verifying keys..."
    register_vk || error "Failed to register verifying keys"
  fi

  log_progress "Registering token contracts..."
  register_tokens || error "Failed to register token contracts"
}

set_up_shielding