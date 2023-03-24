.PHONY: help test-shielder test-shielder-clean setup-test-network setup-resources


setup-test-network: ## Setups local docker network and deploys shielder and its dependencies
	@export E2E_TEST=1 && ./deploy/deploy.sh

setup-resources: ## Setups resources required for running integration test for shielder
	@./cli/tests/setup_local.sh

test-shielder: setup-resources ## Setups resources and runs integration tests against already-running network.
	@cd cli && cargo test --release ${TEST_CASES} -- --show-output --ignored

test-shielder-clean: setup-test-network test-shielder ## Restarts docker network and runs Shielder tests.

help: ## Displays this help
	@awk 'BEGIN {FS = ":.*##"; printf "$(MAKEFILE_NAME)\n\nUsage:\n  make \033[1;36m<target>\033[0m\n\nTargets:\n"} /^[a-zA-Z0-9_-]+:.*?##/ { printf "  \033[1;36m%-25s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)
