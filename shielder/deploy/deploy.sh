#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)

# bump corresponding tag whenever a new version is released (updates should not be quite via `latest` tag)
export NODE_IMAGE=public.ecr.aws/p6e8q1z1/snarkeling:46c4726
export CLIAIN_IMAGE=public.ecr.aws/p6e8q1z1/cliain-snarkeling:46c4726
export CARGO_IMAGE=public.ecr.aws/p6e8q1z1/ink-dev:0.2.0

# actors
DAMIAN=//0
DAMIAN_PUBKEY=5D34dL5prEUaGNQtPPZ3yN5Y6BnkfXunKXXz6fo7ZJbLwRRH
HANS=//1
HANS_PUBKEY=5GBNeWRhZc2jXu7D55rBimKYDk8PGk8itRYFTPfC8RJLKG5o

ADMIN=//Alice
ADMIN_PUBKEY=5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY

MERKLE_TREE_HEIGHT=16
MERKLE_LEAVES=65536

# env
NODE="ws://127.0.0.1:9944"
DOCKER_USER="$(id -u):$(id -g)"
export DOCKER_USER

# tokenomics
TOTAL_TOKEN_ISSUANCE_PER_CONTRACT=2000
TOKEN_PER_PERSON=1000
TOKEN_ALLOWANCE=500

# command aliases
DOCKER_SH="docker run --rm -e RUST_LOG=debug -u ${DOCKER_USER} --entrypoint /bin/sh"

INSTANTIATE_CMD="cargo contract instantiate --skip-confirm --url ${NODE} --suri ${ADMIN} --output-json"
CALL_CMD="cargo contract call --quiet --skip-confirm  --url ${NODE}"

get_timestamp() {
  date +'%Y-%m-%d %H:%M:%S'
}

error() {
  echo -e "[$(get_timestamp)] [ERROR] ❌ $*"
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

prepare_fs() {
  # ensure that we are in shielder/deploy/ folder
  cd "${SCRIPT_DIR}"

  # forget everything from the past launches - start the chain from a scratch
  rm -rf docker/node_data/

  # ensure that all these folders are present
  mkdir -p docker/node_data/
  mkdir -p docker/keys/

  log_progress "✅ Directories are set up"
}

generate_chainspec() {
  CHAINSPEC_ARGS="\
    --base-path /data \
    --account-ids ${DAMIAN_PUBKEY} \
    --sudo-account-id ${ADMIN_PUBKEY} \
    --rich-account-ids ${DAMIAN_PUBKEY},${HANS_PUBKEY},${ADMIN_PUBKEY} \
    --chain-id a0smnet \
    --token-symbol SNZERO \
    --chain-name 'Aleph Zero Snarkeling'"

  $DOCKER_SH \
    -v "${SCRIPT_DIR}/docker/node_data:/data" \
    "${NODE_IMAGE}" \
    -c "aleph-node bootstrap-chain ${CHAINSPEC_ARGS} > /data/chainspec.snarkeling.json"

  log_progress "✅ Generated chainspec was written to docker/data/chainspec.snarkeling.json"
}

export_bootnode_address() {
  BOOTNODE_PEER_ID=$(
    $DOCKER_SH \
      -v "${SCRIPT_DIR}/docker/node_data:/data" \
      "${NODE_IMAGE}" \
      -c "aleph-node key inspect-node-key --file /data/${DAMIAN_PUBKEY}/p2p_secret"
  )
  export BOOTNODE_PEER_ID
  log_progress "✅ Exported bootnode address (${BOOTNODE_PEER_ID})"
}

run_snarkeling_node() {
  NODE_PUBKEY=$DAMIAN_PUBKEY docker-compose -f docker-compose.yml up --remove-orphans -d
  log_progress "✅ Successfully launched snarkeling node"
}

generate_relation_keys() {
  $DOCKER_SH \
    -v "${SCRIPT_DIR}/docker/keys:/workdir" \
    -w "/workdir" \
    "${CLIAIN_IMAGE}" \
    -c "/usr/local/bin/cliain snark-relation generate-keys ${1} ${2:-}"

  log_progress "✅ Generated keys for '${1}' relation"
}

generate_keys() {
  generate_relation_keys "deposit"
  generate_relation_keys "withdraw" "--max-path-len ${MERKLE_TREE_HEIGHT}"
}

move_keys() {
  mv docker/keys/deposit.groth16.pk.bytes ../cli/deposit.pk.bytes
  mv docker/keys/withdraw.groth16.pk.bytes ../cli/withdraw.pk.bytes

  log_progress "✅ Proving keys were made available to CLI"
}

build() {
  cd "${SCRIPT_DIR}"/../public_token/
  cargo contract build --quiet --release 1>/dev/null 2>/dev/null
  log_progress "✅ Public token contract was built"

  cd "${SCRIPT_DIR}"/../contract/
  cargo contract build --quiet --release 1>/dev/null 2>/dev/null
  log_progress "✅ Shielder contract was built"

  cd "${SCRIPT_DIR}"/../cli/
  cargo build --quiet --release 1>/dev/null 2>/dev/null
  log_progress "✅ CLI was built"
}

move_build_artifacts() {
  cp ../contract/target/ink/metadata.json ../cli/shielder-metadata.json
  log_progress "✅ Shielder metadata was made visible to CLI"
}

deploy_token_contracts() {
  cd "${SCRIPT_DIR}"/../public_token/

  TOKEN_A_ADDRESS=$($INSTANTIATE_CMD --args "${TOTAL_TOKEN_ISSUANCE_PER_CONTRACT}" --salt "0x$(random_salt)" | jq -r '.contract')
  export TOKEN_A_ADDRESS
  log_progress "Token A address: ${TOKEN_A_ADDRESS}"

  TOKEN_B_ADDRESS=$($INSTANTIATE_CMD --args "${TOTAL_TOKEN_ISSUANCE_PER_CONTRACT}" --salt "0x$(random_salt)" | jq -r '.contract')
  export TOKEN_B_ADDRESS
  log_progress "Token B address: ${TOKEN_B_ADDRESS}"
}

distribute_tokens() {
  cd "${SCRIPT_DIR}"/../public_token/

  for token in "${TOKEN_A_ADDRESS}" "${TOKEN_B_ADDRESS}"; do
    for recipient in "${DAMIAN_PUBKEY}" "${HANS_PUBKEY}"; do
      $CALL_CMD --contract "${token}" --message "PSP22::transfer" --args "${recipient}" "${TOKEN_PER_PERSON}" "0x00" --suri "${ADMIN}" 1> /dev/null 2> /dev/null
    done
  done
}

deploy_shielder_contract() {
  cd "${SCRIPT_DIR}"/../contract/
  SHIELDER_ADDRESS=$($INSTANTIATE_CMD --args "${MERKLE_LEAVES}" --salt "0x$(random_salt)" | jq -r '.contract')
  export SHIELDER_ADDRESS
  log_progress "Shielder address: ${SHIELDER_ADDRESS}"
}

set_allowances() {
  cd "${SCRIPT_DIR}"/../public_token/

  for token in "${TOKEN_A_ADDRESS}" "${TOKEN_B_ADDRESS}"; do
    for actor in "${DAMIAN}" "${HANS}"; do
      $CALL_CMD --contract "${token}" --message "PSP22::approve" --args "${SHIELDER_ADDRESS}" "${TOKEN_ALLOWANCE}" --suri "${actor}" 1> /dev/null 2> /dev/null
    done
  done
}

register_vk() {
  cd "${SCRIPT_DIR}"/../contract/

  DEPOSIT_VK_BYTES="0x$(xxd -ps <"${SCRIPT_DIR}"/docker/keys/deposit.groth16.vk.bytes | tr -d '\n')"
  WITHDRAW_VK_BYTES="0x$(xxd -ps <"${SCRIPT_DIR}"/docker/keys/withdraw.groth16.vk.bytes | tr -d '\n')"

  $CALL_CMD --contract "${SHIELDER_ADDRESS}" --message "register_vk" --args Deposit "${DEPOSIT_VK_BYTES}" --suri "${ADMIN}" 1> /dev/null 2> /dev/null
  $CALL_CMD --contract "${SHIELDER_ADDRESS}" --message "register_vk" --args Withdraw "${WITHDRAW_VK_BYTES}" --suri "${ADMIN}" 1> /dev/null 2> /dev/null
}

register_tokens() {
  cd "${SCRIPT_DIR}"/../contract/
  $CALL_CMD --contract "${SHIELDER_ADDRESS}" --message "register_new_token" --args 0 "${TOKEN_A_ADDRESS}" --suri "${ADMIN}" 1> /dev/null 2> /dev/null
  $CALL_CMD --contract "${SHIELDER_ADDRESS}" --message "register_new_token" --args 1 "${TOKEN_B_ADDRESS}" --suri "${ADMIN}" 1> /dev/null 2> /dev/null
}

setup_cli() {
  rm ~/.shielder-state 2>/dev/null || true

  cd "${SCRIPT_DIR}"/../cli/
  ./target/release/shielder-cli set-contract-address "${SHIELDER_ADDRESS}"

  log_progress "✅ Shielder CLI was set up"
}

deploy() {
  # general setup
  prepare_fs || error "Failed prepare file system"

  # launching node
  generate_chainspec || error "Failed to generate chainspec"
  export_bootnode_address || error "Failed to find out bootnode address"
  run_snarkeling_node || error "Failed to launch snarkeling node"

  # key generation
  generate_keys || error "Failed to generate keys"
  move_keys || error "Failed to move keys"

  # build contracts and CLI
  build || error "Failed to build contracts and CLI"
  move_build_artifacts || error "Failed to move build artifacts"

  # deploy and set up contracts
  deploy_token_contracts || error "Failed to deploy token contracts"
  distribute_tokens || error "Failed to distribute tokens"
  deploy_shielder_contract || error "Failed to deploy Shielder contract"
  set_allowances || error "Failed to set allowances"
  register_vk || error "Failed to register verifying keys"
  register_tokens || error "Failed to register token contracts"

  # setup CLI
  setup_cli || error "Failed to register token contracts"

  log_progress "🙌 Deployment successful"
}

deploy

# go back to the CLI directory (so that e.g. contract metadata is visible)
cd "$SCRIPT_DIR"/../cli
# enable `shielder_cli` shortcut for the current terminal session
alias shielder_cli="${SCRIPT_DIR}"/../cli/target/release/shielder-cli

set +euo pipefail
