.DEFAULT_GOAL := help

.PHONY: help
help: # Show help for each of the Makefile recipes.
	@grep -E '^[a-zA-Z0-9 -]+:.*#'  Makefile | sort | while read -r l; do printf "\033[1;32m$$(echo $$l | cut -f 1 -d':')\033[00m:$$(echo $$l | cut -f 2- -d'#')\n"; done

.PHONY: check
check: ## Run cargo checks
	cargo clippy --all-targets -- -D warnings
	cargo fmt --all --check

.PHONY: build-psp22
build-psp22: ## Builds psp22 contracts.
	@echo "Building psp22 contract" ; \
	git submodule update --init ; \
	cargo contract build --quiet --manifest-path PSP22/Cargo.toml --features "contract" --release ; \

.PHONY: build-shielder
build-shielder: ## Builds shielder contracts.
	@echo "Building shielder contract" ; \
	cargo contract build --manifest-path contract/Cargo.toml --release ; \

.PHONY: setup-tests
setup-tests: build-psp22 build-shielder ## Builds contracts and generates wrappers.

.PHONY: shielder-unit-tests
shielder-unit-tests: ## Runs unit tests for contract.
	@echo "Running shielder unit tests" ; \
	cargo test --quiet --manifest-path contract/Cargo.toml --release -- --nocapture ; \

.PHONY: mocked-zk-unit-tests
mocked-zk-unit-tests: ## Runs unit tests for contract.
	@echo "Running mocked_zk unit tests" ; \
	cargo test --quiet --manifest-path mocked_zk/Cargo.toml --release -- --nocapture ; \

.PHONY: shielder-drink-tests
shielder-drink-tests: setup-tests ## Runs drink tests for contract.
	@echo "Running shielder drink tests" ; \
	cargo test --quiet --manifest-path drink_tests/Cargo.toml --release -- --nocapture ; \

.PHONY: tests
tests: shielder-unit-tests mocked-zk-unit-tests shielder-drink-tests ## Runs tests for contract.

.PHONY:
all: check tests
