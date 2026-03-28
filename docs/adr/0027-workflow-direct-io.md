# ADR 0027: ecc-workflow Keeps Direct I/O

## Status

Accepted (2026-03-28)

## Context

The main ECC binary (`ecc-cli`) follows hexagonal architecture with all I/O abstracted behind port traits (`FileSystem`, `ShellExecutor`, etc.). The `ecc-workflow` binary is a standalone tool for workflow state management, invoked by hooks during Claude Code sessions.

The full audit (2026-03-28) identified that ecc-workflow has 60+ direct `std::fs` calls bypassing the port system, making it untestable with in-memory test doubles.

Two approaches were evaluated:
1. **Full port integration**: Add ecc-ports dependency, thread `&dyn FileSystem` through all command handlers
2. **Functional Core, Imperative Shell**: Extract pure logic into testable functions, keep direct I/O in thin command shells

## Decision

We chose option 2: **Functional Core, Imperative Shell**. ecc-workflow keeps direct I/O (`std::fs`, `std::io`) in its command handlers. Pure business logic (content formatting, slug generation, JSON construction) is extracted into unit-testable functions.

## Consequences

- Pure functions are fully unit-testable without filesystem
- Integration tests continue to spawn the binary as a subprocess
- ecc-workflow remains a lightweight standalone binary without the port system overhead
- Future: if test coverage proves insufficient via subprocess tests, reconsider port integration
