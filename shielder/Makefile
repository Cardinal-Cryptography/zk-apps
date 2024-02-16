.DEFAULT_GOAL := help

.PHONY: help
help: # Show help for each of the Makefile recipes.
	@grep -E '^[a-zA-Z0-9 -]+:.*#'  Makefile | sort | while read -r l; do printf "\033[1;32m$$(echo $$l | cut -f 1 -d':')\033[00m:$$(echo $$l | cut -f 2- -d'#')\n"; done

.PHONY: check-shielder
check-shielder: ## Run cargo checks on shielder contract.
	@echo "Checking shielder contract" ; \
	cargo clippy --manifest-path contract/Cargo.toml -- -D warnings ; \
	cargo fmt --quiet --all --check --manifest-path contract/Cargo.toml ; \


.PHONY: build-psp22
build-psp22: ## Builds psp22 contracts.
	@echo "Building psp22 contract" ; \
	git submodule update --init ; \
	cargo contract build --quiet --manifest-path PSP22/Cargo.toml --features "contract" --release ; \

.PHONY: build-shielder
build-shielder: ## Builds shielder contracts.
	@echo "Building shielder contract" ; \
	cargo contract build --quiet --manifest-path contract/Cargo.toml --release ; \

.PHONY: setup-tests
setup-tests: build-psp22 build-shielder ## Builds contracts and generates wrappers.

.PHONY: tests
tests: setup-tests ## Runs tests for contract.
	@echo "Running drink tests" ; \
	cargo test --quiet --manifest-path contract/Cargo.toml --release -- --nocapture ; \

.PHONY:
all: check-shielder tests
