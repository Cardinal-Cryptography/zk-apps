#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)

# bump corresponding tag whenever a new version is released (updates should not be quite via `latest` tag)
export NODE_IMAGE=public.ecr.aws/p6e8q1z1/snarkeling:46c4726
export CLIAIN_IMAGE=public.ecr.aws/p6e8q1z1/cliain-snarkeling:8c5fe07
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

get_timestamp() {
  date +'%Y-%m-%d %H:%M:%S'
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
  generate_relation_keys "deposit-and-merge" "--max-path-len ${MERKLE_TREE_HEIGHT}"
  generate_relation_keys "withdraw" "--max-path-len ${MERKLE_TREE_HEIGHT}"
}

move_keys() {
  mv docker/keys/deposit.groth16.pk.bytes ../cli/deposit.pk.bytes
  mv docker/keys/deposit_and_merge.groth16.pk.bytes ../cli/deposit_and_merge.pk.bytes
  mv docker/keys/withdraw.groth16.pk.bytes ../cli/withdraw.pk.bytes

  log_progress "✅ Proving keys were made available to CLI"
}

docker_cargo() {
  docker run --rm \
    -u "${DOCKER_USER}" \
    -v "${PWD}":/code \
    -v ~/.cargo/git:/usr/local/cargo/git \
    -v ~/.cargo/registry:/usr/local/cargo/registry \
    --network host \
    --entrypoint /bin/sh \
    "${CARGO_IMAGE}" \
    -c "cargo ${1}"
}

build() {
  cd "${SCRIPT_DIR}"/..

  docker_cargo "contract build --release --manifest-path public_token/Cargo.toml 1>/dev/null 2>/dev/null"
  log_progress "✅ Public token contract was built"

  docker_cargo "contract build --release --manifest-path contract/Cargo.toml 1>/dev/null 2>/dev/null"
  log_progress "✅ Shielder contract was built"

  docker_cargo "build --release --manifest-path cli/Cargo.toml 1>/dev/null 2>/dev/null"
  log_progress "✅ CLI was built"
}

move_build_artifacts() {
  cp "${SCRIPT_DIR}"/../contract/target/ink/metadata.json "${SCRIPT_DIR}"/../cli/shielder-metadata.json
  log_progress "✅ Shielder metadata was made visible to CLI"
}

contract_instantiate() {
  docker_cargo "contract instantiate --skip-confirm --url ${NODE} --suri ${ADMIN} --output-json --salt 0x$(random_salt) ${1}"
}

contract_call() {
  docker_cargo "contract call --quiet --skip-confirm --url ${NODE} ${1}"
}

deploy_token_contracts() {
  cd "${SCRIPT_DIR}"/..

  TOKEN_A_ADDRESS=$(contract_instantiate "--args ${TOTAL_TOKEN_ISSUANCE_PER_CONTRACT} --manifest-path public_token/Cargo.toml" | jq -r '.contract')
  export TOKEN_A_ADDRESS
  log_progress "Token A address: ${TOKEN_A_ADDRESS}"

  TOKEN_B_ADDRESS=$(contract_instantiate "--args ${TOTAL_TOKEN_ISSUANCE_PER_CONTRACT} --manifest-path public_token/Cargo.toml"| jq -r '.contract')
  export TOKEN_B_ADDRESS
  log_progress "Token B address: ${TOKEN_B_ADDRESS}"
}

distribute_tokens() {
  cd "${SCRIPT_DIR}"/../public_token/

  for token in "${TOKEN_A_ADDRESS}" "${TOKEN_B_ADDRESS}"; do
    for recipient in "${DAMIAN_PUBKEY}" "${HANS_PUBKEY}"; do
      contract_call "--contract ${token} --message PSP22::transfer --args ${recipient} ${TOKEN_PER_PERSON} 0x00 --suri ${ADMIN}" 1> /dev/null 2> /dev/null
    done
  done
}

deploy_shielder_contract() {
  cd "${SCRIPT_DIR}"/..
  SHIELDER_ADDRESS=$(contract_instantiate "--args ${MERKLE_LEAVES} --manifest-path contract/Cargo.toml" | jq -r '.contract')
  export SHIELDER_ADDRESS
  log_progress "Shielder address: ${SHIELDER_ADDRESS}"
}

set_allowances() {
  cd "${SCRIPT_DIR}"/../public_token/

  for token in "${TOKEN_A_ADDRESS}" "${TOKEN_B_ADDRESS}"; do
    for actor in "${DAMIAN}" "${HANS}"; do
       contract_call "--contract ${token} --message PSP22::approve --args ${SHIELDER_ADDRESS} ${TOKEN_ALLOWANCE} --suri ${actor}" 1> /dev/null 2> /dev/null
    done
  done
}

register_vk() {
  cd "${SCRIPT_DIR}"/../contract/

  DEPOSIT_VK_BYTES="0x$(xxd -ps <"${SCRIPT_DIR}"/docker/keys/deposit.groth16.vk.bytes | tr -d '\n')"
  DEPOSIT_MERGE_VK_BYTES="0x$(xxd -ps <"${SCRIPT_DIR}"/docker/keys/deposit_and_merge.groth16.vk.bytes | tr -d '\n')"
  WITHDRAW_VK_BYTES="0x$(xxd -ps <"${SCRIPT_DIR}"/docker/keys/withdraw.groth16.vk.bytes | tr -d '\n')"

  contract_call "--contract  ${SHIELDER_ADDRESS} --message register_vk --args Deposit         ${DEPOSIT_VK_BYTES}       --suri ${ADMIN}" 1> /dev/null 2> /dev/null
  contract_call "--contract  ${SHIELDER_ADDRESS} --message register_vk --args DepositAndMerge ${DEPOSIT_MERGE_VK_BYTES} --suri ${ADMIN}" 1> /dev/null 2> /dev/null
  contract_call "--contract  ${SHIELDER_ADDRESS} --message register_vk --args Withdraw        ${WITHDRAW_VK_BYTES}      --suri ${ADMIN}" 1> /dev/null 2> /dev/null
}

register_tokens() {
  cd "${SCRIPT_DIR}"/../contract/
  contract_call "--contract ${SHIELDER_ADDRESS} --message register_new_token --args 0 ${TOKEN_A_ADDRESS} --suri ${ADMIN}" 1> /dev/null 2> /dev/null
  contract_call "--contract ${SHIELDER_ADDRESS} --message register_new_token --args 1 ${TOKEN_B_ADDRESS} --suri ${ADMIN}" 1> /dev/null 2> /dev/null
}

setup_cli() {
  rm ~/.shielder-state 2>/dev/null || true

  cd "${SCRIPT_DIR}"/../cli/
  ./target/release/shielder-cli set-contract-address "${SHIELDER_ADDRESS}"

  log_progress "✅ Shielder CLI was set up"
}

deploy() {
  # general setup
  prepare_fs

  # launching node
#  generate_chainspec
#  export_bootnode_address
#  run_snarkeling_node

  # key generation
  generate_keys
  move_keys

  # build contracts and CLI
  build
  move_build_artifacts

  # deploy and set up contracts
  deploy_token_contracts
  distribute_tokens
  deploy_shielder_contract
  set_allowances
  register_vk
  register_tokens

  # setup CLI
  setup_cli

  log_progress "🙌 Deployment successful"
}

deploy

# go back to the CLI directory (so that e.g. contract metadata is visible)
cd "$SCRIPT_DIR"/../cli
# enable `shielder_cli` shortcut for the current terminal session
alias shielder_cli="${SCRIPT_DIR}"/../cli/target/release/shielder-cli

set +euo pipefail
