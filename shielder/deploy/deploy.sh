#!/usr/bin/env bash

set -euo pipefail

# Check if run in e2e shielder test context. Defaults to unset.
E2E_TEST_CONTEXT=${E2E_TEST:-}

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)

export NODE_IMAGE="public.ecr.aws/p6e8q1z1/aleph-node-liminal:d93048e"
export CLIAIN_IMAGE="public.ecr.aws/p6e8q1z1/cliain-liminal:d93048e"
export INK_DEV_IMAGE="public.ecr.aws/p6e8q1z1/ink-dev:1.1.0"

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

generate_ink_types() {
  # ensure that we are in shielder/cli folder
  cd "${SCRIPT_DIR}"/../cli/
  docker_ink_dev "ink-wrapper -m shielder-metadata.json | rustfmt --edition 2021 > src/ink_contract.rs"

  log_progress "✅ Ink types were generated"
}

generate_chainspec() {
  CHAINSPEC_ARGS="\
    --base-path /data \
    --account-ids ${DAMIAN_PUBKEY} \
    --sudo-account-id ${ADMIN_PUBKEY} \
    --faucet-account-id ${ADMIN_PUBKEY} \
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

transfer() {
  $DOCKER_SH \
    --network host \
    ${CLIAIN_IMAGE} \
    -c "/usr/local/bin/cliain --node ${NODE} --seed ${ADMIN} transfer --amount-in-tokens ${TOKEN_PER_PERSON} --to-account ${1}" 1>/dev/null

  log_progress "✅ Transferred ${TOKEN_PER_PERSON} to ${1}"
}

generate_keys() {
  generate_relation_keys "deposit"
  generate_relation_keys "deposit-and-merge" "--max-path-len ${MERKLE_TREE_HEIGHT}"
  generate_relation_keys "merge" "--max-path-len ${MERKLE_TREE_HEIGHT}"
  generate_relation_keys "withdraw" "--max-path-len ${MERKLE_TREE_HEIGHT}"
}

move_keys() {
  mv docker/keys/deposit.groth16.pk.bytes ../cli/deposit.pk.bytes
  mv docker/keys/deposit_and_merge.groth16.pk.bytes ../cli/deposit_and_merge.pk.bytes
  mv docker/keys/merge.groth16.pk.bytes ../cli/merge.pk.bytes
  mv docker/keys/withdraw.groth16.pk.bytes ../cli/withdraw.pk.bytes

  log_progress "✅ Proving keys were made available to CLI"
}

docker_ink_dev() {
  docker run --rm \
    -u "${DOCKER_USER}" \
    -v "${PWD}":/code \
    -v ~/.cargo/git:/usr/local/cargo/git \
    -v ~/.cargo/registry:/usr/local/cargo/registry \
    --network host \
    --entrypoint /bin/sh \
    "${INK_DEV_IMAGE}" \
    -c "${1}"
}

build() {
  cd "${SCRIPT_DIR}"/..

  docker_ink_dev "cargo contract build --release --manifest-path public_token/Cargo.toml 1>/dev/null"
  log_progress "✅ Public token contract was built"

  docker_ink_dev "cargo contract build --release --manifest-path contract/Cargo.toml 1>/dev/null"
  log_progress "✅ Shielder contract was built"
}

move_build_artifacts() {
  cp "${SCRIPT_DIR}"/../contract/target/ink/shielder.json "${SCRIPT_DIR}"/../cli/shielder-metadata.json
  log_progress "✅ Shielder metadata was made visible to CLI"
}

contract_instantiate() {
  docker_ink_dev "cargo contract instantiate --skip-confirm --url ${NODE} --suri ${ADMIN} --output-json --salt 0x$(random_salt) ${1}"
}

contract_call() {
  docker_ink_dev "cargo contract call --quiet --skip-confirm --url ${NODE} ${1}"
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

# Funds Damian and Hans accounts from faucet
prefund_users() {
  for recipient in "${DAMIAN_PUBKEY}" "${HANS_PUBKEY}"; do
    transfer ${recipient}
  done
}

# Distribute TOKEN_PER_PERSON of TOKEN_A and TOKEN_B to DAMIAN and HANS.
distribute_tokens() {
  cd "${SCRIPT_DIR}"/../public_token/

  for token in "${TOKEN_A_ADDRESS}" "${TOKEN_B_ADDRESS}"; do
    for recipient in "${DAMIAN_PUBKEY}" "${HANS_PUBKEY}"; do
      contract_call "--contract ${token} --message PSP22::transfer --args ${recipient} ${TOKEN_PER_PERSON} 0x00 --suri ${ADMIN}" 1>/dev/null
    done
  done
}

deploy_shielder_contract() {
  cd "${SCRIPT_DIR}"/..
  SHIELDER_ADDRESS=$(contract_instantiate "--args ${MERKLE_LEAVES} --manifest-path contract/Cargo.toml" | jq -r '.contract')
  export SHIELDER_ADDRESS
  log_progress "Shielder address: ${SHIELDER_ADDRESS}"
}

# Set allowance at TOKEN_ALLOWANCE on TOKEN_A and TOKEN_B from SHIELDER, from DAMIAN and HANS.
# I.E. Shielder contract can now transfer up to TOKEN_ALLOWANCE of tokens from DAMIAN and HANS' accounts.
set_allowances() {
  cd "${SCRIPT_DIR}"/../public_token/

  for token in "${TOKEN_A_ADDRESS}" "${TOKEN_B_ADDRESS}"; do
    for actor in "${DAMIAN}" "${HANS}"; do
       contract_call "--contract ${token} --message PSP22::approve --args ${SHIELDER_ADDRESS} ${TOKEN_ALLOWANCE} --suri ${actor}" 1>/dev/null
    done
  done

  log_progress "✅ Allowances set."
}

store_contract_addresses() {
  jq -n --arg shielder_address "$SHIELDER_ADDRESS" \
        --arg token_a_address "$TOKEN_A_ADDRESS" \
        --arg token_b_address "$TOKEN_B_ADDRESS" \
        '{
          shielder_address: $shielder_address,
          token_a_address: $token_a_address,
          token_b_address: $token_b_address,
        }' > ${SCRIPT_DIR}/addresses.json

  log_progress "✅ Contract addresses stored in a ${SCRIPT_DIR}/addresses.json"
}

register_vk() {
  DEPOSIT_VK_BYTES="0x$(xxd -ps <"${SCRIPT_DIR}"/docker/keys/deposit.groth16.vk.bytes | tr -d '\n')"
  DEPOSIT_MERGE_VK_BYTES="0x$(xxd -ps <"${SCRIPT_DIR}"/docker/keys/deposit_and_merge.groth16.vk.bytes | tr -d '\n')"
  MERGE_VK_BYTES="0x$(xxd -ps <"${SCRIPT_DIR}"/docker/keys/merge.groth16.vk.bytes | tr -d '\n')"
  WITHDRAW_VK_BYTES="0x$(xxd -ps <"${SCRIPT_DIR}"/docker/keys/withdraw.groth16.vk.bytes | tr -d '\n')"

  pushd $SCRIPT_DIR/../contract

  contract_call "--contract  ${SHIELDER_ADDRESS} --message register_vk --args Deposit         ${DEPOSIT_VK_BYTES}       --suri ${ADMIN}" 1>/dev/null
  contract_call "--contract  ${SHIELDER_ADDRESS} --message register_vk --args DepositAndMerge ${DEPOSIT_MERGE_VK_BYTES} --suri ${ADMIN}" 1>/dev/null
  contract_call "--contract  ${SHIELDER_ADDRESS} --message register_vk --args Merge           ${MERGE_VK_BYTES}         --suri ${ADMIN}" 1>/dev/null
  contract_call "--contract  ${SHIELDER_ADDRESS} --message register_vk --args Withdraw        ${WITHDRAW_VK_BYTES}      --suri ${ADMIN}" 1>/dev/null

  popd
}

register_tokens() {
  cd "${SCRIPT_DIR}"/../contract/
  contract_call "--contract ${SHIELDER_ADDRESS} --message register_new_token --args 0 ${TOKEN_A_ADDRESS} --suri ${ADMIN}" 1>/dev/null
  contract_call "--contract ${SHIELDER_ADDRESS} --message register_new_token --args 1 ${TOKEN_B_ADDRESS} --suri ${ADMIN}" 1>/dev/null
}

setup_cli() {
  cd "${SCRIPT_DIR}"/..
  docker_ink_dev "cargo build --release --manifest-path cli/Cargo.toml 1>/dev/null"
  log_progress "✅ CLI was built"

  rm ~/.shielder-state 2>/dev/null || true

  cd "${SCRIPT_DIR}"/../cli/
  ./target/release/shielder-cli set-contract-address "${SHIELDER_ADDRESS}"

  log_progress "✅ Shielder CLI was set up"
}

deploy() {
  # general setup
  prepare_fs

  # launching node
  generate_chainspec
  export_bootnode_address
  run_snarkeling_node

  # key generation
  generate_keys
  move_keys

  # build contracts
  build
  move_build_artifacts

  # use ink-wrapper to generate ink types based on current contract metadata
  generate_ink_types

  prefund_users

  # deploy and set up contracts
  deploy_token_contracts
  distribute_tokens
  deploy_shielder_contract
  set_allowances
  register_vk
  register_tokens

  # store contract addresses in a file
  store_contract_addresses

  # build and setup CLI
  if [[ -z "${E2E_TEST_CONTEXT}" ]]; then
    log_progress "Setting up CLI..."
    setup_cli
  else
    log_progress "Running in e2e test context. Skipping CLI setup."
  fi

  log_progress "🙌 Deployment successful"
}

deploy

# go back to the CLI directory (so that e.g. contract metadata is visible)
cd "$SCRIPT_DIR"/../cli
# enable `shielder_cli` shortcut for the current terminal session
alias shielder_cli="${SCRIPT_DIR}"/../cli/target/release/shielder-cli

set +euo pipefail
