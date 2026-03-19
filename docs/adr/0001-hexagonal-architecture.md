# ADR-0001: Hexagonal Architecture with 6 Crates

## Status

Accepted

## Context

ECC is a CLI tool that manages content installation, hook execution, configuration auditing, and project initialization. These concerns require clear separation between business logic, I/O abstractions, and infrastructure adapters to enable:

- Full in-memory testing without touching the filesystem or spawning processes
- Swappable adapters (e.g., test doubles vs real OS implementations)
- A pure domain layer with zero I/O imports

## Decision

Organize the Rust workspace into 6 crates following hexagonal architecture:

| Crate | Layer | Responsibility |
|-------|-------|----------------|
| `ecc-domain` | Domain | Pure business logic — zero I/O |
| `ecc-ports` | Ports | Trait definitions (FileSystem, ShellExecutor, Environment, TerminalIO) |
| `ecc-app` | Application | Use cases — orchestrates domain + ports |
| `ecc-infra` | Infrastructure | Production adapters (OS filesystem, process executor, terminal) |
| `ecc-cli` | Presentation | CLI binary entry point |
| `ecc-test-support` | Testing | Test doubles (InMemoryFileSystem, MockExecutor, MockEnvironment) |

Dependency direction flows inward: CLI -> App -> Domain, with Ports defining the boundaries and Infra/TestSupport providing implementations.

## Consequences

- **Easier**: Testing is fast and deterministic — 999 tests run in-memory without OS dependencies
- **Easier**: Adding new adapters (e.g., a future GUI or web interface) requires only a new presentation crate
- **Harder**: More crates to maintain; simple changes may touch multiple crates
- **Enforced**: `ecc-domain` must have zero I/O imports — validated by a boundary-crossing hook

See also: [Architecture](../ARCHITECTURE.md) for the full system diagram.
