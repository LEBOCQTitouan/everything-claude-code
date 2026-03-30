# Design: BL-091 ECC Diagnostics — Tiered Verbosity

## Overview

Replace `log`/`env_logger` with `tracing`/`tracing-subscriber` across 4 crates, add tiered verbosity CLI flags, instrument key handlers, create `ecc status` and `ecc config set` commands, and add a new `ConfigStore` port with persistence at `~/.ecc/config.toml`. Domain and ports remain tracing-free (ADR-0032).

## Architecture Changes

| # | Change | File(s) | Layer |
|---|--------|---------|-------|
| 1 | Add `LogLevel` value object | `crates/ecc-domain/src/log_level.rs` | Entity |
| 2 | Add `ConfigStore` port trait | `crates/ecc-ports/src/config_store.rs` | Ports |
| 3 | Replace `log` with `tracing` in ecc-app | `crates/ecc-app/Cargo.toml`, ~15 source files | App |
| 4 | Add diagnostics use case (`ecc status`) | `crates/ecc-app/src/diagnostics.rs` | App |
| 5 | Add config use case (`ecc config set`) | `crates/ecc-app/src/config_store.rs` | App |
| 6 | Replace `log` with `tracing` in ecc-infra | `crates/ecc-infra/Cargo.toml`, 1 source file | Infra |
| 7 | Add `FileConfigStore` adapter | `crates/ecc-infra/src/file_config_store.rs` | Infra |
| 8 | Add `InMemoryConfigStore` test double | `crates/ecc-test-support/src/in_memory_config_store.rs` | Test Support |
| 9 | Replace `log`/`env_logger` with `tracing`/`tracing-subscriber` in ecc-cli | `crates/ecc-cli/Cargo.toml`, `main.rs` | CLI |
| 10 | Add `-v`/`-vv`/`-vvv`/`-q` flags, remove `--verbose` | `crates/ecc-cli/src/main.rs` | CLI |
| 11 | Add `ecc status` command | `crates/ecc-cli/src/commands/status.rs` | CLI |
| 12 | Add `ecc config set` command | `crates/ecc-cli/src/commands/config.rs` | CLI |
| 13 | Replace `log`/`env_logger` with `tracing`/`tracing-subscriber` in ecc-workflow | `crates/ecc-workflow/Cargo.toml`, `main.rs` | CLI |
| 14 | Add `-v`/`-vv`/`-vvv`/`-q` flags to ecc-workflow | `crates/ecc-workflow/src/main.rs` | CLI |
| 15 | Add ecc-infra dep to ecc-workflow | `crates/ecc-workflow/Cargo.toml` | CLI |
| 16 | Add tracing instrumentation to dispatch, phase-gate, transitions, etc. | `crates/ecc-app/src/hook/mod.rs`, ecc-workflow command files | App + CLI |
| 17 | Update observable_logging test | `crates/ecc-app/tests/observable_logging.rs` | Test |
| 18 | ADR-0032 | `docs/adr/0032-tracing-cross-cutting.md` | Docs |
| 19 | Update workspace Cargo.toml | `Cargo.toml` | Build |
| 20 | Update CLAUDE.md | `CLAUDE.md` | Docs |

## Dependency Graph

```
Phase 1 (Entity + Ports)
  LogLevel value object in ecc-domain
  ConfigStore port trait in ecc-ports
  InMemoryConfigStore in ecc-test-support
    |
Phase 2 (Crate Migration: ecc-infra)
  Replace log -> tracing in ecc-infra
  Add FileConfigStore adapter
    |
Phase 3 (Crate Migration: ecc-app)
  Replace log -> tracing in ecc-app (~15 files)
  Add diagnostics use case
  Add config_store use case
  Add tracing instrumentation to dispatch/handlers
    |
Phase 4 (Binary: ecc-cli)
  Replace log/env_logger -> tracing/tracing-subscriber
  Add -v/-q flags, remove --verbose
  Wire subscriber init
  Add ecc status command
  Add ecc config set command
    |
Phase 5 (Binary: ecc-workflow)
  Replace log/env_logger -> tracing/tracing-subscriber
  Add -v/-q flags
  Add ecc-infra dep for shared ConfigStore
  Wire subscriber init
  Add tracing instrumentation to phase-gate, transitions, memory writes
    |
Phase 6 (Docs + Final)
  ADR-0032
  Update CLAUDE.md
  Clippy + build gate
```

## Implementation Phases

### Phase 1: Entity + Ports + Test Support

**Layers: [Entity, Ports]**

**Step 1.1**: Add `LogLevel` value object to `ecc-domain`

File: `crates/ecc-domain/src/log_level.rs`
- Define `LogLevel` enum: `Error`, `Warn`, `Info`, `Debug`, `Trace`
- Derive `Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize`
- Implement `Display` (lowercase: "error", "warn", "info", "debug", "trace")
- Implement `FromStr` with case-insensitive parsing, returning a descriptive error for invalid values
- Implement `Default` returning `Warn`
- No tracing dependency

File: `crates/ecc-domain/src/lib.rs`
- Add `pub mod log_level;`

Risk: Low
Dependencies: None

**Step 1.2**: Add `ConfigStore` port trait to `ecc-ports`

File: `crates/ecc-ports/src/config_store.rs`
- Define `EccConfig` struct: `log_level: Option<LogLevel>` (serde optional fields, `#[serde(default)]`)
- Define `ConfigStore` trait:
  - `fn load_global(&self) -> Result<EccConfig, ConfigError>` — reads `~/.ecc/config.toml`
  - `fn load_local(&self, project_dir: &Path) -> Result<EccConfig, ConfigError>` — reads `.ecc/config.toml`
  - `fn load_merged(&self, project_dir: &Path) -> Result<EccConfig, ConfigError>` — global + local, local wins per-key
  - `fn save_global(&self, config: &EccConfig) -> Result<(), ConfigError>`
- Define `ConfigError` enum: `Io(PathBuf, String)`, `Parse(PathBuf, String)`, `Serialize(String)`

File: `crates/ecc-ports/src/lib.rs`
- Add `pub mod config_store;`

File: `crates/ecc-ports/Cargo.toml`
- Add `ecc-domain = { workspace = true }` dependency (for `LogLevel` type)
- Add `serde = { workspace = true }` dependency (for `EccConfig` derive)
- Add `toml = "0.8"` dependency (for deserialization in trait default methods? No — keep port pure. Toml goes in infra only.)

Wait — the port trait must stay pure. `EccConfig` can live in ecc-domain or ecc-ports. Since it references `LogLevel` (domain type), and ports already depend on nothing domain... Actually, ports don't depend on domain. Let me re-check.

Looking at `ecc-ports/Cargo.toml`: it depends only on `thiserror` and `serde_json`. No domain dep. The `EccConfig` struct needs `LogLevel`. Options:
1. Put `EccConfig` in ecc-domain (pure data, no I/O)
2. Add ecc-domain dep to ecc-ports (violates direction: ports should not depend on domain)
3. Use a string for log_level in EccConfig at the port level, convert in app layer

Option 1 is cleanest: `EccConfig` is a value object, belongs in domain alongside `LogLevel`. The port trait references it.

But then ecc-ports needs to reference ecc-domain types in the trait signature. Currently ecc-ports has no ecc-domain dep. Let me check the architecture rules again...

From `crates/CLAUDE.md`: `ecc-app → ecc-ports ← ecc-infra`, `ecc-app → ecc-domain`. Ports don't depend on domain. This is the standard hexagonal pattern where ports are independent of domain.

So option 3: `ConfigStore` trait uses `Option<String>` for log_level at the port level. The app layer converts to/from `LogLevel`. This keeps ports domain-free.

Actually, a cleaner approach: define `EccConfig` as a simple data struct in ecc-ports (just strings/Options), and the domain `LogLevel` conversion happens in the app layer. The `ConfigStore` trait deals with raw config values.

File: `crates/ecc-ports/src/config_store.rs`
```rust
use std::path::Path;

/// Raw configuration values as persisted in config.toml.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RawEccConfig {
    pub log_level: Option<String>,
}

pub trait ConfigStore: Send + Sync {
    fn load_global(&self) -> Result<RawEccConfig, ConfigError>;
    fn load_local(&self, project_dir: &Path) -> Result<RawEccConfig, ConfigError>;
    fn save_global(&self, config: &RawEccConfig) -> Result<(), ConfigError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config I/O error at {0}: {1}")]
    Io(std::path::PathBuf, String),
    #[error("config parse error at {0}: {1}")]
    Parse(std::path::PathBuf, String),
    #[error("config serialization error: {0}")]
    Serialize(String),
    #[error("config file not found: {0}")]
    NotFound(std::path::PathBuf),
}
```

Merge logic (local wins per-key) lives in the app layer, not in the port.

Risk: Low
Dependencies: None

**Step 1.3**: Add `InMemoryConfigStore` test double

File: `crates/ecc-test-support/src/in_memory_config_store.rs`
- Implement `ConfigStore` backed by `RefCell<HashMap<String, RawEccConfig>>` with "global" and "local" keys
- Builder methods: `with_global(config)`, `with_local(config)`

File: `crates/ecc-test-support/src/lib.rs`
- Add `pub mod in_memory_config_store;` and re-export

File: `crates/ecc-test-support/Cargo.toml`
- Already depends on `ecc-ports`; no new deps needed

Risk: Low
Dependencies: Step 1.2

#### Test Targets for Phase 1
- **Unit tests**: `LogLevel::from_str` valid/invalid, `Display`, `Default`
- **Unit tests**: `InMemoryConfigStore` load/save round-trip, not-found behavior
- **Edge cases**: Invalid log level string, case insensitivity ("INFO" vs "info")
- **Expected test files**: `crates/ecc-domain/src/log_level.rs` (inline `#[cfg(test)]`), `crates/ecc-test-support/src/in_memory_config_store.rs` (inline)

#### Pass Conditions
| PC | AC | Type | Assertion |
|----|-----|------|-----------|
| PC-001 | AC-001.5 | `cargo test` | `LogLevel` has no tracing imports; `grep -r "tracing" crates/ecc-domain/src/log_level.rs` returns empty |
| PC-002 | AC-005.9 | `cargo test` | `InMemoryConfigStore::load_global` returns `NotFound` when empty; returns config after `save_global` |
| PC-003 | AC-005.7 | `cargo test` | `LogLevel::from_str("invalid")` returns `Err` with message listing valid levels |

#### Commit Cadence
1. `test: add LogLevel and ConfigStore port tests (RED)`
2. `feat: add LogLevel value object, ConfigStore port, InMemoryConfigStore (GREEN)`
3. `refactor: improve Phase 1` (if needed)

---

### Phase 2: Crate Migration — ecc-infra

**Layers: [Adapter]**

**Step 2.1**: Replace `log` with `tracing` in ecc-infra

File: `crates/ecc-infra/Cargo.toml`
- Remove `log = "0.4"`
- Add `tracing = "0.1"`

File: `crates/ecc-infra/src/rustyline_input.rs`
- Replace `log::warn!(...)` with `tracing::warn!(...)`

Risk: Low — only 4 call sites in one file
Dependencies: None (but Phase 1 for ConfigStore dep)

**Step 2.2**: Add `FileConfigStore` adapter

File: `crates/ecc-infra/Cargo.toml`
- Add `toml = "0.8"`
- Add `serde = { workspace = true }` (already available? No, not in current deps — add it)
- Add `ecc-domain = { workspace = true }` — wait, infra depends on ports not domain. Actually infra CAN depend on domain if needed. Let me check... No, `ecc-infra` currently only depends on `ecc-ports` and `ecc-flock`. The `FileConfigStore` implements `ConfigStore` (from ports) and only deals with `RawEccConfig` (from ports). No domain dep needed.

File: `crates/ecc-infra/Cargo.toml`
- Add `toml = "0.8"`
- Add `serde = { workspace = true }` (for deserializing TOML into internal structs)

File: `crates/ecc-infra/src/file_config_store.rs`
```rust
/// Production config store backed by TOML files.
/// Global: ~/.ecc/config.toml
/// Local: <project_dir>/.ecc/config.toml
pub struct FileConfigStore {
    home_dir: PathBuf,
}
```
- `load_global`: read `~/.ecc/config.toml`, parse TOML, map to `RawEccConfig`. Return `NotFound` if file missing.
- `load_local`: read `<project_dir>/.ecc/config.toml`, same pattern.
- `save_global`: serialize to TOML, atomic write (mktemp + rename in same dir), create `~/.ecc/` if missing.
- Internal TOML struct: `#[derive(Serialize, Deserialize)] struct ConfigToml { log_level: Option<String> }` with `#[serde(default)]`
- Malformed TOML: return `ConfigError::Parse` (caller decides to warn and use default)

File: `crates/ecc-infra/src/lib.rs`
- Add `pub mod file_config_store;`

Risk: Medium — atomic write + directory creation
Dependencies: Step 1.2

#### Test Targets for Phase 2
- **Unit tests**: `FileConfigStore` round-trip (save then load), missing file returns `NotFound`, malformed TOML returns `Parse` error
- **Integration tests**: `#[ignore]` tests with real filesystem via tempdir
- **Edge cases**: `~/.ecc/` missing on save (auto-create), concurrent write safety (atomic rename), invalid TOML content
- **Expected test files**: `crates/ecc-infra/src/file_config_store.rs` (inline `#[cfg(test)]`)

#### Pass Conditions
| PC | AC | Type | Assertion |
|----|-----|------|-----------|
| PC-004 | AC-001.4 | `grep` | `grep -r "log = " crates/ecc-infra/Cargo.toml` returns empty |
| PC-005 | AC-005.8 | `cargo test` | `FileConfigStore::save_global` creates `~/.ecc/` if missing (tempdir test) |
| PC-006 | AC-005.10 | `cargo test` | `FileConfigStore::load_global` with malformed TOML returns `ConfigError::Parse` |
| PC-007 | AC-005.11 | `cargo test` | Saved file is written atomically (verify via tempfile existence pattern) |

#### Commit Cadence
1. `test: add FileConfigStore and tracing migration tests (RED)`
2. `feat: migrate ecc-infra to tracing, add FileConfigStore (GREEN)`
3. `refactor: improve Phase 2` (if needed)

---

### Phase 3: Crate Migration — ecc-app + Instrumentation

**Layers: [UseCase]**

**Step 3.1**: Replace `log` with `tracing` in ecc-app

File: `crates/ecc-app/Cargo.toml`
- Remove `log = "0.4"`
- Remove `testing_logger = "0.1"` from dev-dependencies
- Add `tracing = "0.1"`

All ~15 source files with `log::warn!`, `log::debug!`, `log::error!`:
- Mechanical replacement: `log::warn!` -> `tracing::warn!`, `log::debug!` -> `tracing::debug!`, etc.
- Replace `use log::warn;` -> `use tracing::warn;`
- Files to touch (from grep above):
  - `src/hook/handlers/tier1_simple/dev_hooks.rs`
  - `src/hook/handlers/tier2_notify.rs`
  - `src/hook/handlers/tier2_tools/quality.rs`
  - `src/hook/handlers/tier2_tools/formatting.rs`
  - `src/hook/handlers/tier3_session/helpers.rs`
  - `src/hook/handlers/tier3_session/lifecycle.rs`
  - `src/hook/handlers/tier3_session/tracking.rs`
  - `src/hook/handlers/tier3_session/logging.rs`
  - `src/hook/handlers/tier3_session/daily.rs`
  - `src/hook/handlers/tier3_session/compact.rs`
  - `src/hook/handlers/tier3_session/reflection.rs`
  - `src/hook/handlers/tier3_session/mod.rs`
  - `src/session/aliases/mod.rs`
  - `src/backlog.rs`
  - `src/merge/mod.rs`
  - `src/config/gitignore.rs`
  - `src/config/merge.rs`
  - `src/install/global/steps.rs`
  - `src/install/init.rs`
  - `src/install/helpers/settings.rs`
  - `src/install/helpers/artifacts.rs`
  - `src/dev/switch.rs`
  - `src/claw/mod.rs`
  - `src/claw/handlers/session.rs`

File: `crates/ecc-app/tests/observable_logging.rs`
- Update the check to look for `tracing::warn!` or `warn!(` instead of `log::warn!`
- Update the PC-004 doc comment

Risk: Low — mechanical, no logic changes
Dependencies: Phase 2 (infra migration must be done first so crate compiles)

**Step 3.2**: Add tracing instrumentation to hook dispatch

File: `crates/ecc-app/src/hook/mod.rs`
- In `dispatch()`: add `tracing::debug!(hook_id = %ctx.hook_id, "dispatching hook")` at entry
- On disabled hook: `tracing::debug!(hook_id = %ctx.hook_id, "hook disabled, passthrough")`
- On unknown hook: `tracing::debug!(hook_id = %ctx.hook_id, "unknown hook")`
- Add `tracing::trace!(payload_len = stdin.len(), "hook stdin payload")` for full payload logging

Risk: Low
Dependencies: Step 3.1

**Step 3.3**: Add diagnostics use case

File: `crates/ecc-app/src/diagnostics.rs`
- `DiagnosticSnapshot` struct: version, phase, feature, artifacts (spec/design/tasks present), component_counts (agents, skills, commands, rules, hooks), config_path
- `ComponentCounts` struct: agents, skills, commands, rules, hooks (all `usize`)
- `fn gather_diagnostics(fs: &dyn FileSystem, env: &dyn Environment) -> DiagnosticSnapshot`
  - Version from `version::version()`
  - Workflow state: read `.claude/workflow/state.json` via fs port
  - Component counts: count files in `~/.claude/agents/`, `~/.claude/skills/`, `~/.claude/commands/`, `~/.claude/rules/`, count entries in `~/.claude/settings.json` hooks array
  - Config path: check `~/.ecc/config.toml` existence
  - If `~/.claude/` missing: return snapshot with "not installed" flag
- `fn format_human(snapshot: &DiagnosticSnapshot) -> String` — key-value lines per AC-004.1
- `fn format_json(snapshot: &DiagnosticSnapshot) -> String` — serde_json::to_string_pretty

File: `crates/ecc-app/src/lib.rs`
- Add `pub mod diagnostics;`

Risk: Medium — multiple file reads, needs defensive error handling
Dependencies: Step 3.1

**Step 3.4**: Add config use case

File: `crates/ecc-app/src/config_cmd.rs`
- `fn set_log_level(config_store: &dyn ConfigStore, level: &str) -> Result<(), ConfigCmdError>`
  - Parse level string via `LogLevel::from_str`
  - Load current global config, update log_level field, save
  - Return error with valid levels on invalid input
- `fn resolve_log_level(config_store: &dyn ConfigStore, project_dir: &Path, cli_level: Option<LogLevel>, env_ecc_log: Option<&str>, env_rust_log: Option<&str>) -> LogLevel`
  - Precedence: cli_level > env_ecc_log > env_rust_log > config (merged) > default(Warn)

File: `crates/ecc-app/src/lib.rs`
- Add `pub mod config_cmd;`

Risk: Low
Dependencies: Step 1.1 (LogLevel), Step 1.2 (ConfigStore)

#### Test Targets for Phase 3
- **Unit tests**: `dispatch()` emits debug event (use `tracing-test` crate subscriber capture), `gather_diagnostics` with mocked fs, `set_log_level` valid/invalid, `resolve_log_level` precedence chain
- **Edge cases**: `~/.claude/` missing, workflow state.json corrupt, config merge local-wins-per-key
- **Expected test files**: `crates/ecc-app/src/hook/mod.rs` (extend existing tests), `crates/ecc-app/src/diagnostics.rs` (inline), `crates/ecc-app/src/config_cmd.rs` (inline)

#### Pass Conditions
| PC | AC | Type | Assertion |
|----|-----|------|-----------|
| PC-008 | AC-001.3 | `grep` | `grep -rn "log::" crates/ecc-app/src/` returns only comments/docs, not code |
| PC-009 | AC-001.4 | `grep` | `grep "log = " crates/ecc-app/Cargo.toml` returns empty |
| PC-010 | AC-003.1 | `cargo test` | `dispatch()` test with tracing subscriber captures debug event with hook_id field |
| PC-011 | AC-003.7 | `cargo test` | `dispatch()` test captures trace event with full payload |
| PC-012 | AC-004.1 | `cargo test` | `format_human` includes "ECC", "Phase:", "Feature:", "Components:", "Config:" lines |
| PC-013 | AC-004.2 | `cargo test` | `gather_diagnostics` with no state.json returns "No active workflow" |
| PC-014 | AC-004.3 | `cargo test` | `gather_diagnostics` with missing `~/.claude/` returns "ECC not installed" |
| PC-015 | AC-004.4 | `cargo test` | `format_json` output parses as valid JSON |
| PC-016 | AC-004.5 | `cargo test` | diagnostics function signature takes `&dyn FileSystem` and `&dyn Environment` |
| PC-017 | AC-005.1 | `cargo test` | `set_log_level("info")` succeeds and persists via InMemoryConfigStore |
| PC-018 | AC-005.2 | `cargo test` | `resolve_log_level` with global=warn and local=debug returns debug |
| PC-019 | AC-005.3 | `cargo test` | `resolve_log_level` with config=info and no flags/env returns info |
| PC-020 | AC-005.4 | `cargo test` | `resolve_log_level` with config=info and ecc_log=debug returns debug |
| PC-021 | AC-005.5 | `cargo test` | `resolve_log_level` with config=info and cli=trace returns trace |
| PC-022 | AC-005.6 | `cargo test` | Full precedence chain test: cli > ECC_LOG > RUST_LOG > config > default |
| PC-023 | AC-005.7 | `cargo test` | `set_log_level("invalid")` returns error listing valid levels |

#### Commit Cadence
1. `test: add ecc-app tracing migration and use case tests (RED)`
2. `feat: migrate ecc-app to tracing, add diagnostics and config use cases (GREEN)`
3. `refactor: improve Phase 3` (if needed)

---

### Phase 4: Binary — ecc-cli

**Layers: [Framework]**

**Step 4.1**: Replace log/env_logger with tracing/tracing-subscriber in ecc-cli

File: `crates/ecc-cli/Cargo.toml`
- Remove `log = "0.4"` and `env_logger = "0.11"`
- Add `tracing = "0.1"`
- Add `tracing-subscriber = { version = "0.3", features = ["fmt", "env-filter", "ansi"] }`

File: `crates/ecc-cli/src/main.rs`
- Remove `--verbose` flag (AC-002.9)
- Add verbosity flags:
  ```rust
  /// Increase verbosity (-v info, -vv debug, -vvv trace)
  #[arg(short, long, action = clap::ArgAction::Count, global = true, conflicts_with = "quiet")]
  verbose: u8,

  /// Suppress all output except errors
  #[arg(short, long, global = true, conflicts_with = "verbose")]
  quiet: bool,
  ```
- Replace env_logger init with tracing-subscriber init:
  ```rust
  fn init_tracing(cli_verbose: u8, cli_quiet: bool, config_store: &dyn ConfigStore, project_dir: Option<&Path>) {
      let level = resolve_log_level(config_store, project_dir, cli_level, ecc_log, rust_log);
      let filter = EnvFilter::new(level.to_string());
      tracing_subscriber::fmt()
          .with_env_filter(filter)
          .with_writer(std::io::stderr)
          .init();
  }
  ```
- Map cli flags to LogLevel: quiet=Error, 0=None(use config/env/default), 1=Info, 2=Debug, 3=Trace
- ECC_LOG env var check before RUST_LOG
- Conflicts_with ensures -v and -q are mutually exclusive (clap error, AC-002.8)

File: `crates/ecc-cli/src/commands/version.rs`
- Replace `log::debug!` with `tracing::debug!`

File: `crates/ecc-cli/src/commands/install.rs`
- Replace `log::warn!` with `tracing::warn!`

Risk: Medium — subscriber initialization is the core behavior change
Dependencies: Phase 3

**Step 4.2**: Add `ecc status` command

File: `crates/ecc-cli/src/commands/status.rs` (new)
```rust
pub struct StatusArgs {
    #[arg(long)]
    json: bool,
}

pub fn run(args: StatusArgs) -> anyhow::Result<()> {
    let fs = OsFileSystem;
    let env = OsEnvironment;
    let snapshot = diagnostics::gather_diagnostics(&fs, &env);
    let output = if args.json {
        diagnostics::format_json(&snapshot)
    } else {
        diagnostics::format_human(&snapshot)
    };
    println!("{output}");
    Ok(())
}
```

File: `crates/ecc-cli/src/main.rs`
- Add `Status(commands::status::StatusArgs)` variant to `Command` enum

File: `crates/ecc-cli/src/commands/mod.rs`
- Add `pub mod status;`

Risk: Low
Dependencies: Step 3.3 (diagnostics use case)

**Step 4.3**: Add `ecc config set` command

File: `crates/ecc-cli/src/commands/config.rs` (new)
```rust
#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    action: ConfigAction,
}

#[derive(Subcommand)]
enum ConfigAction {
    Set {
        key: String,   // only "log-level" supported in v1
        value: String,
    },
}
```
- Wire `FileConfigStore` and call `config_cmd::set_log_level`
- Print confirmation on success

File: `crates/ecc-cli/src/main.rs`
- Add `Config(commands::config::ConfigArgs)` variant

File: `crates/ecc-cli/src/commands/mod.rs`
- Add `pub mod config;`

Risk: Low
Dependencies: Step 3.4, Step 2.2

#### Test Targets for Phase 4
- **Unit tests**: CLI flag parsing (clap `try_parse_from` tests): default=warn, -v=info, -vv=debug, -vvv=trace, -q=error, -v -q=error (conflict)
- **Integration tests**: `ecc status` output format, `ecc config set log-level info` persistence
- **Edge cases**: `--verbose` rejected, ECC_LOG overrides -v, RUST_LOG fallback
- **Expected test files**: `crates/ecc-cli/src/main.rs` (inline), `crates/ecc-cli/src/commands/status.rs` (inline), `crates/ecc-cli/src/commands/config.rs` (inline)

#### Pass Conditions
| PC | AC | Type | Assertion |
|----|-----|------|-----------|
| PC-024 | AC-001.1 | `cargo test` | ecc binary initializes tracing-subscriber writing to stderr with EnvFilter |
| PC-025 | AC-001.4 | `grep` | `grep "env_logger" crates/ecc-cli/Cargo.toml` returns empty |
| PC-026 | AC-002.1 | `cargo test` | Default (no flags/env) resolves to warn |
| PC-027 | AC-002.2 | `cargo test` | `-q` resolves to error |
| PC-028 | AC-002.3 | `cargo test` | `-v` resolves to info |
| PC-029 | AC-002.4 | `cargo test` | `-vv` resolves to debug |
| PC-030 | AC-002.5 | `cargo test` | `-vvv` resolves to trace |
| PC-031 | AC-002.6 | `cargo test` | `ECC_LOG=info` overrides default |
| PC-032 | AC-002.7 | `cargo test` | `ECC_LOG=debug` + `-v` -> ECC_LOG wins (debug) |
| PC-033 | AC-002.8 | `cargo test` | `-v -q` returns clap error |
| PC-034 | AC-002.9 | `cargo test` | `--verbose` returns clap error (unrecognized) |
| PC-035 | AC-002.10 | `cargo test` | `RUST_LOG=debug` with no ECC_LOG -> debug |

#### Commit Cadence
1. `test: add ecc-cli tracing migration and command tests (RED)`
2. `feat: migrate ecc-cli to tracing, add status and config commands (GREEN)`
3. `refactor: improve Phase 4` (if needed)

---

### Phase 5: Binary — ecc-workflow + Instrumentation

**Layers: [Framework, UseCase]**

**Step 5.1**: Migrate ecc-workflow to tracing

File: `crates/ecc-workflow/Cargo.toml`
- Remove `log = "0.4"` and `env_logger = "0.11"`
- Add `tracing = "0.1"`
- Add `tracing-subscriber = { version = "0.3", features = ["fmt", "env-filter", "ansi"] }`
- Add `ecc-infra = { workspace = true }` (for FileConfigStore)
- Add `ecc-ports = { workspace = true }` (for ConfigStore trait)

File: `crates/ecc-workflow/src/main.rs`
- Add same `-v`/`-vv`/`-vvv`/`-q` flags as ecc-cli
- Replace `env_logger::init()` with tracing-subscriber init (same pattern as ecc-cli)
- Replace `log::debug!` with `tracing::debug!`
- ECC_WORKFLOW_BYPASS=1 check BEFORE tracing init (already correct position)

File: `crates/ecc-workflow/src/io.rs`
- Replace `log::warn!` with `tracing::warn!`

Risk: Medium — new ecc-infra dependency changes the crate graph
Dependencies: Phase 2 (FileConfigStore), Phase 4 pattern

**Step 5.2**: Add tracing instrumentation to workflow commands

File: `crates/ecc-workflow/src/commands/phase_gate.rs`
- Add `tracing::info!(phase = %..., tool = %..., verdict = %..., "phase-gate decision")` (AC-003.2)

File: `crates/ecc-workflow/src/commands/transition.rs`
- Add `tracing::info!(from_phase = %..., to_phase = %..., feature = %..., "workflow transition")` (AC-003.3)

File: `crates/ecc-workflow/src/commands/memory_write.rs`
- Add `tracing::debug!(memory_type = %kind, success = true, "memory write")` (AC-003.4)

File: `crates/ecc-workflow/src/main.rs` (dispatch fn)
- Existing `log::debug!` already replaced in Step 5.1
- Add `tracing::info!(session_id = %..., "session start")` in relevant session paths (AC-003.5)

Lock instrumentation (AC-003.6) — if ecc-flock is used:
File: `crates/ecc-workflow/src/commands/backlog.rs` or wherever flock is acquired
- Add `tracing::debug!("acquiring lock")` before and `tracing::debug!("lock released")` after

Stdin parsing (AC-003.7):
File: `crates/ecc-workflow/src/main.rs` or `crates/ecc-workflow/src/io.rs`
- Add `tracing::trace!(payload = %..., "hook stdin parsed")` at stdin read site

Risk: Low — adding trace events, no logic changes
Dependencies: Step 5.1

#### Test Targets for Phase 5
- **Unit tests**: CLI flag parsing for ecc-workflow, tracing init
- **Unit tests**: phase-gate emits info event, transition emits info event, memory-write emits debug event
- **Edge cases**: ECC_WORKFLOW_BYPASS=1 exits before tracing init
- **Expected test files**: `crates/ecc-workflow/src/main.rs` (extend existing), individual command test modules

#### Pass Conditions
| PC | AC | Type | Assertion |
|----|-----|------|-----------|
| PC-036 | AC-001.2 | `cargo test` | ecc-workflow binary initializes tracing-subscriber with EnvFilter |
| PC-037 | AC-001.4 | `grep` | `grep "env_logger" crates/ecc-workflow/Cargo.toml` returns empty; `grep "log = " crates/ecc-workflow/Cargo.toml` returns empty |
| PC-038 | AC-003.2 | `cargo test` | Phase-gate emits info event with phase, tool, verdict fields |
| PC-039 | AC-003.3 | `cargo test` | Transition emits info event with from_phase, to_phase, feature fields |
| PC-040 | AC-003.4 | `cargo test` | Memory write emits debug event with memory_type and success fields |
| PC-041 | AC-003.5 | `cargo test` | Session start emits info event with session_id |
| PC-042 | AC-003.6 | `cargo test` | Lock acquire/release emit debug events |
| PC-043 | AC-003.7 | `cargo test` | Stdin parsing emits trace event with payload content |

#### Commit Cadence
1. `test: add ecc-workflow tracing and instrumentation tests (RED)`
2. `feat: migrate ecc-workflow to tracing, add instrumentation (GREEN)`
3. `refactor: improve Phase 5` (if needed)

---

### Phase 6: ADR + Documentation + Final Gate

**Layers: [Docs]**

**Step 6.1**: Write ADR-0032

File: `docs/adr/0032-tracing-cross-cutting.md`
- Status: Accepted (2026-03-30)
- Context: ECC needs structured diagnostics; tracing is a cross-cutting concern that must not pollute domain/ports
- Decision: tracing facade permitted only in ecc-app, ecc-infra, ecc-cli, ecc-workflow. Forbidden in ecc-domain and ecc-ports. Subscriber wiring only in binary main.rs.
- Consequences: domain hook enforcement checks for tracing imports; new code in domain/ports must not add tracing dependency

**Step 6.2**: Update CLAUDE.md

File: `CLAUDE.md`
- Add `ecc status` and `ecc config set` to CLI Commands section
- Add `-v/-vv/-vvv/-q` flag documentation
- Update test count

**Step 6.3**: Final build + clippy gate

- `cargo clippy --workspace -- -D warnings` must pass
- `cargo build --workspace` must pass
- `cargo test --workspace` must pass
- `grep -rn "log::" crates/` returns only comments, doc strings, and test assertions (no code usage)
- `grep "env_logger" crates/*/Cargo.toml` returns empty
- `grep -rn "tracing" crates/ecc-domain/src/` returns empty
- `grep -rn "tracing" crates/ecc-ports/src/` returns empty

#### Pass Conditions
| PC | AC | Type | Assertion |
|----|-----|------|-----------|
| PC-044 | AC-006.1 | `grep` | ADR-0032 file exists and contains "forbidden" + "ecc-domain" + "ecc-ports" |
| PC-045 | AC-006.2 | `grep` | ADR-0032 contains "enforcement" or "hook" in consequences section |
| PC-046 | AC-001.3 | `grep` | `grep -rn "^use log" crates/*/src/**/*.rs` returns empty |
| PC-047 | AC-001.4 | `grep` | `grep "env_logger" crates/*/Cargo.toml` returns empty |
| PC-048 | AC-001.5 | `grep` | `grep -rn "tracing" crates/ecc-domain/src/` returns empty |
| PC-049 | — | `cargo` | `cargo clippy --workspace -- -D warnings` exits 0 |
| PC-050 | — | `cargo` | `cargo build --workspace` exits 0 |
| PC-051 | — | `cargo` | `cargo test --workspace` exits 0 |

#### Commit Cadence
1. `docs: add ADR-0032 tracing cross-cutting concern`
2. `docs: update CLAUDE.md with status and config commands`
3. `chore: final clippy + build verification`

---

## Workspace Cargo.toml Changes

File: `Cargo.toml` (workspace root)
- Add to `[workspace.dependencies]`:
  ```toml
  tracing = "0.1"
  tracing-subscriber = { version = "0.3", features = ["fmt", "env-filter", "ansi"] }
  toml = "0.8"
  ```

This is done in Phase 2 (first phase that needs these deps) and used by subsequent phases.

## E2E Assessment

- **Touches user-facing flows?** Yes — CLI flags, new commands (`ecc status`, `ecc config set`)
- **Crosses 3+ modules end-to-end?** Yes — domain (LogLevel) -> ports (ConfigStore) -> infra (FileConfigStore) -> app (diagnostics, config_cmd) -> cli (commands)
- **New E2E tests needed?** Yes, but lightweight
- **E2E scenarios** (after Phase 5):
  1. `ecc status` outputs version, component counts (run against real `~/.claude/` install)
  2. `ecc config set log-level info` persists to `~/.ecc/config.toml`, subsequent `ecc status` shows config path
  3. `ecc -v version` outputs debug-level tracing to stderr
  4. `ecc -q version` suppresses all but error output

These are `#[ignore]` integration tests in `crates/ecc-integration-tests/`.

## Testing Strategy

- **Unit tests**: LogLevel parsing, ConfigStore round-trip, diagnostics gathering with mocked ports, precedence chain, CLI flag parsing via `try_parse_from`
- **Integration tests**: FileConfigStore with tempdir, tracing subscriber capture via `tracing-test` crate
- **E2E tests**: `#[ignore]` tests running real binaries (ecc status, ecc config set)
- **Static analysis**: grep-based PCs for import bans (no tracing in domain/ports, no log/env_logger anywhere)
- **Tracing event capture**: Add `tracing-test = "0.2"` to dev-dependencies of ecc-app and ecc-workflow for subscriber capture in unit tests

## New Dependencies Summary

| Crate | Add | Remove |
|-------|-----|--------|
| Workspace | `tracing = "0.1"`, `tracing-subscriber = { "0.3", features = [...] }`, `toml = "0.8"` | — |
| ecc-domain | — | — |
| ecc-ports | — | — |
| ecc-infra | `tracing = { workspace = true }`, `toml = { workspace = true }`, `serde = { workspace = true }` | `log = "0.4"` |
| ecc-app | `tracing = { workspace = true }` | `log = "0.4"` |
| ecc-app (dev) | `tracing-test = "0.2"` | `testing_logger = "0.1"` |
| ecc-cli | `tracing = { workspace = true }`, `tracing-subscriber = { workspace = true }` | `log = "0.4"`, `env_logger = "0.11"` |
| ecc-workflow | `tracing = { workspace = true }`, `tracing-subscriber = { workspace = true }`, `ecc-infra = { workspace = true }`, `ecc-ports = { workspace = true }` | `log = "0.4"`, `env_logger = "0.11"` |
| ecc-test-support | — | — |

## File Changes Summary

| # | File | Action | Phase |
|---|------|--------|-------|
| 1 | `Cargo.toml` (workspace) | Edit — add workspace deps | 2 |
| 2 | `crates/ecc-domain/src/log_level.rs` | Create | 1 |
| 3 | `crates/ecc-domain/src/lib.rs` | Edit — add module | 1 |
| 4 | `crates/ecc-ports/src/config_store.rs` | Create | 1 |
| 5 | `crates/ecc-ports/src/lib.rs` | Edit — add module | 1 |
| 6 | `crates/ecc-test-support/src/in_memory_config_store.rs` | Create | 1 |
| 7 | `crates/ecc-test-support/src/lib.rs` | Edit — add module + re-export | 1 |
| 8 | `crates/ecc-infra/Cargo.toml` | Edit — swap log->tracing, add toml+serde | 2 |
| 9 | `crates/ecc-infra/src/rustyline_input.rs` | Edit — log->tracing | 2 |
| 10 | `crates/ecc-infra/src/file_config_store.rs` | Create | 2 |
| 11 | `crates/ecc-infra/src/lib.rs` | Edit — add module | 2 |
| 12 | `crates/ecc-app/Cargo.toml` | Edit — swap log->tracing, swap testing_logger->tracing-test | 3 |
| 13-35 | `crates/ecc-app/src/**/*.rs` (~23 files) | Edit — log->tracing mechanical replacement | 3 |
| 36 | `crates/ecc-app/src/diagnostics.rs` | Create | 3 |
| 37 | `crates/ecc-app/src/config_cmd.rs` | Create | 3 |
| 38 | `crates/ecc-app/src/lib.rs` | Edit — add modules | 3 |
| 39 | `crates/ecc-app/src/hook/mod.rs` | Edit — add tracing instrumentation | 3 |
| 40 | `crates/ecc-app/tests/observable_logging.rs` | Edit — update to tracing | 3 |
| 41 | `crates/ecc-cli/Cargo.toml` | Edit — swap log/env_logger->tracing/tracing-subscriber | 4 |
| 42 | `crates/ecc-cli/src/main.rs` | Edit — rewrite init, add flags, remove --verbose | 4 |
| 43 | `crates/ecc-cli/src/commands/version.rs` | Edit — log->tracing | 4 |
| 44 | `crates/ecc-cli/src/commands/install.rs` | Edit — log->tracing | 4 |
| 45 | `crates/ecc-cli/src/commands/status.rs` | Create | 4 |
| 46 | `crates/ecc-cli/src/commands/config.rs` | Create | 4 |
| 47 | `crates/ecc-cli/src/commands/mod.rs` | Edit — add modules | 4 |
| 48 | `crates/ecc-workflow/Cargo.toml` | Edit — swap deps, add ecc-infra+ecc-ports | 5 |
| 49 | `crates/ecc-workflow/src/main.rs` | Edit — rewrite init, add flags | 5 |
| 50 | `crates/ecc-workflow/src/io.rs` | Edit — log->tracing | 5 |
| 51 | `crates/ecc-workflow/src/commands/phase_gate.rs` | Edit — add instrumentation | 5 |
| 52 | `crates/ecc-workflow/src/commands/transition.rs` | Edit — add instrumentation | 5 |
| 53 | `crates/ecc-workflow/src/commands/memory_write.rs` | Edit — add instrumentation | 5 |
| 54 | `docs/adr/0032-tracing-cross-cutting.md` | Create | 6 |
| 55 | `CLAUDE.md` | Edit — add commands, update test count | 6 |

## Risks & Mitigations

- **Risk**: ecc-workflow gaining ecc-infra dependency creates a larger binary
  - Mitigation: ecc-infra is already small; only FileConfigStore + OsEnvironment are needed. Feature-gate if size becomes concern.

- **Risk**: `tracing-test` crate compatibility with tracing-subscriber version
  - Mitigation: Pin compatible versions; `tracing-test 0.2` works with `tracing 0.1`

- **Risk**: Observable logging test (`tests/observable_logging.rs`) breaks during migration
  - Mitigation: Update in same commit as ecc-app migration (Step 3.1); search for `tracing::warn!` instead of `log::warn!`

- **Risk**: Hook stderr output format changes (tracing-subscriber format differs from env_logger)
  - Mitigation: Hook protocol stderr messages are direct `eprintln!` writes, not logging calls. Tracing output goes to stderr but with different format prefix. This is acceptable for a dev tool.

- **Risk**: Concurrent `ecc config set` corrupts config file
  - Mitigation: Atomic write via mktemp + rename (same pattern as ecc-workflow state.json)

## Success Criteria

- [ ] All `log`/`env_logger` imports removed from workspace (AC-001.3, AC-001.4)
- [ ] Zero tracing imports in ecc-domain and ecc-ports (AC-001.5)
- [ ] Both binaries initialize tracing-subscriber to stderr (AC-001.1, AC-001.2)
- [ ] `-v`/`-vv`/`-vvv`/`-q` flags work with correct level mapping (AC-002.1-002.5)
- [ ] ECC_LOG > RUST_LOG > config > default precedence (AC-002.6-002.10, AC-005.3-005.6)
- [ ] Hook dispatch, phase-gate, transitions, memory writes emit structured events (AC-003.1-003.7)
- [ ] `ecc status` shows diagnostic snapshot in human and JSON formats (AC-004.1-004.5)
- [ ] `ecc config set log-level` persists to `~/.ecc/config.toml` (AC-005.1-005.11)
- [ ] ADR-0032 documents tracing ruling (AC-006.1-006.2)
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] `cargo build --workspace` passes

## Rollback Plan

Reverse dependency order (outermost first, innermost last):

1. **Phase 6 (Docs)**: `git revert` the ADR-0032 + CLAUDE.md + CHANGELOG commits
2. **Phase 5 (ecc-workflow)**: Revert subscriber init, -v/-q flags, instrumentation commits. Remove ecc-infra dep from ecc-workflow/Cargo.toml
3. **Phase 4 (ecc-cli)**: Revert subscriber init, flags, status/config commands
4. **Phase 3c (instrumentation)**: Revert dispatch debug!, phase-gate info! additions
5. **Phase 3b (use cases)**: Delete diagnostics.rs and config_cmd.rs
6. **Phase 3a (app migration)**: Revert tracing -> log in all ecc-app files; restore `log` dep
7. **Phase 2 (infra)**: Delete FileConfigStore; revert tracing -> log; restore `log` dep
8. **Phase 1 (domain + ports)**: Delete LogLevel VO, ConfigStore port, InMemoryConfigStore, RawEccConfig; restore Cargo.toml
9. **State cleanup**: Delete `~/.ecc/` directory if created during testing

Each phase's revert is independently viable — partially-applied phases can be rolled back without affecting earlier phases.

## Doc Update Plan (Phase 6)

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | docs/adr/0032-tracing-cross-cutting.md | ADR | Create | Tracing forbidden in domain/ports | US-006 |
| 2 | CLAUDE.md | Project | Modify | Add ecc status + ecc config CLI commands | US-004, US-005 |
| 3 | CHANGELOG.md | Project | Modify | Add diagnostics entry | — |

### Phase 6 Commit Cadence

1. `docs(adr): add ADR 0032 — tracing cross-cutting concern`
2. `docs: update CLAUDE.md with ecc status and ecc config commands`
3. `docs(changelog): add tiered diagnostics entry`

### Additional PC

PC-052: `grep -q "diagnostics\|tiered verbosity\|tracing" CHANGELOG.md` — verifies CHANGELOG entry exists.

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID (uncle-bob) | CLEAN | 0 prescriptions (praised RawEccConfig) |
| Robert's Oath | 1 WARNING | Phase 3 atomicity (addressed: split into 3 sub-commits) |
| Security | CLEAR | 0 findings |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Rollback | 85 | PASS | 9-step plan added round 2 |
| Doc Plan | 85 | PASS | CHANGELOG + PC-052 added round 2 |
| Coverage | 90 | PASS | 52 PCs covering 40 ACs |
| Order | 85 | PASS | Inward-to-outward hexagonal |
| Architecture | 75 | PASS | RawEccConfig preserves port isolation |
| Fragility | 70 | PASS | grep-based PCs inherent to migration |
| Blast Radius | 65 | PASS | 55 files but organized by crate |
| Missing PCs | 80 | PASS | Build gates per phase suggested |

### File Changes Summary

| Layer | Files | Key Changes |
|-------|-------|-------------|
| Domain | 2 | LogLevel VO (new) |
| Ports | 3 | ConfigStore trait, RawEccConfig, InMemoryConfigStore |
| Infra | 3 | FileConfigStore (new), tracing migration |
| App | ~25 | tracing migration, diagnostics, config_cmd, dispatch instrumentation |
| CLI | 5 | subscriber init, -v/-q, status, config commands |
| Workflow | 8 | subscriber init, -v/-q, ecc-infra dep, instrumentation |
| Docs | 3 | ADR-0032, CLAUDE.md, CHANGELOG |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-30-ecc-diagnostics-verbosity/design.md | Full design + phase summary |
