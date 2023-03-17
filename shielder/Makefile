.PHONY: help test setup-test-network setup-resources


setup-test-network: ## Setups local docker network and deploys shielder and its dependencies
	@./deploy/deploy.sh

setup-resources: ## Setups resources required for running integration test for shielder
	@./cli/tests/setup_local.sh

test-shielder: setup-resources ## Setups resources and runs integration tests against already-running network.
	@export E2E_TEST=1 && cd cli && cargo test --release -- --nocapture --test-threads 3

test-shielder-clean: setup-test-network test-shielder ## Restarts docker network and runs Shielder tests.

help: ## Displays this help
	@awk 'BEGIN {FS = ":.*##"; printf "$(MAKEFILE_NAME)\n\nUsage:\n  make \033[1;36m<target>\033[0m\n\nTargets:\n"} /^[a-zA-Z0-9_-]+:.*?##/ { printf "  \033[1;36m%-25s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)