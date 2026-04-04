# Solution: Fix worktree merge — checkout main before ff-only

## Spec Reference
Concern: fix, Feature: Worktree merge stale working tree

## File Changes

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | crates/ecc-workflow/src/commands/merge.rs | Modify | Add checkout_main() + 6 new tests | US-001, US-002 |
| 2 | CLAUDE.md | Modify | Add gotcha warning | US-003 |
| 3 | CHANGELOG.md | Modify | Add fix entry | Convention |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | checkout_main when already on main | AC-001.1 | `cargo test -p ecc-workflow checkout_main_when_already_on_main` | PASS |
| PC-002 | unit | checkout_main from detached HEAD | AC-001.2 | `cargo test -p ecc-workflow checkout_main_from_detached` | PASS |
| PC-003 | unit | checkout_main from other branch | AC-001.3 | `cargo test -p ecc-workflow checkout_main_from_other_branch` | PASS |
| PC-004 | unit | checkout_main dirty tree fails | AC-001.4 | `cargo test -p ecc-workflow checkout_main_dirty_tree_fails` | PASS |
| PC-005 | integration | merge updates working tree files | AC-002.1 | `cargo test -p ecc-workflow merge_updates_working_tree_files` | PASS |
| PC-006 | integration | merge leaves clean status | AC-002.2 | `cargo test -p ecc-workflow merge_leaves_clean_status` | PASS |
| PC-007 | lint | CLAUDE.md has update-ref warning | AC-003.1 | `grep -q 'update-ref' CLAUDE.md` | exit 0 |
| PC-008 | build | workspace builds | ALL | `cargo build --workspace` | exit 0 |
| PC-009 | lint | clippy clean | ALL | `cargo clippy --workspace -- -D warnings` | exit 0 |

## Coverage Check
All 7 ACs covered. Zero uncovered.

## Rollback Plan
1. Revert CHANGELOG.md
2. Revert CLAUDE.md
3. Revert merge.rs (remove checkout_main + tests)
