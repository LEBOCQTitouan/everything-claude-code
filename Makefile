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

## Install ecc binary + hook shims locally from source
install: build
	cargo install --path crates/ecc-cli
	@INSTALL_DIR="$$(dirname "$$(which ecc 2>/dev/null || echo $$HOME/.cargo/bin/ecc)")" && \
		cp bin/ecc-hook "$$INSTALL_DIR/" && \
		cp bin/ecc-shell-hook.sh "$$INSTALL_DIR/" && \
		chmod +x "$$INSTALL_DIR/ecc-hook" && \
		chmod +x "$$INSTALL_DIR/ecc-shell-hook.sh" && \
		echo "Installed ecc-hook and ecc-shell-hook.sh to $$INSTALL_DIR"
	@# Symlink asset dirs to ~/.ecc/ so resolve_ecc_root() finds them
	@ECC_HOME="$$HOME/.ecc" && \
		mkdir -p "$$ECC_HOME" && \
		REPO_ROOT="$$(cd "$$(dirname "$(MAKEFILE_LIST)")" && pwd)" && \
		for dir in agents commands skills rules hooks contexts mcp-configs schemas examples; do \
			if [ -d "$$REPO_ROOT/$$dir" ]; then \
				rm -f "$$ECC_HOME/$$dir" && \
				ln -sf "$$REPO_ROOT/$$dir" "$$ECC_HOME/$$dir"; \
			fi; \
		done && \
		if [ -f "$$REPO_ROOT/hooks/hooks.json" ]; then \
			rm -f "$$ECC_HOME/hooks.json" && \
			ln -sf "$$REPO_ROOT/hooks/hooks.json" "$$ECC_HOME/hooks.json"; \
		fi && \
		echo "Symlinked asset directories to $$ECC_HOME"

## Build + install + run tests (full local dev cycle)
dev: install
	cargo test
	cargo clippy -- -D warnings
