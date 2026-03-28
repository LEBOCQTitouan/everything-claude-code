# Design: BL-087 — Cargo xtask deploy

## Spec Reference
Concern: dev, Feature: BL-087: Cargo xtask deploy

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `Cargo.toml` (root) | MODIFY | Add "xtask" to workspace members | AC-001.3 |
| 2 | `.cargo/config.toml` | CREATE | Cargo alias for xtask | AC-001.4 |
| 3 | `xtask/Cargo.toml` | CREATE | New crate, clap+anyhow only | AC-001.1, AC-001.7 |
| 4 | `xtask/src/main.rs` | CREATE | Clap CLI with Deploy+--dry-run | AC-001.2, AC-001.5, AC-010.1 |
| 5 | `xtask/src/rc_block.rs` | CREATE | Pure managed block logic | AC-009.1-6 |
| 6 | `xtask/src/shell.rs` | CREATE | Shell detection + paths | AC-005.1, AC-006.1 |
| 7 | `xtask/src/deploy.rs` | CREATE | Orchestration | US-002-008, US-011 |
| 8 | `docs/adr/0026-xtask-deploy.md` | CREATE | ADR for xtask pattern | Decision #1 |
| 9 | CHANGELOG.md | MODIFY | Add BL-087 entry | — |
| 10 | CLAUDE.md | MODIFY | Add `cargo xtask deploy` | — |
| 11 | `docs/domain/glossary.md` | MODIFY | Add xtask, managed RC block | — |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | build | Workspace builds with xtask | AC-001.6 | `cargo build --workspace` | exit 0 |
| PC-002 | lint | --help shows --dry-run | AC-001.5, AC-010.1 | `cargo xtask deploy --help 2>&1 \| grep -q dry-run` | exit 0 |
| PC-003 | lint | No ECC deps | AC-001.7 | `grep -qv 'ecc-' xtask/Cargo.toml` | exit 0 |
| PC-004 | unit | rc_block: insert empty | AC-009.1 | `cargo test -p xtask -- rc_block::tests::insert_empty` | PASS |
| PC-005 | unit | rc_block: insert preserving | AC-009.1 | `cargo test -p xtask -- rc_block::tests::insert_preserving` | PASS |
| PC-006 | unit | rc_block: replace existing | AC-009.2 | `cargo test -p xtask -- rc_block::tests::replace_existing` | PASS |
| PC-007 | unit | rc_block: unchanged | AC-009.3 | `cargo test -p xtask -- rc_block::tests::unchanged` | PASS |
| PC-008 | unit | rc_block: missing marker | AC-009.1 | `cargo test -p xtask -- rc_block::tests::missing_marker` | PASS |
| PC-009 | unit | rc_block: empty lines | AC-009.1 | `cargo test -p xtask -- rc_block::tests::empty_lines` | PASS |
| PC-010 | unit | shell: detect all | AC-005.1 | `cargo test -p xtask -- shell::tests::detect` | PASS |
| PC-011 | unit | shell: unknown | AC-005.5 | `cargo test -p xtask -- shell::tests::unknown` | PASS |
| PC-012 | unit | shell: RC paths | AC-006.1 | `cargo test -p xtask -- shell::tests::rc_paths` | PASS |
| PC-013 | unit | shell: completion paths | AC-005.3 | `cargo test -p xtask -- shell::tests::completion_paths` | PASS |
| PC-014 | unit | shell: fish no source | AC-007.1 | `cargo test -p xtask -- shell::tests::fish_no_source` | PASS |
| PC-015 | unit | shell: block assembly | AC-006.3, AC-007.1 | `cargo test -p xtask -- shell::tests::block_assembly` | PASS |
| PC-016 | unit | deploy: cargo bin dir | AC-003.2 | `cargo test -p xtask -- deploy::tests::cargo_bin` | PASS |
| PC-017 | unit | deploy: summary format | AC-011.1 | `cargo test -p xtask -- deploy::tests::summary` | PASS |
| PC-018 | integration | dry-run exits 0 | AC-010.2-4 | `cargo xtask deploy --dry-run 2>&1; echo $?` | 0 |
| PC-019 | lint | ADR exists | Decision #1 | `test -f docs/adr/0026-xtask-deploy.md` | exit 0 |
| PC-020 | lint | CLAUDE.md updated | — | `grep -q 'xtask deploy' CLAUDE.md` | exit 0 |
| PC-021 | lint | clippy clean | — | `cargo clippy --workspace -- -D warnings` | exit 0 |
| PC-022 | build | full build | — | `cargo build --workspace` | exit 0 |

### Coverage Check
All ACs covered. Key mappings: AC-001.x→PC-001/002/003, AC-002.x→PC-018 (dry-run shows build step), AC-003.x→PC-016/018, AC-005.x→PC-010-014, AC-006.x→PC-012/015, AC-007.x→PC-014/015, AC-009.x→PC-004-009, AC-010.x→PC-002/018, AC-011.x→PC-017/018.

## Test Strategy
TDD order: PC-001→003 (scaffold), PC-004→009 (rc_block, 100% coverage), PC-010→015 (shell), PC-016→017 (deploy helpers), PC-018 (integration), PC-019→022 (docs+gates).

## Doc Update Plan
| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/adr/0026-xtask-deploy.md` | decision | CREATE | Xtask pattern ADR | Decision #1 |
| 2 | CHANGELOG.md | project | Add entry | BL-087 xtask deploy | — |
| 3 | CLAUDE.md | project | Add command | cargo xtask deploy | — |
| 4 | `docs/domain/glossary.md` | domain | Add 2 terms | xtask, managed RC block | — |

## SOLID Assessment
CLEAN — 4-file structure with clear SRP, no hexagonal boundary violations, pure logic isolated in rc_block.

## Robert's Oath Check
CLEAN — 0 warnings, rework ratio 0.00.

## Security Notes
CLEAR — RC file handling needs: atomic writes (same FS), backup, permission preservation, non-UTF-8 tolerance. All addressable at implementation time.

## Rollback Plan
1. Revert docs (glossary, CLAUDE.md, ADR, CHANGELOG)
2. Remove xtask/ directory
3. Remove "xtask" from Cargo.toml members
4. Delete .cargo/config.toml
