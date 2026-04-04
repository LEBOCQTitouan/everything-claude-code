---
name: hexagonal
category: architecture
tags: [architecture, ports-and-adapters, ddd, hexagonal]
languages: [all]
difficulty: advanced
---

## Intent

Isolate the application core (business logic) from external systems by placing all I/O behind explicit port interfaces. Adapters translate between the external world and the ports, keeping the domain free of infrastructure concerns.

## Problem

Business logic becomes entangled with database drivers, HTTP frameworks, and CLI parsers. Changing the database requires touching domain code. Writing unit tests requires spinning up real infrastructure. The application cannot be run in multiple modes (HTTP, CLI, test) without duplication.

## Solution

Define the application core using port interfaces (traits/interfaces) for every external interaction. Provide adapter implementations for each deployment context. The domain depends only on port abstractions; adapters depend on the domain, never the reverse.

```
┌────────────────────────────────────────────────┐
│               Driving Adapters                 │
│  (CLI, REST, gRPC, test harness)               │
│               │  calls                         │
│      ┌────────▼───────────┐                   │
│      │   Driving Ports    │  ← use-case        │
│      │   (in-ports)       │    interfaces      │
│      │                    │                    │
│      │   Domain Core      │                    │
│      │                    │                    │
│      │   Driven Ports     │  ← repository,     │
│      │   (out-ports)      │    event bus       │
│      └────────┬───────────┘                   │
│               │  implemented by                │
│               ▼                                │
│           Driven Adapters                      │
│  (Postgres, S3, SMTP, in-memory)               │
└────────────────────────────────────────────────┘
```

## Language Implementations

### all

ECC implements hexagonal architecture across its crate workspace:

- `ecc-domain` — pure business logic; zero I/O imports; contains aggregates, value objects, domain services.
- `ecc-ports` — port trait definitions (`FileSystem`, `ShellExecutor`, `Environment`, `TerminalIO`); no implementations.
- `ecc-app` — orchestration layer; depends only on `ecc-domain` and `ecc-ports`; implements driving-port use cases.
- `ecc-infra` — driven adapters (`OsFileSystem`, `OsExecutor`); depends on `ecc-ports`, never on `ecc-domain` directly.
- `ecc-cli` — driving adapter; wires infra adapters into app use cases; thin entry point.
- `ecc-test-support` — in-memory adapters (`InMemoryFileSystem`, `MockExecutor`) for fast, isolated unit tests.

Key invariant: `ecc-domain` must compile with zero dependencies on any adapter crate.

## When to Use

- When the business logic is complex enough to warrant isolation from infrastructure.
- When the application must support multiple deployment modes (HTTP, CLI, lambda, tests).
- When you want to test the domain without spinning up databases or external services.

## When NOT to Use

- For simple CRUD scripts where the domain is trivial.
- When the team is not yet familiar with the pattern — the indirection adds cognitive overhead.

## Anti-Patterns

- Importing framework types (Axum extractors, Diesel models) inside the domain core — this breaks the isolation.
- Leaking ORM entities through driven ports — ports must return domain types only.
- Placing business rules in adapters — adapters translate, they do not decide.

## Related Patterns

- [clean-architecture](clean-architecture.md) — a similar layering model with explicit concentric rings.
- [cqrs](cqrs.md) — often combined with hexagonal architecture for read/write separation.

## References

- Alistair Cockburn — Hexagonal Architecture: https://alistair.cockburn.us/hexagonal-architecture/
- ECC Architecture: docs/ARCHITECTURE.md
- ECC Bounded Contexts: docs/domain/bounded-contexts.md
