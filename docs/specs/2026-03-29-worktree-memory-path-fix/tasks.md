# Tasks: Fix worktree-safe memory path resolution

## Pass Conditions

- [ ] PC-001: resolve_project_memory_dir errors on non-git | unit | AC-001.3, AC-001.5
- [ ] PC-002: resolve_project_memory_dir succeeds for git repo | unit | AC-001.4
- [ ] PC-003: Worktree daily resolves to main repo hash | integration | AC-001.1
- [ ] PC-004: Worktree memory-index resolves to main repo hash | integration | AC-001.2
- [ ] PC-005: Non-git dir returns error | integration | AC-001.3, AC-001.5
- [ ] PC-006: Existing memory_write_subcommands passes | integration | AC-001.4
- [ ] PC-007: cargo clippy zero warnings | lint | all
- [ ] PC-008: cargo build --release | build | all
- [ ] PC-009: Full test suite passes | test | all

## Post-TDD

- [ ] E2E tests
- [ ] Code review
- [ ] Doc updates
- [ ] Supplemental docs
- [ ] Write implement-done.md

## Status Trail

<!-- Append status updates here -->
