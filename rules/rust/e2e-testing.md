---
paths:
  - "crates/ecc-integration-tests/**"
  - "crates/ecc-infra/**"
  - "crates/ecc-ports/**"
---

# E2E Testing Conventions (Rust)

## Boundary Mapping

E2E tests validate the full hexagonal stack through port-adapter boundaries:

| Layer | Crate | Role in E2E |
|-------|-------|-------------|
| Ports | `ecc-ports` | Trait definitions — the contracts under test |
| Infra | `ecc-infra` | Production adapters — what E2E tests exercise |
| Integration | `ecc-integration-tests` | Test harness — wires real adapters to real I/O |

## `#[ignore]` Pattern

Tests requiring external services (filesystem, network, processes) use `#[ignore]`:

```rust
#[test]
#[ignore] // Requires: filesystem access
fn test_real_filesystem_operations() {
    let fs = OsFileSystem::new();
    // ...
}
```

Run ignored tests explicitly:
```bash
cargo test -- --ignored           # all ignored tests
cargo test -- --ignored test_name # specific ignored test
```

## Environment Variable Gating

For tests that need specific environment setup:

```rust
#[test]
#[ignore]
fn test_with_env_dependency() {
    if std::env::var("ECC_E2E_ENABLED").is_err() {
        eprintln!("Skipping: set ECC_E2E_ENABLED=1 to run");
        return;
    }
    // ...
}
```

## Test Organization

- Place E2E tests in `crates/ecc-integration-tests/`
- Use descriptive module names matching the boundary being tested
- Group by port trait (e.g., `filesystem_tests`, `executor_tests`)
- Keep test helpers in `crates/ecc-test-support/`

## Conventions

1. **No mocks in E2E tests** — use real adapters from `ecc-infra`
2. **Temp directories** — use `tempfile::TempDir` for filesystem tests, clean up automatically
3. **Deterministic** — avoid tests that depend on timing or external state
4. **Fast feedback** — default `cargo test` runs unit tests only; `--ignored` opts into E2E
5. **Clear skip messages** — when a test skips, print why and what's needed to run it
