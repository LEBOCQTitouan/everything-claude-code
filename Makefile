.PHONY: ci ci-dry ci-job build install dev

## Run all CI workflows locally via act
ci:
	act

## Dry-run: list all jobs without executing
ci-dry:
	act -n -l

## Run a specific job: make ci-job JOB=validate
ci-job:
	act -j $(JOB)

## Build release binary
build:
	cargo build --release

## Install ecc binary locally from source (for testing)
install: build
	cargo install --path crates/ecc-cli

## Build + install + run tests (full local dev cycle)
dev: install
	cargo test
	cargo clippy -- -D warnings
