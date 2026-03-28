# Tasks: BL-065 Sub-Spec D — Serialized Merge-to-Main

## Pass Conditions

- [ ] PC-001: MergeError maps to WorkflowOutput | `cargo test -p ecc-workflow -- merge::tests::error_mapping` | pending@2026-03-28T16:00:00Z
- [ ] PC-003: Lock timeout returns block | `cargo test -p ecc-workflow -- merge::tests::timeout_blocks` | pending@2026-03-28T16:00:00Z
- [ ] PC-005: Rebase conflict aborts + notifies | `cargo test -p ecc-workflow -- merge::tests::conflict_aborts` | pending@2026-03-28T16:00:00Z
- [ ] PC-007: Verify failure notifies | `cargo test -p ecc-workflow -- merge::tests::verify_failure` | pending@2026-03-28T16:00:00Z
- [ ] PC-010: Session branch guard rejects main | `cargo test -p ecc-workflow -- merge::tests::rejects_main` | pending@2026-03-28T16:00:00Z
- [ ] PC-002: acquire_merge_lock uses 60s timeout | `cargo test -p ecc-workflow -- merge::tests::acquires_lock` | pending@2026-03-28T16:00:00Z
- [ ] PC-004: rebase runs git rebase main | `cargo test -p ecc-workflow -- merge::tests::runs_rebase` | pending@2026-03-28T16:00:00Z
- [ ] PC-006: fast verify runs build+test+clippy | `cargo test -p ecc-workflow -- merge::tests::runs_verify` | pending@2026-03-28T16:00:00Z
- [ ] PC-008: merge_fast_forward does ff-only | `cargo test -p ecc-workflow -- merge::tests::ff_merge` | pending@2026-03-28T16:00:00Z
- [ ] PC-009: cleanup removes worktree+branch | `cargo test -p ecc-workflow -- merge::tests::cleanup` | pending@2026-03-28T16:00:00Z
- [ ] PC-011: implement.md references merge | `grep -q 'ecc-workflow merge' commands/implement.md` | pending@2026-03-28T16:00:00Z
- [ ] PC-012: ADR exists | `test -f docs/adr/0024-concurrent-session-safety.md` | pending@2026-03-28T16:00:00Z
- [ ] PC-013: Glossary has 3 terms | `grep -c 'session worktree\|merge lock\|fast verify' docs/domain/glossary.md` | pending@2026-03-28T16:00:00Z
- [ ] PC-014: clippy clean | `cargo clippy -- -D warnings` | pending@2026-03-28T16:00:00Z
- [ ] PC-015: cargo build | `cargo build` | pending@2026-03-28T16:00:00Z
- [ ] PC-016: All tests pass | `cargo test` | pending@2026-03-28T16:00:00Z

## Post-TDD

- [ ] E2E tests | pending@2026-03-28T16:00:00Z
- [ ] Code review | pending@2026-03-28T16:00:00Z
- [ ] Doc updates | pending@2026-03-28T16:00:00Z
- [ ] Supplemental docs | pending@2026-03-28T16:00:00Z
- [ ] Write implement-done.md | pending@2026-03-28T16:00:00Z
