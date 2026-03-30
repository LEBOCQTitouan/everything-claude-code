# Design: ECC diagnostics — tiered verbosity with tracing (BL-091)

## Overview

Replace `log`+`env_logger` with `tracing`+`tracing-subscriber` across all ECC binaries, add tiered verbosity (-q/default/-v/-vv/-vvv), a new `ecc status` command, and persistent config via `~/.ecc/config.toml`. The migration is ordered so that each phase compiles and passes tests independently.

## Architecture Changes

| # | File | Layer | Change | AC |
|---|------|-------|--------|----|
| 1 | `Cargo.toml` (workspace) | Build | Add `tracing = "0.1"`, `tracing-subscriber = { version = "0.3", features = ["env-filter"] }`, `tracing-log = "0.1"`, `toml = "0.8"` to workspace deps | AC-001.1 |
| 2 | `crates/ecc-domain/src/config/ecc_config.rs` | Entity | New `LogLevel` enum, `EccConfig` struct, TOML parse/serialize | AC-003.1, AC-003.6 |
| 3 | `crates/ecc-domain/src/config/mod.rs` | Entity | Add `pub mod ecc_config;` | AC-003.1 |
| 4 | `crates/ecc-app/src/ecc_config.rs` | UseCase | Config get/set use case via `FileSystem` port | AC-003.1, AC-003.5, AC-003.9, AC-004.6 |
| 5 | `crates/ecc-app/src/status.rs` | UseCase | Status use case — read workflow state, count components, check binary versions | AC-002.1..AC-002.6 |
| 6 | `crates/ecc-app/src/lib.rs` | UseCase | Add `pub mod ecc_config; pub mod status;` | — |
| 7 | `crates/ecc-app/Cargo.toml` | Build | Replace `log = "0.4"` with `tracing = { workspace = true }`, add `toml = { workspace = true }` | AC-001.1, AC-004.4 |
| 8 | `crates/ecc-infra/Cargo.toml` | Build | Replace `log = "0.4"` with `tracing = { workspace = true }` | AC-001.1, AC-004.4 |
| 9 | `crates/ecc-cli/Cargo.toml` | Build | Remove `env_logger`, `log`; add `tracing`, `tracing-subscriber`, `tracing-log`, `toml` | AC-001.1 |
| 10 | `crates/ecc-cli/src/main.rs` | Framework | Replace env_logger init with tracing-subscriber, add -q flag, tiered -v (Count), ECC_LOG/RUST_LOG precedence, config file loading | AC-001.2..AC-001.7, AC-001.10, AC-003.2..AC-003.4, AC-003.7, AC-003.8 |
| 11 | `crates/ecc-cli/src/commands/mod.rs` | Framework | Add `pub mod status; pub mod config;` | — |
| 12 | `crates/ecc-cli/src/commands/status.rs` | Framework | New `ecc status` subcommand, wire to app use case | AC-002.1..AC-002.6 |
| 13 | `crates/ecc-cli/src/commands/config.rs` | Framework | New `ecc config get/set` subcommand | AC-003.1, AC-003.5, AC-003.6, AC-003.9 |
| 14 | `crates/ecc-workflow/Cargo.toml` | Build | Remove `env_logger`, `log`; add `tracing`, `tracing-subscriber` | AC-001.1 |
| 15 | `crates/ecc-workflow/src/main.rs` | Framework | Replace env_logger with silent-default tracing (only when ECC_LOG set) | AC-001.8, AC-001.9 |
| 16 | `crates/ecc-app/src/**/*.rs` | UseCase | Replace all `log::warn!` with `tracing::warn!`, `log::error!` with `tracing::error!`, etc. | AC-004.4 |
| 17 | `crates/ecc-infra/src/**/*.rs` | Infra | Replace all `log::warn!` with `tracing::warn!` etc. | AC-004.4 |
| 18 | `crates/ecc-cli/src/commands/version.rs` | Framework | Replace `log::debug!` with `tracing::debug!` | AC-004.4 |
| 19 | `crates/ecc-workflow/src/commands/phase_gate.rs` | Framework | Add `tracing::info!` at decision points | AC-004.1 |
| 20 | `crates/ecc-workflow/src/commands/transition.rs` | Framework | Add `tracing::info!` for from/to phases | AC-004.2 |
| 21 | `crates/ecc-workflow/src/commands/memory_write.rs` | Framework | Add `tracing::info!` for target file | AC-004.3 |
| 22 | `crates/ecc-workflow/src/io.rs` | Framework | Replace `log::warn!` with `tracing::warn!` | AC-004.4 |
| 23 | `crates/ecc-app/tests/observable_logging.rs` | Test | Update grep pattern to accept both `tracing::warn!` and `warn!(` | AC-004.5 |
| 24 | `docs/adr/0031-tracing-migration.md` | Doc | ADR for log-to-tracing migration | Decision 1-7 |
| 25 | `CLAUDE.md` | Doc | Add `ecc status` and `ecc config` to CLI commands section | Doc impact |
| 26 | `CHANGELOG.md` | Doc | Add v4.3.0 entry | Doc impact |

## Precedence Resolution Algorithm (ecc-cli main.rs)

```
fn resolve_log_level(cli_verbosity: u8, cli_quiet: bool) -> String {
    // 1. CLI flag (highest priority)
    if cli_quiet { return "error" }
    if cli_verbosity >= 3 { return "trace" }
    if cli_verbosity >= 2 { return "debug" }
    if cli_verbosity >= 1 { return "info" }

    // 2. ECC_LOG env var
    if let Ok(val) = std::env::var("ECC_LOG") { return val }

    // 3. RUST_LOG env var (with deprecation warning)
    if let Ok(val) = std::env::var("RUST_LOG") {
        eprintln!("warning: RUST_LOG is deprecated for ECC, use ECC_LOG instead");
        return val
    }

    // 4. Config file (~/.ecc/config.toml)
    if let Some(level) = read_config_log_level() { return level }

    // 5. Default
    "warn"
}
```

## Domain Types (ecc-domain)

```rust
// crates/ecc-domain/src/config/ecc_config.rs

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub const VALID_LEVELS: &[&str] = &["error", "warn", "info", "debug", "trace"];
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EccConfig {
    pub log_level: Option<LogLevel>,
}

// FromStr, Display, TOML serialize/deserialize
```

## Config Use Case (ecc-app)

```rust
// crates/ecc-app/src/ecc_config.rs

pub fn config_set(fs: &dyn FileSystem, env: &dyn Environment, key: &str, value: &str)
    -> Result<(), ConfigError>

pub fn config_get(fs: &dyn FileSystem, env: &dyn Environment, key: &str)
    -> Result<Option<String>, ConfigError>

pub fn read_config(fs: &dyn FileSystem, env: &dyn Environment)
    -> Result<EccConfig, ConfigError>
```

Config path: `env.home_dir()?.join(".ecc/config.toml")`. Uses `FileSystem` port for all I/O (AC-004.6).

## Status Use Case (ecc-app)

```rust
// crates/ecc-app/src/status.rs

pub struct EccStatus {
    pub ecc_version: String,
    pub workflow_version: Option<String>,  // from `ecc-workflow --version` or None
    pub workflow: Option<WorkflowInfo>,    // from state.json
    pub components: ComponentCounts,       // from manifest
    pub artifacts: ArtifactStatus,         // from state.json
}

pub fn ecc_status(
    fs: &dyn FileSystem,
    env: &dyn Environment,
    shell: &dyn ShellExecutor,
) -> EccStatus
```

For `ecc-workflow` version: shell executor runs `ecc-workflow --version`, captures stdout. If command fails, `workflow_version = None` (AC-002.6).

## ecc-workflow Tracing Init

```rust
// Silent by default. Only enable tracing subscriber when ECC_LOG is explicitly set.
if let Ok(filter) = std::env::var("ECC_LOG") {
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
}
```

No RUST_LOG fallback in ecc-workflow (AC-001.8) -- only ECC_LOG activates tracing.

## Pass Conditions (TDD dependency order)

| PC | Phase | Description | Layers | Pass Command | AC |
|----|-------|-------------|--------|--------------|-----|
| PC-001 | 1 | `LogLevel` enum parses valid levels, rejects invalid | Entity | `cargo test -p ecc-domain config::ecc_config` | AC-003.6 |
| PC-002 | 1 | `EccConfig` round-trips through TOML serialize/deserialize | Entity | `cargo test -p ecc-domain config::ecc_config` | AC-003.1 |
| PC-003 | 2 | `config_set` writes `~/.ecc/config.toml` via FileSystem port | UseCase | `cargo test -p ecc-app ecc_config` | AC-003.1, AC-004.6 |
| PC-004 | 2 | `config_get` reads persisted log-level | UseCase | `cargo test -p ecc-app ecc_config` | AC-003.5 |
| PC-005 | 2 | `config_set` rejects invalid key | UseCase | `cargo test -p ecc-app ecc_config` | AC-003.6 |
| PC-006 | 2 | `config_set` rejects invalid log-level value | UseCase | `cargo test -p ecc-app ecc_config` | AC-003.6 |
| PC-007 | 2 | `read_config` returns default when file missing | UseCase | `cargo test -p ecc-app ecc_config` | AC-003.7 |
| PC-008 | 2 | `read_config` warns and returns default when file corrupt | UseCase | `cargo test -p ecc-app ecc_config` | AC-003.8 |
| PC-009 | 2 | `config_set` errors when HOME not set | UseCase | `cargo test -p ecc-app ecc_config` | AC-003.9 |
| PC-010 | 3 | `ecc_status` returns version and component counts from manifest | UseCase | `cargo test -p ecc-app status` | AC-002.1, AC-002.4 |
| PC-011 | 3 | `ecc_status` shows active workflow info when state.json exists | UseCase | `cargo test -p ecc-app status` | AC-002.2, AC-002.5 |
| PC-012 | 3 | `ecc_status` shows "No active workflow" when state.json absent | UseCase | `cargo test -p ecc-app status` | AC-002.3 |
| PC-013 | 3 | `ecc_status` handles missing ecc-workflow binary gracefully | UseCase | `cargo test -p ecc-app status` | AC-002.6 |
| PC-014 | 4 | Workspace deps updated: tracing, tracing-subscriber in Cargo.toml | Build | `cargo check --workspace` | AC-001.1 |
| PC-015 | 4 | ecc-cli tracing init with tiered verbosity compiles | Framework | `cargo build -p ecc-cli` | AC-001.2..AC-001.7 |
| PC-016 | 4 | ecc-cli reads ECC_LOG with RUST_LOG deprecation fallback | Framework | `cargo test -p ecc-cli` (integration) | AC-001.2, AC-001.10 |
| PC-017 | 4 | ecc-cli reads config file as lowest-priority source | Framework | `cargo test -p ecc-cli` (integration) | AC-003.2..AC-003.4 |
| PC-018 | 5 | `ecc config set/get` CLI subcommands wire to app use case | Framework | `cargo test -p ecc-cli` | AC-003.1, AC-003.5 |
| PC-019 | 5 | `ecc status` CLI subcommand wires to app use case | Framework | `cargo test -p ecc-cli` | AC-002.1..AC-002.6 |
| PC-020 | 6 | ecc-workflow silent default (no stderr when ECC_LOG unset) | Framework | `cargo test -p ecc-workflow` | AC-001.8 |
| PC-021 | 6 | ecc-workflow emits tracing when ECC_LOG set | Framework | `cargo test -p ecc-workflow` | AC-001.9 |
| PC-022 | 7 | All `log::warn!` in ecc-app replaced with `tracing::warn!` | UseCase | `grep -r 'log::warn!' crates/ecc-app/src/ \| wc -l` returns 0 | AC-004.4 |
| PC-023 | 7 | All `log::warn!` in ecc-infra replaced with `tracing::warn!` | Adapter | `grep -r 'log::warn!' crates/ecc-infra/src/ \| wc -l` returns 0 | AC-004.4 |
| PC-024 | 7 | All `log::` in ecc-workflow replaced with `tracing::` | Framework | `grep -r 'log::' crates/ecc-workflow/src/ \| wc -l` returns 0 | AC-004.4 |
| PC-025 | 7 | observable_logging.rs test updated and passes | Test | `cargo test -p ecc-app --test observable_logging` | AC-004.5 |
| PC-026 | 8 | ecc-workflow phase_gate emits info event at decision point | Framework | `ECC_LOG=info cargo test -p ecc-workflow phase_gate` | AC-004.1 |
| PC-027 | 8 | ecc-workflow transition emits info event with from/to | Framework | `ECC_LOG=info cargo test -p ecc-workflow transition` | AC-004.2 |
| PC-028 | 8 | ecc-workflow memory_write emits info event with target file | Framework | `ECC_LOG=info cargo test -p ecc-workflow memory_write` | AC-004.3 |
| PC-029 | 9 | `cargo clippy --workspace -- -D warnings` passes | Build | `cargo clippy --workspace -- -D warnings` | — |
| PC-030 | 9 | `cargo build --workspace` succeeds | Build | `cargo build --workspace` | — |
| PC-031 | 9 | `cargo test --workspace` all tests pass | Build | `cargo test --workspace` | — |
| PC-032 | 10 | ADR 0031 written | Doc | `test -f docs/adr/0031-tracing-migration.md` | Decision 1-7 |
| PC-033 | 10 | CLAUDE.md updated with ecc status, ecc config | Doc | `grep 'ecc status' CLAUDE.md && grep 'ecc config' CLAUDE.md` | Doc impact |

## Implementation Phases

### Phase 1: Domain — LogLevel & EccConfig (Entity)

Layers: [Entity]

**Files:**
- `crates/ecc-domain/src/config/ecc_config.rs` (new)
- `crates/ecc-domain/src/config/mod.rs` (add module)

**Actions:**
1. Create `LogLevel` enum with `FromStr`, `Display`, `PartialEq`, `Eq`, `Clone`, `Debug`
2. Create `EccConfig` struct with `log_level: Option<LogLevel>`
3. Implement `EccConfig::from_toml(content: &str) -> Result<Self, ConfigError>` (pure parsing)
4. Implement `EccConfig::to_toml(&self) -> String` (pure serialization)
5. Add `pub mod ecc_config;` to `config/mod.rs`

**Dependencies:** None
**Risk:** Low — pure domain types, no I/O

**Test targets:**
- `LogLevel::from_str("info")` returns `Ok(LogLevel::Info)`
- `LogLevel::from_str("banana")` returns descriptive error with valid levels
- `EccConfig::from_toml("log-level = \"info\"")` parses correctly
- `EccConfig::to_toml()` round-trips
- `EccConfig::from_toml("")` returns config with `log_level: None`
- `EccConfig::from_toml("garbage{{{")` returns parse error
- Expected test file: `crates/ecc-domain/src/config/ecc_config.rs` (`#[cfg(test)]` module)

**Pass condition:** `cargo test -p ecc-domain config::ecc_config` (PC-001, PC-002)

### Phase 2: UseCase — Config get/set (UseCase)

Layers: [UseCase]

**Files:**
- `crates/ecc-app/src/ecc_config.rs` (new)
- `crates/ecc-app/src/lib.rs` (add module)
- `crates/ecc-app/Cargo.toml` (add `toml` dep)

**Actions:**
1. Add `toml = { workspace = true }` to ecc-app Cargo.toml (alongside existing deps)
2. Create `ecc_config.rs` with `config_set`, `config_get`, `read_config` functions
3. All functions take `&dyn FileSystem` + `&dyn Environment` for testability
4. Config path: `env.home_dir().join(".ecc/config.toml")`
5. `config_set` validates key is `"log-level"` (whitelisted), validates value via `LogLevel::from_str`
6. `config_set` creates `~/.ecc/` directory if missing, writes TOML atomically
7. `config_get` reads and parses TOML, returns `Option<String>`
8. `read_config` returns `EccConfig` — default on missing file, warns on corrupt
9. Add `pub mod ecc_config;` to `lib.rs`

**Dependencies:** Phase 1 (LogLevel/EccConfig types)
**Risk:** Medium — must handle all error paths (missing HOME, corrupt file, missing dir)

**Test targets (using InMemoryFileSystem + MockEnvironment):**
- `config_set` writes valid TOML to `~/.ecc/config.toml`
- `config_set` creates `~/.ecc/` dir when absent
- `config_set` rejects unknown key (e.g., "theme")
- `config_set` rejects invalid level value (e.g., "banana")
- `config_get` returns `Some("info")` after `config_set("log-level", "info")`
- `config_get` returns `None` when file missing
- `read_config` returns default `EccConfig` when file missing
- `read_config` warns (tracing event) and returns default when file corrupt
- `config_set` errors with clear message when HOME not set
- Expected test file: `crates/ecc-app/src/ecc_config.rs` (`#[cfg(test)]` module)

**Pass condition:** `cargo test -p ecc-app ecc_config` (PC-003..PC-009)

### Phase 3: UseCase — Status (UseCase)

Layers: [UseCase]

**Files:**
- `crates/ecc-app/src/status.rs` (new — distinct from `dev/status.rs`)
- `crates/ecc-app/src/lib.rs` (add module)

**Actions:**
1. Define `EccStatus`, `WorkflowInfo`, `ComponentCounts`, `ArtifactStatus` structs
2. Implement `ecc_status(fs, env, shell) -> EccStatus`
3. Read ecc version from `version()` function (already exists in ecc-app)
4. Run `ecc-workflow --version` via ShellExecutor, capture stdout; None on failure (AC-002.6)
5. Read `~/.claude/.ecc-manifest.json` via FileSystem for component counts (reuse `read_manifest`)
6. Read `.claude/workflow/state.json` via FileSystem for workflow info
7. Artifact status: spec/design/tasks presence from state.json
8. Add `pub mod status;` to lib.rs (rename will be needed since `status` module path collides — use `ecc_status` or namespace under a different name)

**Note:** `ecc-app/src/dev/status.rs` already exists with `DevStatus`. The new `status.rs` at crate root is a different use case. To avoid confusion, name it `ecc_status.rs`.

**Dependencies:** None (reads existing manifest/state.json formats)
**Risk:** Medium — ShellExecutor integration for ecc-workflow version

**Test targets (using InMemoryFileSystem + MockEnvironment + MockExecutor):**
- Returns ecc version from `version()`
- Returns workflow version from ShellExecutor output
- Returns `None` workflow version when ecc-workflow not found
- Returns component counts from manifest
- Returns active workflow info from state.json
- Returns "No active workflow" when state.json absent
- Shows artifact checkmarks when spec/design/tasks paths present
- Expected test file: `crates/ecc-app/src/ecc_status.rs` (`#[cfg(test)]` module)

**Pass condition:** `cargo test -p ecc-app ecc_status` (PC-010..PC-013)

### Phase 4: Framework — Workspace deps & tracing init (Framework)

Layers: [Framework]

**Files:**
- `Cargo.toml` (workspace root)
- `crates/ecc-cli/Cargo.toml`
- `crates/ecc-cli/src/main.rs`
- `crates/ecc-workflow/Cargo.toml`
- `crates/ecc-infra/Cargo.toml`
- `crates/ecc-app/Cargo.toml`

**Actions:**
1. Add to workspace deps: `tracing = "0.1"`, `tracing-subscriber = { version = "0.3", features = ["env-filter"] }`, `tracing-log = "0.1"`, `toml = "0.8"`
2. ecc-cli Cargo.toml: remove `log = "0.4"` and `env_logger = "0.11"`, add `tracing = { workspace = true }`, `tracing-subscriber = { workspace = true }`, `tracing-log = { workspace = true }`, `toml = { workspace = true }`
3. ecc-app Cargo.toml: replace `log = "0.4"` with `tracing = { workspace = true }`, add `toml = { workspace = true }`; replace `testing_logger` dev-dep with `tracing-test = "0.2"` or remove if not needed
4. ecc-infra Cargo.toml: replace `log = "0.4"` with `tracing = { workspace = true }`
5. ecc-workflow Cargo.toml: remove `log = "0.4"` and `env_logger = "0.11"`, add `tracing = { workspace = true }`, `tracing-subscriber = { workspace = true }`
6. ecc-cli main.rs: rewrite the `Cli` struct:
   - Change `verbose: bool` to `verbose: u8` with `action = ArgAction::Count`
   - Add `#[arg(short, long, global = true)] quiet: bool`
   - Replace env_logger init block with tracing-subscriber init using `resolve_log_level`
7. Implement `resolve_log_level` in main.rs (CLI flag > ECC_LOG > RUST_LOG+warning > config > "warn")
8. For config reading in main.rs: use `OsFileSystem` + `OsEnvironment` to call `read_config`

**Dependencies:** Phases 1-2 (config reading depends on domain types + app use case)
**Risk:** High — this is the core migration, must not break existing behavior

**Test targets:**
- `cargo check --workspace` passes (PC-014)
- `cargo build -p ecc-cli` succeeds (PC-015)
- Integration test: with ECC_LOG=info, tracing captures info events (PC-016)
- Integration test: with RUST_LOG set, deprecation warning emitted on stderr (PC-016)
- Integration test: config file level used when no flags/env (PC-017)
- Expected test file: `crates/ecc-cli/tests/tracing_init.rs` or inline in main (may require binary integration tests)

**Pass condition:** `cargo check --workspace && cargo build -p ecc-cli` (PC-014..PC-017)

### Phase 5: Framework — CLI subcommands status & config (Framework)

Layers: [Framework]

**Files:**
- `crates/ecc-cli/src/commands/status.rs` (new)
- `crates/ecc-cli/src/commands/config.rs` (new)
- `crates/ecc-cli/src/commands/mod.rs` (add modules)
- `crates/ecc-cli/src/main.rs` (add Command variants + dispatch)

**Actions:**
1. Create `status.rs`: wire `StatusArgs` to `ecc_app::ecc_status::ecc_status()`, format output
2. Create `config.rs`: wire `ConfigArgs` with `Set { key, value }` and `Get { key }` subcommands to `ecc_app::ecc_config::{config_set, config_get}`
3. Add `Status` and `Config` variants to `Command` enum in main.rs
4. Add dispatch arms
5. Add `pub mod status; pub mod config;` to commands/mod.rs

**Dependencies:** Phases 2-4 (app use cases + tracing init)
**Risk:** Low — thin CLI wiring

**Test targets:**
- `ecc status` displays version and "No active workflow" in clean state
- `ecc config set log-level info` succeeds
- `ecc config get log-level` returns "info"
- `ecc config set log-level banana` exits with error
- Expected test file: `crates/ecc-cli/src/commands/status.rs` and `config.rs` (inline tests or integration)

**Pass condition:** `cargo test -p ecc-cli` (PC-018, PC-019)

### Phase 6: Framework — ecc-workflow tracing migration (Framework)

Layers: [Framework]

**Files:**
- `crates/ecc-workflow/src/main.rs`

**Actions:**
1. Remove `env_logger::init()` call
2. Add conditional tracing init: only when `ECC_LOG` env var is set
3. Replace `log::debug!` with `tracing::debug!`
4. Use `tracing_subscriber::fmt().with_env_filter(...).with_writer(std::io::stderr).init()`

**Dependencies:** Phase 4 (workspace deps)
**Risk:** Medium — must verify hook JSON output is not polluted

**Test targets:**
- With ECC_LOG unset: ecc-workflow produces zero stderr output (PC-020)
- With ECC_LOG=info: ecc-workflow produces tracing output on stderr (PC-021)
- Expected test file: `crates/ecc-workflow/tests/silent_default.rs` (integration, `#[ignore]`)

**Pass condition:** `cargo test -p ecc-workflow` (PC-020, PC-021)

### Phase 7: Mechanical — log-to-tracing macro replacement (UseCase, Adapter)

Layers: [UseCase, Adapter]

**Files:**
- All `crates/ecc-app/src/**/*.rs` files with `log::` calls (~35 sites)
- All `crates/ecc-infra/src/**/*.rs` files with `log::` calls (~4 sites)
- All `crates/ecc-workflow/src/**/*.rs` files with `log::` calls (~3 sites)
- `crates/ecc-cli/src/commands/version.rs` (1 site)
- `crates/ecc-app/tests/observable_logging.rs` (update grep pattern)

**Actions:**
1. Global find-replace: `log::warn!` -> `tracing::warn!`, `log::debug!` -> `tracing::debug!`, `log::error!` -> `tracing::error!`, `log::info!` -> `tracing::info!`
2. Remove `use log;` or `extern crate log;` if present
3. Update `observable_logging.rs` to grep for `tracing::warn!` instead of `log::warn!`
4. Remove `log` from `[dependencies]` in any crate Cargo.toml that still has it
5. Remove `testing_logger` from ecc-app dev-dependencies if no longer needed

**Dependencies:** Phase 6 (all tracing init must be in place before replacing macros)
**Risk:** Low — mechanical replacement, but high file count

**Test targets:**
- `grep -r 'log::warn!' crates/ecc-app/src/` returns 0 matches (PC-022)
- `grep -r 'log::warn!' crates/ecc-infra/src/` returns 0 matches (PC-023)
- `grep -r 'log::' crates/ecc-workflow/src/` returns 0 matches (PC-024)
- `observable_logging.rs` test passes with updated patterns (PC-025)
- Expected test file: `crates/ecc-app/tests/observable_logging.rs` (updated)

**Pass condition:** grep commands + `cargo test -p ecc-app --test observable_logging` (PC-022..PC-025)

### Phase 8: Framework — Workflow instrumentation (Framework)

Layers: [Framework]

**Files:**
- `crates/ecc-workflow/src/commands/phase_gate.rs`
- `crates/ecc-workflow/src/commands/transition.rs`
- `crates/ecc-workflow/src/commands/memory_write.rs`

**Actions:**
1. `phase_gate.rs`: Add `tracing::info!(phase = %phase, decision = %decision, "phase gate evaluated")` at the point where the gate decision is made
2. `transition.rs`: Add `tracing::info!(from = %current, to = %target, "workflow transition")` after successful transition
3. `memory_write.rs`: Add `tracing::info!(target = %path, kind = %kind, "memory write complete")` after successful write

**Dependencies:** Phase 6 (ecc-workflow tracing init)
**Risk:** Low — additive instrumentation

**Test targets:**
- With ECC_LOG=info, phase_gate emits info event containing phase and decision (PC-026)
- With ECC_LOG=info, transition emits info event containing from/to phases (PC-027)
- With ECC_LOG=info, memory_write emits info event containing target file (PC-028)
- Expected test files: inline `#[cfg(test)]` in each command file

**Pass condition:** `cargo test -p ecc-workflow` (PC-026..PC-028)

### Phase 9: Quality gate — lint + build + test (Build)

Layers: [Build]

**Actions:**
1. `cargo fmt --all`
2. `cargo clippy --workspace -- -D warnings`
3. `cargo build --workspace`
4. `cargo test --workspace`
5. Fix any issues found

**Dependencies:** All previous phases
**Risk:** Low

**Pass condition:** All three commands exit 0 (PC-029..PC-031)

### Phase 10: Documentation (Doc)

Layers: [Doc]

**Files:**
- `docs/adr/0031-tracing-migration.md` (new)
- `CLAUDE.md` (update CLI commands section)
- `CHANGELOG.md` (add entry)

**Actions:**
1. Write ADR 0031 covering: decision to replace log+env_logger with tracing+tracing-subscriber, ECC_LOG env var policy, RUST_LOG deprecation, ecc-workflow silent default, ecc-domain purity preserved, config.toml location
2. Update CLAUDE.md CLI commands list to include `ecc status` and `ecc config set/get log-level`
3. Add CHANGELOG entry for v4.3.0

**Dependencies:** Phases 1-9 complete
**Risk:** Low

**Pass condition:** `test -f docs/adr/0031-tracing-migration.md && grep 'ecc status' CLAUDE.md && grep 'ecc config' CLAUDE.md` (PC-032, PC-033)

## E2E Assessment

- **Touches user-facing flows?** Yes -- new CLI subcommands (`ecc status`, `ecc config`), changed tracing output
- **Crosses 3+ modules end-to-end?** Yes -- domain -> app -> cli, plus ecc-workflow
- **New E2E tests needed?** Yes, but lightweight (binary invocation tests)
- **E2E scenarios** (after Phase 9):
  1. `ecc status` produces expected output sections (version, workflow, components)
  2. `ecc config set log-level info && ecc config get log-level` returns "info"
  3. `ecc -v version 2>&1` produces info-level tracing on stderr
  4. `ecc -q version 2>&1` produces no tracing on stderr
  5. `ECC_LOG=debug ecc version 2>&1` produces debug-level tracing on stderr
  6. `ecc-workflow status 2>/tmp/err && test ! -s /tmp/err` (no stderr by default)

## Testing Strategy

- **Unit tests:** LogLevel parsing, EccConfig TOML round-trip, config_set/get validation, status struct assembly (Phases 1-3)
- **Integration tests:** Tracing init with env vars, CLI subcommand wiring (Phases 4-6)
- **E2E tests:** Binary invocation of `ecc status`, `ecc config`, verbosity flags (post Phase 9)
- **Mechanical verification:** grep for `log::` residue (Phase 7)

## Risks & Mitigations

- **Risk:** Tracing subscriber init order conflicts with tests that also init subscribers
  - Mitigation: Use `tracing_subscriber::fmt().try_init()` (returns Err if already set, non-fatal)

- **Risk:** ecc-workflow hook JSON output contaminated by tracing stderr
  - Mitigation: Silent default (Phase 6) + integration test verifying empty stderr (PC-020)

- **Risk:** `testing_logger` crate incompatible after removing `log` dep
  - Mitigation: Remove `testing_logger` from dev-deps, use `tracing-test` crate or manual assertions

- **Risk:** Large file count in Phase 7 mechanical replacement introduces typos
  - Mitigation: Use sed/replace-all, then verify with `cargo build --workspace` + grep

- **Risk:** ecc-domain gaining tracing dependency violates purity constraint
  - Mitigation: Domain uses no logging crate; `tracing-log` bridge in ecc-app/cli automatically captures `log::` macro calls if any remain during transition

## Success Criteria

- [ ] `-q` shows only errors, default shows warns, `-v` shows info, `-vv` shows debug, `-vvv` shows trace
- [ ] `ECC_LOG` takes precedence over `RUST_LOG` (with deprecation warning)
- [ ] `~/.ecc/config.toml` persists log-level preference
- [ ] Precedence: CLI flag > ECC_LOG > RUST_LOG > config > default(warn)
- [ ] `ecc status` shows version, workflow state, component counts, artifact status
- [ ] `ecc status` handles missing ecc-workflow gracefully
- [ ] ecc-workflow produces zero stderr by default
- [ ] ecc-domain has zero tracing/log dependencies
- [ ] All ~60 `log::` macro calls replaced with `tracing::` equivalents
- [ ] Key workflow handlers (phase-gate, transition, memory-write) emit info events
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] ADR 0031 documents migration rationale

## Rollback Plan

Reverse phase order:
1. Revert Phase 10 docs (ADR, CLAUDE.md, CHANGELOG)
2. Revert Phase 8 instrumentation (remove tracing::info! calls)
3. Revert Phase 7 migration (restore log:: macros — use `git revert` on mechanical replacement commits)
4. Revert Phase 6 ecc-workflow tracing init (restore env_logger)
5. Revert Phase 5 CLI wiring (remove status/config subcommands)
6. Revert Phase 4 workspace deps (restore log+env_logger, remove tracing+tracing-subscriber)
7. Revert Phase 3 status use case (delete ecc-app/src/status.rs)
8. Revert Phase 2 config use case (delete ecc-app/src/ecc_config.rs)
9. Revert Phase 1 domain types (delete ecc-domain/src/config/ecc_config.rs)

**Critical safety note**: Phase 7 (log→tracing mechanical replacement) must keep `log` as a dependency via `tracing-log` bridge until ALL call sites are migrated. The `log` crate is only removed from direct deps AFTER Phase 7 completes and PC-022/023/024 verify zero remaining `log::` calls.

## SOLID Assessment

PASS — Domain pure (LogLevel/EccConfig have no I/O deps). App uses FileSystem + Environment + ShellExecutor ports. CLI is thin dispatch. tracing-log bridge keeps ecc-domain independent.

## Robert's Oath Check

CLEAN — 33 PCs, TDD phases, atomic commits per phase. No mess.

## Security Notes

CLEAR — Config is user-local (~/.ecc/config.toml). No network I/O. No secrets. Input validation on log level values.

## Design Fixes (from adversarial review)

### Fix 1: TOML parsing location
EccConfig parsing uses hand-rolled TOML (key=value single-line format) in ecc-domain, NOT the `toml` crate. The `toml` crate dependency goes to ecc-app only for robust serialization. Domain `from_toml` is a pure string parser.

### Fix 2: Tracing test strategy
PC-026/027/028 use `tracing-test` crate's `#[traced_test]` attribute in unit tests, NOT env var prefix. This sets up a per-test subscriber. The `ECC_LOG=info` prefix in the PC commands is for integration tests only.

### Fix 3: ecc-cli log:: sites included in Phase 7
Phase 7 migration scope explicitly includes ecc-cli (backlog.rs, install.rs, main.rs) alongside ecc-app, ecc-infra, and ecc-workflow. PC-022 expanded to check ALL crates.
