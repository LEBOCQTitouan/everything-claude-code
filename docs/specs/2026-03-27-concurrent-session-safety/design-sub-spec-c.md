# Design: BL-065 Sub-Spec C — Worktree Session Isolation

## Spec Reference
Concern: dev, Feature: BL-065 Sub-Spec C: Worktree session isolation

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/worktree.rs` | CREATE | WorktreeName VO — generate, parse, validate | AC-006.6 |
| 2 | `crates/ecc-domain/src/lib.rs` | MODIFY | Add `pub mod worktree;` | US-006 |
| 3 | `crates/ecc-app/src/worktree.rs` | CREATE | gc use case with port traits | AC-007.3, AC-007.4 |
| 4 | `crates/ecc-app/src/lib.rs` | MODIFY | Add `pub mod worktree;` | US-007 |
| 5 | `crates/ecc-cli/src/commands/worktree.rs` | CREATE | `ecc worktree gc` CLI | AC-007.3 |
| 6 | `crates/ecc-cli/src/commands/mod.rs` | MODIFY | Register worktree | US-007 |
| 7 | `crates/ecc-cli/src/main.rs` | MODIFY | Add Worktree variant | US-007 |
| 8 | `crates/ecc-workflow/src/commands/worktree_name.rs` | CREATE | worktree-name subcommand | AC-006.6 |
| 9 | `crates/ecc-workflow/src/commands/mod.rs` | MODIFY | Add module | US-006 |
| 10 | `crates/ecc-workflow/src/main.rs` | MODIFY | Add variant | US-006 |
| 11 | `crates/ecc-workflow/src/commands/memory_write.rs` | MODIFY | resolve_repo_root for paths | AC-006.5 |
| 12-16 | `commands/spec-dev.md`, `spec-fix.md`, `spec-refactor.md`, `design.md`, `implement.md` | MODIFY | EnterWorktree/ExitWorktree | AC-006.1, AC-007.1, AC-007.2 |
| 17 | CHANGELOG.md | MODIFY | Add entry | — |
| 18 | CLAUDE.md | MODIFY | Add `ecc worktree gc` | AC-007.3 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | WorktreeName rejects injection chars | AC-006.6 | `cargo test -p ecc-domain -- worktree::tests::rejects_injection` | PASS |
| PC-002 | unit | WorktreeName generates correct format | AC-006.6 | `cargo test -p ecc-domain -- worktree::tests::generates_correct_format` | PASS |
| PC-003 | unit | WorktreeName parses name back | AC-007.3 | `cargo test -p ecc-domain -- worktree::tests::parses_name` | PASS |
| PC-004 | unit | worktree-name subcommand outputs valid name | AC-006.6 | `cargo test -p ecc-workflow -- worktree_name` | PASS |
| PC-005 | unit | memory_write resolves to repo root | AC-006.5 | `cargo test -p ecc-workflow -- memory_write::tests::resolves_repo_root` | PASS |
| PC-006 | unit | gc filters ecc-session-* only | AC-007.3 | `cargo test -p ecc-app -- worktree::tests::filters_session_worktrees` | PASS |
| PC-007 | unit | gc skips active worktrees | AC-007.4 | `cargo test -p ecc-app -- worktree::tests::skips_active` | PASS |
| PC-008 | unit | gc removes stale worktrees | AC-007.3 | `cargo test -p ecc-app -- worktree::tests::removes_stale` | PASS |
| PC-009 | integration | gc e2e with real git | AC-007.3, AC-007.4 | `cargo test -p ecc-integration-tests --test worktree_gc -- --ignored` | PASS |
| PC-010 | lint | EnterWorktree in 5 pipeline commands | AC-006.1 | `grep -c 'EnterWorktree' commands/spec-dev.md commands/spec-fix.md commands/spec-refactor.md commands/design.md commands/implement.md` | each >= 1 |
| PC-011 | lint | ExitWorktree in implement.md | AC-007.1 | `grep -q 'ExitWorktree' commands/implement.md` | exit 0 |
| PC-012 | lint | No EnterWorktree in non-pipeline | AC-006.3 | `grep -rL 'EnterWorktree' commands/audit*.md commands/verify.md commands/review.md commands/commit.md commands/catchup.md` | exit 0 |
| PC-013 | lint | gc uses -- separator | Security | `grep -q '"--"' crates/ecc-app/src/worktree.rs` | exit 0 |
| PC-014 | lint | clippy clean | — | `cargo clippy -- -D warnings` | exit 0 |
| PC-015 | build | cargo build | — | `cargo build` | exit 0 |
| PC-016 | unit | All tests pass | — | `cargo test` | all pass |

## Coverage Check
All 10 ACs covered. AC-006.2 and AC-006.4 are implicit (EnterWorktree tool semantics). AC-007.2 covered by command file text.

## Test Strategy
TDD order: PC-001→002→003 (domain), PC-004→005 (ecc-workflow), PC-006→007→008 (ecc-app), PC-009 (integration), PC-010→011→012→013 (lint), PC-014→015→016 (gates).

## Doc Update Plan
| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CHANGELOG.md | project | Add entry | Sub-Spec C worktree isolation | US-006, US-007 |
| 2 | CLAUDE.md | project | Add `ecc worktree gc` | CLI reference | AC-007.3 |

## SOLID Assessment
PASS (after prescriptions): WorktreeName in domain, gc via ecc-app use case with port traits.

## Robert's Oath Check
CLEAN (1 warning addressed).

## Security Notes
4 findings addressed: branch name allowlist, `--` separator, PID+age check, path canonicalization.

## Rollback Plan
Reverse order: revert commands → remove ecc-cli/ecc-app/ecc-workflow worktree code → remove ecc-domain worktree module.
