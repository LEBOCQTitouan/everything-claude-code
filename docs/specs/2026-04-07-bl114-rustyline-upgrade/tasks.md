# Tasks: BL-114 — Upgrade rustyline 15 → 17

**Feature:** BL-114: Upgrade rustyline 15 to 17
**Generated:** 2026-04-07

## Pass Conditions

| PC | Status | Description | Command |
|----|--------|-------------|---------|
| PC-001 | done | Create editor without history | `cargo test -p ecc-infra create_without_history -- --exact` |
| PC-002 | done | Create editor with temp history | `cargo test -p ecc-infra create_with_temp_history -- --exact` |
| PC-003 | done | Full test suite regression | `cargo test` |
| PC-004 | done | Clippy clean | `cargo clippy -- -D warnings` |
| PC-005 | done | Cargo check | `cargo check` |
| PC-006 | done | Release build | `cargo build --release` |
| PC-007 | done | Format check | `cargo fmt --check` |
| PC-008 | done | No String error type in adapter | `! grep -q 'Result<Self, String>' crates/ecc-infra/src/rustyline_input.rs` |
| PC-009 | done | TerminalError::Io wrapping present | `grep -c 'TerminalError::Io(format!' crates/ecc-infra/src/rustyline_input.rs` |
| PC-010 | done | No map_err workaround in claw.rs | `grep 'RustylineInput::new' crates/ecc-cli/src/commands/claw.rs \| grep -vc map_err` |
| PC-011 | done | CHANGELOG entry exists | `grep -q 'BL-114' CHANGELOG.md` |

## Wave Plan

**Wave 1** (sequential): PC-001 through PC-011 — all files overlap, single execution wave.

## Status Trail

- `2026-04-07T13:20:00Z` — tasks.md generated
- `2026-04-07T14:00:00Z` — all PCs done, implementation complete
