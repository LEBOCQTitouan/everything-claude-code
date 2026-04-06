# Crates — Hexagonal Architecture

## Dependency Direction (strict)

```
ecc-cli → ecc-app → ecc-ports ← ecc-infra
                  → ecc-domain
```

- `ecc-domain`: Pure business logic. Zero `std::fs`, `std::process`, `std::net`, or `tokio` imports. If you need I/O, define a port trait in `ecc-ports` instead.
- `ecc-ports`: Trait definitions only (`FileSystem`, `ShellExecutor`, `Environment`, `TerminalIO`, `GitInfo`, `WorktreeManager`, `CostStore`). No implementations.
- `ecc-app`: Orchestrates domain + ports. Depends on traits, never on concrete adapters.
- `ecc-infra`: Concrete adapters implementing port traits. Only crate that touches the OS.
- `ecc-cli`: Wires infra adapters into app use cases. Thin — no business logic here.
- `ecc-test-support`: In-memory test doubles (`InMemoryFileSystem`, `MockExecutor`, `MockEnvironment`, `MockWorktreeManager`). Used by unit and integration tests.

## Gotchas

- Adding a new I/O capability? Create a port trait in `ecc-ports` first, then implement in `ecc-infra`.
- `ecc-domain` depending on `ecc-infra` (or any adapter) is a build-breaking architecture violation.
- All tests use `ecc-test-support` doubles — never construct real filesystem/executor in tests.
- Integration tests live in `ecc-integration-tests`, not inside individual crates.
