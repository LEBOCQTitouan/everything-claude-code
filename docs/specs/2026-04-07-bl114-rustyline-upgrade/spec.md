# Spec: BL-114 — Upgrade rustyline 15 → 17

**Type:** Refactor (dependency upgrade)
**Date:** 2026-04-07
**Status:** Approved
**Backlog:** BL-114

---

## Problem Statement

rustyline is pinned at v15, two major versions behind the current v17.0.2. The adapter also uses `Result<Self, String>` as its error type, which violates the project's typed-error convention flagged in the 2026-03-28 post-remediation audit as MEDIUM severity. The dependency lag increases future migration cost and misses bug fixes and performance improvements.

### Symptoms

- `Cargo.toml` workspace dependency: `rustyline = "15"` (2 majors behind)
- `RustylineInput::new()` returns `Result<Self, String>` instead of typed `TerminalError`
- Caller in `claw.rs` uses `.map_err(|e| anyhow::anyhow!(e))` workaround for String error

## Research Summary

- Web radar (2026-03-31): Migration effort rated 2/5, maturity 4/5, strategic fit 3/5
- rustyline v16-v17 release notes show no breaking changes to the APIs used (DefaultEditor, readline, history, ReadlineError)
- API surface used: `DefaultEditor::new()`, `load_history()`, `save_history()`, `readline()`, `add_history_entry()`, `ReadlineError::{Interrupted, Eof}`
- No custom completers, highlighters, or validators — minimal exposure to API churn

## Decisions

| # | Decision | Rationale |
|---|----------|-----------|
| D-1 | Fix `Result<String>` → `Result<TerminalError>` during upgrade | Approved by user — clean up while in the file |
| D-2 | Pin to `"17"` (major-level, semver-compatible) | Matches existing style (`rustyline = "15"`) |
| D-3 | Clean up `claw.rs` caller to remove `.map_err` workaround | Approved by user — typed error enables direct `?` |
| D-4 | No new tests required — API is compatible and existing 2 tests cover construction paths | Web research confirms no breaking changes to used APIs |
| D-5 | Use `TerminalError::Io(String)` variant (not a new variant) | `Io` already covers I/O initialization failures; a dedicated `EditorInit` variant is over-engineering for one callsite |
| D-6 | US-2 depends on US-1 (version bump first, then error type fix) | Error type fix is independent of version but logically ordered after bump |

## User Stories

### US-1: Upgrade rustyline dependency

**As a** maintainer,
**I want** rustyline upgraded to v17,
**So that** the project stays current and benefits from upstream fixes.

**Acceptance Criteria:**

- [ ] AC-1.1: `rustyline = "17"` in workspace `Cargo.toml`
- [ ] AC-1.2: `cargo check` passes with no compilation errors
- [ ] AC-1.3: All existing tests pass (`cargo test`)
- [ ] AC-1.4: `cargo clippy -- -D warnings` clean

### US-2: Replace String error with typed TerminalError

**As a** developer,
**I want** `RustylineInput::new()` to return `Result<Self, TerminalError>`,
**So that** error handling follows the project's typed-error convention.

**Acceptance Criteria:**

- [ ] AC-2.1: `RustylineInput::new()` returns `Result<Self, TerminalError>` instead of `Result<Self, String>`
- [ ] AC-2.2: Error wrapping uses `TerminalError::Io(format!(...))` pattern with upstream `Display`
- [ ] AC-2.3: `claw.rs:49` `RustylineInput::new()` caller uses `?` directly without `.map_err` workaround (line 59 `run_repl` is unrelated)
- [ ] AC-2.4: Unit tests in `rustyline_input.rs` still pass

## Affected Modules

| Module | Crate | Impact |
|--------|-------|--------|
| `rustyline_input.rs` | ecc-infra | Version bump + error type change |
| `claw.rs` | ecc-cli | Caller cleanup (remove map_err) |
| `Cargo.toml` | workspace | Version bump |
| `Cargo.lock` | workspace | Auto-updated |

## Constraints

- Must not change the `ReplInput` port trait (hex arch boundary)
- Must not introduce new dependencies
- `TerminalError` must already support the error variant needed (it has `Io(String)`)

## Non-Requirements

- Expanding test coverage beyond API change needs (deferred)
- Switching to reedline (no async REPL requirement)
- Upgrading other deps (BL-115 toml is separate)

## E2E Boundaries

No E2E tests needed — this is a dependency version bump with a minor error type refactor. The REPL adapter is tested via unit tests and the existing NanoClaw integration is manual.

## Doc Impact Assessment

- **CLAUDE.md**: No changes needed
- **MODULE-SUMMARIES.md**: No structural changes
- **ADR**: Not warranted (routine dep upgrade)

## Open Questions

None — scope is clear and API appears compatible.
