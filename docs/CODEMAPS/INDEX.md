<!-- Generated: 2026-03-15 | Crates: 6 | Files: 109 .rs -->

# Codemap Index -- Everything Claude Code (ECC)

## Maps

| File | Description |
|------|-------------|
| [architecture.md](architecture.md) | Hexagonal architecture, data flow, crate boundaries, build pipeline |
| [backend.md](backend.md) | Crate-level module breakdown, hook handlers, CLI commands |
| [data.md](data.md) | Rust data structures, storage format, configuration files |
| [dependencies.md](dependencies.md) | Cargo workspace deps, external tools, GitHub Releases distribution |

## Quick Stats

- **Source:** 109 Rust files across 6 crates (~23,800 LOC)
- **Tests:** 996 tests passing (514 domain + 464 app + 18 others)
- **Content:** 30 agents, 7 commands, 81 skills, 7 rule groups
- **Runtime deps:** serde, clap, regex, crossterm, rustyline, walkdir
- **Build:** `cargo build --release` (single binary `ecc`)

## Entry Points

```
crates/ecc-cli/src/main.rs  → CLI binary (`ecc` command via clap)
scripts/get-ecc.sh          → curl installer (GitHub Releases → ~/.ecc/)
```

## Crate Architecture

```
ecc-cli        Binary entry, clap argument parsing, subcommand dispatch
  └─ ecc-app        Use case orchestration (install, merge, audit, validate, hook, claw)
       ├─ ecc-domain    Pure business logic, zero I/O (config, detection, diff, session)
       └─ ecc-ports     Trait definitions (FileSystem, ShellExecutor, Environment, TerminalIO)
            ├─ ecc-infra         Production OS adapters (std::fs, std::process, crossterm)
            └─ ecc-test-support  Test doubles (InMemoryFileSystem, MockExecutor, etc.)
```

## Agent Ecosystem

```
Orchestrators:  doc-orchestrator, arch-reviewer, audit-orchestrator
Reviewers:      code-reviewer, python-reviewer, go-reviewer, security-reviewer, database-reviewer, uncle-bob
Architects:     architect, architect-module
Builders:       build-error-resolver, go-build-resolver, tdd-guide, e2e-runner
Doc system:     doc-analyzer, doc-generator, doc-validator, doc-reporter, diagram-generator
Audit system:   evolution-analyst, test-auditor, observability-auditor, error-handling-auditor, convention-auditor
Utilities:      planner, requirements-analyst, refactor-cleaner, harness-optimizer, doc-updater
```
