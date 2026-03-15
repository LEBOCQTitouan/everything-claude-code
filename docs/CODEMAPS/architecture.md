<!-- Generated: 2026-03-15 | Crates: 6 | Files: 109 .rs -->

# Architecture Overview

## System Type

CLI tool (`ecc`) -- Rust binary distributed via GitHub Releases with curl installer, providing Claude Code configuration management.

## Hexagonal Architecture

```
                    ┌───────────────────────────┐
                    │        ecc-cli            │
                    │  (clap args, dispatch)     │
                    └─────────┬─────────────────┘
                              │
                    ┌─────────▼─────────────────┐
                    │        ecc-app            │
                    │  (use cases / orchestration)│
                    │  install, merge, audit,    │
                    │  validate, hook, claw      │
                    └──┬──────────────────┬──────┘
                       │                  │
            ┌──────────▼──────┐  ┌────────▼────────┐
            │   ecc-domain   │  │   ecc-ports     │
            │  (pure logic)  │  │  (trait defs)   │
            │  zero I/O      │  │  boundaries     │
            └────────────────┘  └──┬──────────┬───┘
                                   │          │
                        ┌──────────▼──┐ ┌─────▼──────────┐
                        │ ecc-infra  │ │ ecc-test-support│
                        │ (OS adapt.)│ │ (test doubles)  │
                        └────────────┘ └─────────────────┘
```

## Data Flow

```
User CLI
  │
  ├─ ecc install → InstallContext
  │    ├─ detect::detect_and_report  → scan existing setup
  │    ├─ manifest::read_manifest    → track ECC artifacts
  │    ├─ merge::merge_directory     → interactive diff review
  │    ├─ merge::merge_hooks         → hook merge with legacy removal
  │    ├─ deny_rules::ensure_deny_rules → security deny rules
  │    └─ manifest::write_manifest   → persist updated manifest
  │
  ├─ ecc audit → AuditOptions
  │    └─ config::audit::run_all_checks → score + grade
  │
  ├─ ecc validate <target> → ValidateTarget
  │    └─ validate::{agents,commands,hooks,skills,rules,paths}
  │
  ├─ ecc hook <id> [profiles] → HookContext
  │    └─ hook::dispatch → 20+ hook handlers (passthrough/warn/block)
  │
  ├─ ecc init → init_project (gitignore + untrack)
  │
  └─ ecc claw → ClawConfig → run_repl
       └─ REPL loop: parse_command → dispatch_command → claude -p
```

## Key Boundaries

| Boundary | Description |
|----------|-------------|
| domain ↔ ports | Domain types are pure; all I/O goes through port traits |
| app ↔ ports | Use cases accept `&dyn Trait` references, never concrete types |
| infra → ports | Production adapters implement port traits against OS primitives |
| test-support → ports | Test doubles implement port traits with in-memory state |
| cli → app | CLI parses args, constructs contexts, delegates to use cases |

## Build Pipeline

```
crates/**/*.rs  →  cargo build --release  →  target/release/ecc (single binary)
                                                  │
                                           GitHub Release tarballs (binary + content)
                                                  │
                                           curl installer → ~/.ecc/
```

## Test Architecture

```
cargo test                → 999 tests across all crates
  ├─ ecc-domain (515)    → pure unit tests, proptest property tests
  ├─ ecc-app (466)       → use case tests with InMemoryFileSystem + MockExecutor
  ├─ ecc-cli (13)        → CLI integration tests
  ├─ ecc-ports (3)       → trait contract tests (3 ignored — require OS)
  ├─ ecc-infra (2)       → adapter integration tests
  └─ ecc-test-support (0) → test double self-tests
```

## Runtime Dependencies

- `serde` + `serde_json` -- serialization
- `clap` + `clap_complete` -- CLI parsing and shell completions
- `regex` -- pattern matching
- `crossterm` -- terminal control
- `rustyline` -- REPL line editing with history
- `walkdir` -- recursive directory traversal
- `thiserror` / `anyhow` -- error handling
- Optional: `claude` CLI (for smart-merge)
