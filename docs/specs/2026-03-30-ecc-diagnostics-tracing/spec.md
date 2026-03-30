# Spec: ECC diagnostics — tiered verbosity with tracing (BL-091)

## Problem Statement

ECC hooks fire silently — users can't see what's happening under the hood. The current `log`+`env_logger` setup provides only warn-level output with a binary --verbose toggle. There's no way to see workflow state, component counts, or binary versions without manually inspecting files. No persistent configuration for verbosity exists. This makes debugging hook failures, onboarding new users, and understanding ECC's runtime behavior unnecessarily difficult — all while zero model cost is achievable since diagnostics go to stderr.

## Research Summary

- `tracing` is the modern Rust standard for structured diagnostics, superseding the `log` crate
- `tracing-subscriber` with `EnvFilter` provides per-module filtering via env vars (`ECC_LOG=ecc_domain=trace`)
- `tracing` maintains backward compatibility with the `log` crate via `tracing-log` bridge (existing `log::warn!` calls work during migration)
- Config precedence pattern (CLI flag > env var > config file > default) is standard across Rust CLI tools
- BL-091 backlog research confirmed: separate `ECC_LOG` env var avoids conflicts with other Rust tooling using `RUST_LOG`

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Replace `log`+`env_logger` with `tracing`+`tracing-subscriber` | Modern Rust standard, structured, per-module filtering | Yes |
| 2 | 5-level tiered verbosity: -q/default/-v/-vv/-vvv | Granular control from errors-only to full trace | Yes |
| 3 | `ECC_LOG` env var with `RUST_LOG` fallback + deprecation warning | Clean ECC-specific namespace, backward compat during transition | Yes |
| 4 | ecc-workflow silent by default (tracing only when ECC_LOG explicitly set) | Protect JSON hook output from stderr pollution | Yes |
| 5 | `~/.ecc/config.toml` for persistent preferences | No config persistence exists — toml is standard for Rust CLIs | Yes |
| 6 | All 3 sub-features together | Cohesive: config needed for tracing persistence, status leverages tracing state | No |
| 7 | ecc-domain stays pure (no tracing dep) | Hexagonal architecture constraint — use tracing-log bridge in app/infra | Yes |
| 8 | Config keys are whitelisted: only `log-level` initially | Prevents unbounded key-value store; new keys added explicitly per feature | No |

## User Stories

### US-001: Tiered verbosity via tracing

**As a** developer using ECC, **I want** configurable diagnostic verbosity from errors-only to full trace, **so that** I can debug hook failures and understand ECC's runtime behavior without wasting model tokens.

#### Acceptance Criteria

- AC-001.1: Given ecc-cli, when `tracing` and `tracing-subscriber` are added as dependencies, then `env_logger` is removed from ecc-cli Cargo.toml
- AC-001.2: Given ecc-cli main.rs, when the binary starts, then tracing-subscriber is initialized with EnvFilter reading ECC_LOG (fallback to RUST_LOG with deprecation warning on stderr)
- AC-001.3: Given `-q` flag, when ecc runs, then only error-level messages appear on stderr
- AC-001.4: Given no flags (default), when ecc runs, then warn-level messages appear (current behavior preserved)
- AC-001.5: Given `-v` flag, when ecc runs, then info-level messages appear
- AC-001.6: Given `-vv` flag, when ecc runs, then debug-level messages appear
- AC-001.7: Given `-vvv` flag, when ecc runs, then trace-level messages appear
- AC-001.8: Given ecc-workflow binary, when ECC_LOG is not set, then zero tracing output appears on stderr (silent default)
- AC-001.9: Given ecc-workflow binary, when `ECC_LOG=info` is set, then info-level messages appear on stderr
- AC-001.10: Given both `ECC_LOG` and `RUST_LOG` are set, when ecc runs, then `ECC_LOG` takes precedence and a deprecation warning for RUST_LOG is emitted on stderr

#### Dependencies

- Depends on: none

### US-002: ecc status command

**As a** developer, **I want** `ecc status` to show workflow state, binary versions, artifact status, and component counts, **so that** I can understand the current ECC state at a glance.

#### Acceptance Criteria

- AC-002.1: Given `ecc status` runs, when output is displayed, then it shows ecc binary version and ecc-workflow binary version
- AC-002.2: Given an active workflow (state.json exists), when `ecc status` runs, then it shows phase, feature name, and started_at timestamp
- AC-002.3: Given no active workflow, when `ecc status` runs, then it shows "No active workflow"
- AC-002.4: Given installed ECC components, when `ecc status` runs, then it shows counts of agents, skills, commands, rules, hooks
- AC-002.5: Given `ecc status` runs, when artifacts exist in state.json, then it shows which artifacts are present (spec, design, tasks) with checkmarks
- AC-002.6: Given ecc-workflow binary is not installed, when `ecc status` runs, then it shows "ecc-workflow: not found" instead of panicking

#### Dependencies

- Depends on: none

### US-003: Persistent verbosity via ecc config

**As a** developer, **I want** `ecc config set log-level <level>` to persist my preferred verbosity, **so that** I don't have to set environment variables every time.

#### Acceptance Criteria

- AC-003.1: Given `ecc config set log-level info`, when the command runs, then `~/.ecc/config.toml` is created/updated with `log-level = "info"`
- AC-003.2: Given `~/.ecc/config.toml` contains `log-level = "info"`, when ecc runs without -v flag or ECC_LOG, then info-level messages appear
- AC-003.3: Given `ECC_LOG=debug` is set AND config has `log-level = "info"`, when ecc runs, then debug-level messages appear (env var takes precedence)
- AC-003.4: Given `-v` flag AND config has `log-level = "error"`, when ecc runs, then info-level messages appear (CLI flag takes precedence)
- AC-003.5: Given `ecc config get log-level`, when the command runs, then it displays the current persistent log level
- AC-003.6: Given `ecc config set log-level banana` (invalid level), when the command runs, then it exits with error listing valid levels {error, warn, info, debug, trace}
- AC-003.7: Given `~/.ecc/config.toml` does not exist, when ecc runs, then it uses the default warn level without error
- AC-003.8: Given `~/.ecc/config.toml` is corrupt/unparseable, when ecc runs, then it logs a warning and falls back to default warn level
- AC-003.9: Given HOME environment variable is not set, when `ecc config set` runs, then it exits with a clear error message ("HOME not set")


#### Dependencies

- Depends on: US-001

### US-004: Tracing instrumentation of key handlers

**As a** developer debugging hook behavior, **I want** info-level tracing events at key decision points, **so that** `-v` reveals what ECC is doing.

#### Acceptance Criteria

- AC-004.1: Given ecc-workflow phase-gate runs with `ECC_LOG=info`, when a phase gate decision is made, then an info-level event is emitted with the phase and decision
- AC-004.2: Given ecc-workflow transition runs with `ECC_LOG=info`, when a state transition occurs, then an info-level event is emitted with from/to phases
- AC-004.3: Given ecc-workflow memory-write runs with `ECC_LOG=info`, when a memory write completes, then an info-level event is emitted with the target file
- AC-004.4: Given all existing `log::warn!` calls in ecc-cli, ecc-app, ecc-infra, and ecc-workflow, when the tracing migration is complete, then they are replaced with `tracing::warn!`
- AC-004.5: Given the existing `observable_logging.rs` test that greps for `log::warn!` patterns, when the migration is complete, then the test is updated to use tracing-compatible assertions
- AC-004.6: Given config.toml I/O in ecc-app, when config is read/written, then it uses the existing FileSystem port trait (no direct std::fs in ecc-app)

#### Dependencies

- Depends on: US-001

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `Cargo.toml` (workspace) | Build | Add tracing, tracing-subscriber to workspace deps |
| `crates/ecc-cli/Cargo.toml` | Build | Replace env_logger with tracing-subscriber |
| `crates/ecc-cli/src/main.rs` | CLI | Replace env_logger init with tracing-subscriber, add -q flag, tiered -v |
| `crates/ecc-cli/src/commands/status.rs` | CLI (new) | New ecc status subcommand |
| `crates/ecc-cli/src/commands/config.rs` | CLI (new) | New ecc config subcommand |
| `crates/ecc-cli/src/commands/mod.rs` | CLI | Add Status and Config subcommand variants |
| `crates/ecc-workflow/Cargo.toml` | Build | Replace env_logger with tracing-subscriber |
| `crates/ecc-workflow/src/main.rs` | Workflow CLI | Replace env_logger with silent-default tracing |
| `crates/ecc-app/Cargo.toml` | Build | Replace log with tracing |
| `crates/ecc-app/src/status.rs` | App (new) | Status use case (read state, count components) |
| `crates/ecc-app/src/ecc_config.rs` | App (new) | Config read/write use case |
| `crates/ecc-domain/src/config/ecc_config.rs` | Domain (new) | EccConfig struct, LogLevel enum, config parsing |
| `crates/ecc-app/src/*.rs` (various) | App | Replace log::warn! with tracing::warn! |
| `crates/ecc-infra/Cargo.toml` | Build | Replace log with tracing |
| `crates/ecc-infra/src/*.rs` (various) | Infra | Replace log::warn! with tracing::warn! |
| `crates/ecc-workflow/src/commands/*.rs` (various) | Workflow | Add tracing::info! instrumentation |

## Constraints

- All tracing output to stderr — zero model cost (never enters Claude's context)
- ecc-workflow silent by default — only emits tracing when ECC_LOG explicitly set
- ecc-domain stays pure — no tracing dependency (tracing-log bridge handles log→tracing)
- Existing tests must pass (log capture tests may need migration to tracing-test)
- ECC_LOG + RUST_LOG both supported (RUST_LOG with deprecation warning)
- Precedence: CLI flag (-v) > env var (ECC_LOG) > config (~/.ecc/config.toml) > default (warn)

## Non-Requirements

- Persistent log files to disk (BL-092)
- SQLite log index (BL-092)
- Cost/token tracking (BL-096)
- Async tracing spans (CLI is sync)
- Structured JSON log output (BL-092)
- OpenTelemetry integration

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| CLI binary (ecc) | New subcommands + tracing init | Integration tests need update |
| CLI binary (ecc-workflow) | Tracing init change | Hook output tests verify no stderr pollution |
| FileSystem port | Read ~/.ecc/config.toml | Config persistence tested via integration |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New CLI commands | CLAUDE.md | CLAUDE.md | Add ecc status and ecc config |
| ADR | ADR | docs/adr/0031-tracing-migration.md | Document log→tracing migration, env var policy |
| Changelog | Project | CHANGELOG.md | Add entry |

## Open Questions

None — all resolved during grill-me.
