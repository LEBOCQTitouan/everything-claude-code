# Tasks: BL-131 Phase-Gate Worktree State Fix

## Pass Conditions

| PC | Status | Wave | Test | Verifies | Notes |
|----|--------|------|------|----------|-------|
| PC-001 | [ ] pending | 1 | `cargo test -p ecc-app -- anchor_file_overrides_git_resolution` | AC-001.2, AC-001.6 | new |
| PC-002 | [ ] pending | 1 | `cargo test -p ecc-app -- worktree_returns_git_dir` | AC-001.3 | regression |
| PC-003 | [ ] pending | 1 | `cargo test -p ecc-app -- corrupt_anchor_falls_back` | AC-001.7 | new |
| PC-004 | [ ] pending | 1 | `cargo test -p ecc-app -- stale_anchor_falls_back` | AC-001.7 | new |
| PC-005 | [ ] pending | 2 | `cargo test -p ecc-workflow -- init_writes_state_dir_anchor` | AC-001.1, AC-001.8 | new |
| PC-006 | [ ] pending | 2 | `cargo test -p ecc-workflow -- init_succeeds_without_anchor` | AC-001.8 | new |
| PC-007 | [ ] pending | 3 | `cargo test -p ecc-workflow -- reset_deletes_state_dir_anchor` | AC-001.4 | new |
| PC-008 | [ ] pending | 3 | `cargo test -p ecc-workflow -- reset_force_deletes` | AC-001.9 | regression |
| PC-009 | [ ] pending | 1 | `cargo test -p ecc-app -- workflow::state_resolver` | AC-001.5 | regression |
| PC-010 | [ ] pending | 2 | `cargo test -p ecc-workflow -- init` | AC-001.5 | regression |
| PC-011 | [ ] pending | 3 | `cargo test -p ecc-workflow -- reset` | AC-001.5 | regression |
| PC-012 | [ ] pending | 4 | `cargo clippy -p ecc-app -p ecc-workflow -- -D warnings` | — | lint |
| PC-013 | [ ] pending | 4 | `cargo build -p ecc-app -p ecc-workflow` | — | build |
| PC-014 | [ ] pending | 5 | `git check-ignore .claude/workflow/.state-dir` | AC-002.2 | verify |
| PC-015 | [ ] pending | 5 | `test -z "$(git ls-files .claude/workflow/implement-done.md)"` | AC-002.1 | verify |

## Status Trail

- pending@2026-04-07T12:30:00Z — tasks.md created
