# Spec: ECC Diagnostics — Tiered Verbosity

## Problem Statement

ECC hooks fire silently — users can't see what's happening. When things work there's no confirmation; when things break the error context is minimal. The current `log`/`env_logger` setup defaults to `warn` with only 2 `debug!` calls, making verbose mode nearly useless. There's no way to see workflow phase, loaded hooks, or component counts without manually inspecting files.

## Research Summary

- **Standard init pattern**: `tracing_subscriber::fmt()` with `.with_env_filter()` as early as possible in `main()`. Fallback default when env var unset.
- **CLI verbosity mapping**: 0 flags = warn, -v = info, -vv = debug, -vvv = trace, -q = error. Build EnvFilter from matched level.
- **EnvFilter directives**: Comma-separated, per-target granularity (`ecc_app=debug,hyper=warn`).
- **Migration from env_logger**: Replace `env_logger::init()` with `tracing_subscriber::fmt::init()`. `tracing-log` bridge captures third-party `log` facade usage.
- **Library vs binary rule**: Libraries instrument only, never install subscribers. Binary main.rs configures subscriber.
- **Structured output**: `FmtSubscriber` supports Full, Compact, and Json formats.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Full log/env_logger removal (no compat layer) | Only 4 crates use log directly; no third-party deps need bridge | No |
| 2 | ECC_LOG priority, RUST_LOG fallback | Avoids surprises for users with global RUST_LOG | No |
| 3 | -v/-q mutually exclusive (clap error) | Clear, deterministic behavior | No |
| 4 | Remove --verbose (clean break) | ECC is local dev tool; clean break acceptable | No |
| 5 | Config at ~/.ecc/config.toml (global) + project-local | Dedicated ECC directory; local supplements global | No |
| 6 | Add ecc-infra dep to ecc-workflow | Enables shared ConfigStore adapter | No |
| 7 | dispatch() gets debug! for every hook | Cheap, universal observability | No |
| 8 | Keep hook stderr writes as-is | Part of hook protocol contract | No |
| 9 | ecc status reads directly via FileSystem | No subprocess overhead; testable | No |
| 10 | ecc status supports --json | Useful for statusline/CI | No |
| 11 | Tracing forbidden in domain/ports | Cross-cutting concern restricted to app/infra/binary | Yes (ADR-0032) |
| 12 | tracing-subscriber features: fmt, env-filter, ansi | Minimum set for human-readable stderr. No json feature (BL-092). | No |
| 13 | Use `toml` crate for config serialization | Standard, well-maintained, serde-compatible | No |
| 14 | Config v1 has no version field; ignore unknown keys | Simple forward-compat without schema versioning overhead | No |

## User Stories

### US-001: Migrate log/env_logger to tracing

**As a** developer, **I want** all diagnostic output to use `tracing` instead of `log`/`env_logger`, **so that** I get structured, filterable logging.

#### Acceptance Criteria

- AC-001.1: Given ecc binary startup, when tracing-subscriber is initialized, then it writes to stderr with EnvFilter
- AC-001.2: Given ecc-workflow binary startup, when tracing-subscriber is initialized, then it writes to stderr with EnvFilter
- AC-001.3: Given all workspace crates, when compiled, then no `log::` or `env_logger` imports remain
- AC-001.4: Given Cargo.toml files, when inspected, then `log` and `env_logger` are removed; `tracing` and `tracing-subscriber` present
- AC-001.5: Given the domain crate, when inspected, then it has zero tracing imports (forbidden)

#### Dependencies

- Depends on: none

### US-002: Tiered Verbosity via CLI Flags and Env Var

**As a** developer, **I want** to control verbosity with -q/-v/-vv/-vvv and ECC_LOG, **so that** I choose the right detail level.

#### Acceptance Criteria

- AC-002.1: Given no flags/env, when binary runs, then log level is warn
- AC-002.2: Given -q, when binary runs, then log level is error
- AC-002.3: Given -v, when binary runs, then log level is info
- AC-002.4: Given -vv, when binary runs, then log level is debug
- AC-002.5: Given -vvv, when binary runs, then log level is trace
- AC-002.6: Given ECC_LOG=info, when binary runs, then ECC_LOG overrides default
- AC-002.7: Given ECC_LOG and -v both set, when binary runs, then ECC_LOG takes precedence
- AC-002.8: Given both -v and -q, when binary runs, then clap error is returned
- AC-002.9: Given --verbose flag, when binary runs, then it is not recognized (clean break)
- AC-002.10: Given RUST_LOG=debug and no ECC_LOG, when binary runs, then RUST_LOG is used as fallback

#### Dependencies

- Depends on: US-001

### US-003: Instrument Key Handlers and Commands

**As a** developer, **I want** hook decisions, transitions, memory writes, and session events to emit structured tracing events, **so that** I can understand what ECC does with -v.

#### Acceptance Criteria

- AC-003.1: Given dispatch(), when any hook is dispatched, then a debug! event with hook_id and tool_name
- AC-003.2: Given phase-gate decision, when dispatched, then an info!-level event is always emitted with phase, tool, and verdict fields (subscriber filtering determines visibility)
- AC-003.3: Given workflow transition, when executed, then an info!-level event is emitted with from_phase, to_phase, and feature fields
- AC-003.4: Given memory write, when executed, then a debug!-level event is emitted with memory_type and success/failure
- AC-003.5: Given session start, when hook fires, then an info!-level event is emitted with session_id
- AC-003.6: Given lock acquisition, when attempted, then debug!-level events are emitted before and after lock acquire/release
- AC-003.7: Given hook stdin parsing, when executed, then a trace!-level event logs the full JSON payload

#### Dependencies

- Depends on: US-001

### US-004: Extended ecc status Command

**As a** developer, **I want** `ecc status` to show versions, workflow state, component counts, and config, **so that** I get a diagnostic snapshot.

#### Acceptance Criteria

- AC-004.1: Given active workflow, when ecc status runs, then shows key-value lines: "ECC <version>", "Phase: <phase>", "Feature: <feature>", "Artifacts: spec [✓/✗] design [✓/✗] tasks [✓/✗]", "Components: N agents, N skills, N commands, N rules", "Hooks: N installed", "Config: <path>"
- AC-004.2: Given no workflow, when ecc status runs, then shows versions, "No active workflow", component counts
- AC-004.3: Given ~/.claude/ missing, when ecc status runs, then shows versions and "ECC not installed"
- AC-004.4: Given --json flag, when ecc status runs, then output is valid JSON
- AC-004.5: Given the status function, when tested, then uses FileSystem + Environment ports

#### Dependencies

- Depends on: US-001

### US-005: Persistent Verbosity via ecc config

**As a** developer, **I want** `ecc config set log-level <level>` to persist my preference, **so that** I don't set env vars every time.

#### Acceptance Criteria

- AC-005.1: Given `ecc config set log-level info`, when run, then ~/.ecc/config.toml is updated
- AC-005.2: Given project-level .ecc/config.toml, when present, then its keys override matching global keys (merge strategy: local wins per-key, global provides defaults for absent keys)
- AC-005.3: Given persisted config + no flags/env, when binary runs, then persisted level is used
- AC-005.4: Given persisted config + ECC_LOG, when binary runs, then ECC_LOG overrides config
- AC-005.5: Given persisted config + -v flag, when binary runs, then flag overrides config
- AC-005.6: Given precedence: CLI flag > ECC_LOG > RUST_LOG > config > default(warn)
- AC-005.7: Given invalid level, when config set runs, then error with valid levels listed
- AC-005.8: Given ~/.ecc/ missing, when config set runs, then directory is created
- AC-005.9: Given ConfigStore port, when tested, then uses InMemoryConfigStore
- AC-005.10: Given malformed ~/.ecc/config.toml (invalid TOML), when binary starts, then warns to stderr and uses default(warn)
- AC-005.11: Given concurrent `ecc config set` invocations, when both write, then file is not corrupted (atomic write via mktemp + rename)

#### Dependencies

- Depends on: US-001

### US-006: ADR for Tracing Cross-Cutting Concern

**As a** maintainer, **I want** an ADR documenting that tracing is forbidden in domain/ports, **so that** the ruling is recorded.

#### Acceptance Criteria

- AC-006.1: Given ADR-0032, when read, then it explains tracing forbidden in ecc-domain and ecc-ports
- AC-006.2: Given ADR-0032 consequences, then it lists domain hook enforcement checks for tracing imports

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change Type |
|--------|-------|-------------|
| `crates/ecc-cli/` | CLI (binary) | Subscriber init, -v/-q flags, status/config commands |
| `crates/ecc-workflow/` | CLI (binary) | Subscriber init, -v/-q flags, add ecc-infra dep |
| `crates/ecc-app/` | App | Replace log with tracing, instrument dispatch(), diagnostics module |
| `crates/ecc-infra/` | Infra | Replace log with tracing, new FileConfigStore adapter |
| `crates/ecc-ports/` | Ports | New ConfigStore port trait |
| `crates/ecc-domain/` | Domain | LogLevel enum (no tracing dep) |
| Workspace Cargo.toml | Build | Add tracing/tracing-subscriber |

## Constraints

- ecc-domain: zero tracing imports (enforced by hook + ADR)
- All tracing output to stderr, never stdout
- Hook protocol stderr messages unchanged
- ECC_WORKFLOW_BYPASS=1 early exit before tracing init
- Precedence: CLI flag > ECC_LOG > RUST_LOG > config > default(warn)
- Tracing event assertions in tests use `tracing-test` crate or equivalent capturing subscriber
- Migrate one crate fully before moving to the next (no partial migration within a crate)

## Non-Requirements

- No JSON file logging (BL-092)
- No SQLite index (BL-092)
- No tracing on domain functions
- No --log-format json for stderr (defer)
- No OpenTelemetry
- No `ecc config get` or `ecc config list` in v1
- No per-target directive syntax for ECC_LOG in v1 (only level names: error/warn/info/debug/trace)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| CLI stderr | Format change | tracing-subscriber format differs from env_logger |
| ConfigStore (new) | New port + adapter | ~/.ecc/config.toml persistence |
| ecc-workflow deps | Add ecc-infra | Dependency graph change |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New ADR | docs/adr/ | ADR-0032 | Create |
| New commands | CLAUDE.md | CLI section | Add ecc status, ecc config |
| Changelog | CHANGELOG.md | — | Add entry |

## Open Questions

None — all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Include US-5 (persistent config)? | Yes — full feature | Recommended |
| 2 | Config path? | ~/.ecc/config.toml (global) + project-local | Recommended + User |
| 3 | ECC_LOG vs RUST_LOG? | ECC_LOG priority, RUST_LOG fallback | Recommended |
| 4 | -v/-q conflict + --verbose? | Error on conflict; remove --verbose | Recommended |
| 5 | Shared config reader? | Add ecc-infra dep to ecc-workflow | Recommended |
| 6 | Status data source? | Read directly via FileSystem | Recommended |
| 7 | Instrumentation depth? | debug! in dispatch; keep hook stderr as-is | Recommended |
| 8 | ADR + --json? | ADR-0032 yes; --json yes | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Migrate log/env_logger to tracing | 5 | None |
| US-002 | Tiered Verbosity (CLI + env) | 10 | US-001 |
| US-003 | Instrument Key Handlers | 7 | US-001 |
| US-004 | Extended ecc status | 5 | US-001 |
| US-005 | Persistent Verbosity (ecc config) | 11 | US-001 |
| US-006 | ADR-0032 Tracing Ruling | 2 | None |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Decision Completeness | 85 | PASS | 14 decisions including features, crate choices, config versioning |
| Dependency Gaps | 85 | PASS | Clean DAG, US-001 is foundation |
| Scope Creep Risk | 80 | PASS | Non-requirements bound v1 clearly |
| Rollback & Failure | 80 | PASS | All-or-nothing per crate is safe; config is new |
| Testability | 78 | PASS | tracing-test constraint added |
| Edge Cases | 75 | PASS | Config corruption, concurrent writes, merge semantics covered |
| Ambiguity | 75 | PASS | Output format defined, instrumentation vs filtering clarified |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-30-ecc-diagnostics-verbosity/spec.md | Full spec + phase summary |
