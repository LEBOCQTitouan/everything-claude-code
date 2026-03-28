# Tasks: BL-065 Sub-Spec C — Worktree Session Isolation

## Pass Conditions

- [ ] PC-001: WorktreeName rejects injection chars | `cargo test -p ecc-domain -- worktree::tests::rejects_injection` | pending@2026-03-28T15:10:00Z
- [ ] PC-002: WorktreeName generates correct format | `cargo test -p ecc-domain -- worktree::tests::generates_correct_format` | pending@2026-03-28T15:10:00Z
- [ ] PC-003: WorktreeName parses name back | `cargo test -p ecc-domain -- worktree::tests::parses_name` | pending@2026-03-28T15:10:00Z
- [ ] PC-004: worktree-name subcommand | `cargo test -p ecc-workflow -- worktree_name` | pending@2026-03-28T15:10:00Z
- [ ] PC-005: memory_write resolves to repo root | `cargo test -p ecc-workflow -- memory_write::tests::resolves_repo_root` | pending@2026-03-28T15:10:00Z
- [ ] PC-006: gc filters ecc-session-* | `cargo test -p ecc-app -- worktree::tests::filters_session_worktrees` | pending@2026-03-28T15:10:00Z
- [ ] PC-007: gc skips active worktrees | `cargo test -p ecc-app -- worktree::tests::skips_active` | pending@2026-03-28T15:10:00Z
- [ ] PC-008: gc removes stale worktrees | `cargo test -p ecc-app -- worktree::tests::removes_stale` | pending@2026-03-28T15:10:00Z
- [ ] PC-009: gc e2e with real git | `cargo test -p ecc-integration-tests --test worktree_gc -- --ignored` | pending@2026-03-28T15:10:00Z
- [ ] PC-010: EnterWorktree in 5 pipeline commands | `grep -c 'EnterWorktree' commands/spec-dev.md commands/spec-fix.md commands/spec-refactor.md commands/design.md commands/implement.md` | pending@2026-03-28T15:10:00Z
- [ ] PC-011: ExitWorktree in implement.md | `grep -q 'ExitWorktree' commands/implement.md` | pending@2026-03-28T15:10:00Z
- [ ] PC-012: No EnterWorktree in non-pipeline | lint | pending@2026-03-28T15:10:00Z
- [ ] PC-013: gc uses -- separator | `grep -q '"--"' crates/ecc-app/src/worktree.rs` | pending@2026-03-28T15:10:00Z
- [ ] PC-014: clippy clean | `cargo clippy -- -D warnings` | pending@2026-03-28T15:10:00Z
- [ ] PC-015: cargo build | `cargo build` | pending@2026-03-28T15:10:00Z
- [ ] PC-016: All tests pass | `cargo test` | pending@2026-03-28T15:10:00Z

## Post-TDD

- [ ] E2E tests | pending@2026-03-28T15:10:00Z
- [ ] Code review | pending@2026-03-28T15:10:00Z
- [ ] Doc updates | pending@2026-03-28T15:10:00Z
- [ ] Supplemental docs | pending@2026-03-28T15:10:00Z
- [ ] Write implement-done.md | pending@2026-03-28T15:10:00Z
