# Solution: Dead Code Cleanup — Audit Remediation

## Spec Reference
Concern: refactor, Feature: dead-code-cleanup-audit-remediation

## File Changes (dependency order)
| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | crates/ecc-app/src/ecc_config.rs | delete | Undeclared module, never compiles (120 lines) | US-001, AC-001.1 |
| 2 | crates/ecc-app/src/ecc_status.rs | delete | Undeclared module, superseded by diagnostics.rs (479 lines) | US-001, AC-001.2 |
| 3 | crates/ecc-app/src/merge/helpers_tests.rs | delete | Undeclared test file, never runs (865 lines) | US-001, AC-001.3 |
| 4 | agents/doc-updater.md | modify | Remove stale reference to scripts/codemaps/generate.ts (line 24 and surrounding code block) | US-002, AC-002.1 |
| 5 | commands/audit.md.deprecated | delete | Non-standard extension, deprecated | US-002, AC-002.2 |
| 6 | commands/doc-suite.md.deprecated | delete | Non-standard extension, deprecated | US-002, AC-002.3 |
| 7 | commands/e2e.md.deprecated | delete | Non-standard extension, deprecated | US-002, AC-002.4 |
| 8 | commands/optimize.md.deprecated | delete | Non-standard extension, deprecated | US-002, AC-002.5 |
| 9 | docs/specs/2026-03-30-three-tier-memory-system/spec-draft.md | delete | Stale draft superseded by spec.md | US-003, AC-003.1 |
| 10 | docs/pre-rebase-inventory.md | delete | Stale inventory referencing removed ecc-rs/ | US-003, AC-003.2 |
| 11 | docs/backlog/BACKLOG.md | modify | BL-088 status open → implemented | US-004, AC-004.1 |
| 12 | docs/backlog/BL-088-ecc-update-command.md | modify | status field open → implemented | US-004, AC-004.2 |
| 13 | CLAUDE.md | modify | Update test count from 2148 to actual | US-004, AC-004.3 |

## Pass Conditions
| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | build | ecc_config.rs deleted and build passes | AC-001.1 | `test ! -f crates/ecc-app/src/ecc_config.rs && cargo build` | exit 0 |
| PC-002 | build | ecc_status.rs deleted and build passes | AC-001.2 | `test ! -f crates/ecc-app/src/ecc_status.rs && cargo build` | exit 0 |
| PC-003 | build | helpers_tests.rs deleted and build passes | AC-001.3 | `test ! -f crates/ecc-app/src/merge/helpers_tests.rs && cargo build` | exit 0 |
| PC-004 | integration | Full test suite passes | AC-001.4 | `cargo test` | PASS |
| PC-005 | lint | Zero clippy warnings | AC-001.5 | `cargo clippy -- -D warnings` | exit 0 |
| PC-006 | build | No scripts/codemaps reference in doc-updater | AC-002.1 | `! grep -q 'scripts/codemaps' agents/doc-updater.md` | exit 0 |
| PC-007 | build | No .md.deprecated files in commands/ | AC-002.2-005 | `test $(find commands -name '*.deprecated' 2>/dev/null \| wc -l) -eq 0` | exit 0 |
| PC-008 | build | spec-draft.md deleted | AC-003.1 | `test ! -f docs/specs/2026-03-30-three-tier-memory-system/spec-draft.md` | exit 0 |
| PC-009 | build | pre-rebase-inventory.md deleted | AC-003.2 | `test ! -f docs/pre-rebase-inventory.md` | exit 0 |
| PC-010 | build | Orphan worktrees pruned | AC-003.3 | `git worktree prune` | exit 0 |
| PC-011 | build | BL-088 marked implemented | AC-004.1, AC-004.2 | `grep 'BL-088.*implemented' docs/backlog/BACKLOG.md` | exit 0 |
| PC-012 | build | CLAUDE.md test count accurate | AC-004.3 | `actual=$(cargo test -- --list 2>/dev/null \| grep -c ': test') && claimed=$(grep -oE '[0-9]+ tests' CLAUDE.md \| grep -oE '[0-9]+') && [ "$actual" -eq "$claimed" ]` | exit 0 |
| PC-013 | lint | Code formatting check | AC-001.5 | `cargo fmt -- --check` | exit 0 |

### Coverage Check
All 16 ACs covered by 13 PCs. Zero uncovered.

| AC | Covering PC(s) |
|----|---------------|
| AC-001.1 | PC-001 |
| AC-001.2 | PC-002 |
| AC-001.3 | PC-003 |
| AC-001.4 | PC-004 |
| AC-001.5 | PC-005, PC-013 |
| AC-002.1 | PC-006 |
| AC-002.2 | PC-007 |
| AC-002.3 | PC-007 |
| AC-002.4 | PC-007 |
| AC-002.5 | PC-007 |
| AC-003.1 | PC-008 |
| AC-003.2 | PC-009 |
| AC-003.3 | PC-010 |
| AC-004.1 | PC-011 |
| AC-004.2 | PC-011 |
| AC-004.3 | PC-012 |

### E2E Test Plan
No E2E boundaries affected — all changes are deletions of dead code.

### E2E Activation Rules
None — no E2E tests to activate.

## Test Strategy
TDD order (dependency-driven):
1. PC-001 → Delete ecc_config.rs, verify build
2. PC-002 → Delete ecc_status.rs, verify build
3. PC-003 → Delete helpers_tests.rs, verify build
4. PC-004 → Full test suite passes
5. PC-005 → Clippy zero warnings
6. PC-013 → Formatting check
7. PC-006 → Fix doc-updater.md reference
8. PC-007 → Delete 4 deprecated commands
9. PC-008 → Delete spec-draft.md
10. PC-009 → Delete pre-rebase-inventory.md
11. PC-010 → Prune orphan worktrees
12. PC-011 → Update BL-088 status
13. PC-012 → Update CLAUDE.md test count (must be last)

Commit grouping:
1. `refactor: delete undeclared dead Rust modules` — PC-001 through PC-005, PC-013
2. `refactor: fix stale agent reference and delete deprecated commands` — PC-006, PC-007
3. `chore: remove stale documentation artifacts` — PC-008, PC-009, PC-010
4. `docs: update BL-088 status and fix CLAUDE.md test count` — PC-011, PC-012

## Doc Update Plan
| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CLAUDE.md | Root | Edit test count | Update from 2148 to actual count | AC-004.3 |
| 2 | docs/backlog/BACKLOG.md | Docs | Edit BL-088 row | status: open → implemented | AC-004.1 |
| 3 | docs/backlog/BL-088-ecc-update-command.md | Docs | Edit status field | open → implemented | AC-004.2 |

No CHANGELOG entry — internal housekeeping with no user-visible change.
No ADRs — all decisions marked "ADR Needed? No" in spec.

## SOLID Assessment
PASS — pure deletions of undeclared files. No architectural changes. No new dependencies.

## Robert's Oath Check
CLEAN — no harmful code produced, no mess created, existing tests remain green, small atomic releases.

## Security Notes
CLEAR — deleted files contain no security-sensitive code. No secrets, auth logic, or validation in any deleted file.

## Rollback Plan
Reverse dependency order:
1. Revert CLAUDE.md and backlog changes (`git revert <commit4>`)
2. Restore docs/pre-rebase-inventory.md and spec-draft.md (`git revert <commit3>`)
3. Restore 4 .md.deprecated command files and doc-updater.md (`git revert <commit2>`)
4. Restore 3 dead Rust files (`git revert <commit1>`)
