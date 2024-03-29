.DEFAULT_GOAL := help

.PHONY: help
help: # Show help for each of the Makefile recipes.
	@grep -E '^[a-zA-Z0-9 -]+:.*#'  Makefile | sort | while read -r l; do printf "\033[1;32m$$(echo $$l | cut -f 1 -d':')\033[00m:$$(echo $$l | cut -f 2- -d'#')\n"; done

.PHONY: check
check: ## Run cargo checks
	@cargo clippy --all-targets -- -D warnings
	@cargo clippy --manifest-path contract/Cargo.toml -- -D warnings
	@cargo fmt --all --check
	@cargo fmt --all --check --manifest-path contract/Cargo.toml

.PHONY: build-psp22
build-psp22: ## Builds psp22 contracts.
	@echo "Building psp22 contract" ; \
	git submodule update --init ; \
	cargo contract build --manifest-path PSP22/Cargo.toml --features "contract" --release ; \

.PHONY: build-shielder
build-shielder: ## Builds shielder contracts.
	@echo "Building shielder contract" ; \
	cargo contract build --manifest-path contract/Cargo.toml --release ; \

.PHONY: setup-tests
setup-tests: build-psp22 build-shielder ## Builds contracts and generates wrappers.

.PHONY: shielder-tests
shielder-tests: ## Runs tests for contract.
	@echo "Running shielder tests" ; \
	cargo test --manifest-path contract/Cargo.toml --release -- --nocapture ; \

.PHONY: mocked-zk-unit-tests
mocked-zk-unit-tests: ## Runs unit tests for contract.
	@echo "Running mocked_zk unit tests" ; \
	cargo test --manifest-path mocked_zk/Cargo.toml --release -- --nocapture ; \

.PHONY: tests
tests: shielder-tests mocked-zk-unit-tests ## Runs tests for contract.

.PHONY:
all: check tests
