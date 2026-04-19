# Tasks: BL-155 Foundation Concern Variant

Source: `docs/specs/2026-04-19-bl155-foundation-concern/design.md`
Concern: dev | Feature: BL-155 Add Foundation variant to Concern enum

## TDD Status Trail

### Wave 1: Domain (concern.rs + state.rs)
- [ ] PC-001 FromStr parses "foundation": pending
- [ ] PC-002 Display yields "foundation": pending
- [ ] PC-003 serialize foundation: pending
- [ ] PC-004 deserialize foundation: pending
- [ ] PC-005 UnknownConcern alphabetical text: pending
- [ ] PC-006 from_str rejects "Foundation": pending
- [ ] PC-007 round_trips_all_variants (4): pending
- [ ] PC-008 existing tests unchanged: pending
- [ ] PC-009 state.rs round-trip: pending

### Wave 2: Integration
- [ ] PC-010 init foundation writes concern=foundation: pending
- [ ] PC-011 delegator matches direct: pending
- [ ] PC-012 worktree-name foundation: pending
- [ ] PC-013 full FSM walk: pending

### Wave 3: Docs
- [ ] PC-014 project-foundation.md revert: pending
- [ ] PC-015 catchup.md update: pending
- [ ] PC-016 CHANGELOG BL-155 entry: pending
- [ ] PC-017 campaign-manifest skill update: pending
- [ ] PC-018 artifact-schemas skill update: pending

### Wave 4: Version + Gates
- [ ] PC-022 workspace version 4.3.0: pending
- [ ] PC-019 ecc validate commands: pending
- [ ] PC-020 clippy: pending
- [ ] PC-021 full test suite: pending
- [ ] PC-023 diff-based no-existing-tests-deleted: pending
- [ ] PC-024 cargo semver-checks: pending
- [ ] PC-025 cargo fmt --check: pending

### Post-TDD
- [ ] E2E tests: N/A (integration covers)
- [ ] Code review: pending
- [ ] Doc updates: pending (covered by Wave 3)
- [ ] Supplemental docs: SKIP (doc-only variant extension)
- [ ] Write implement-done.md: pending
