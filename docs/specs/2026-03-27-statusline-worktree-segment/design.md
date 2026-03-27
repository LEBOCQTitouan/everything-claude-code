# Solution: Add Worktree Display Segment to Statusline

## Spec Reference
Concern: dev, Feature: Add worktree display segment to statusline

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `docs/adr/0018-worktree-replaces-branch-segment.md` | create | Document the decision that worktree segment replaces branch segment | US-007, AC-007.1 |
| 2 | `crates/ecc-domain/src/config/statusline.rs` | modify | Add Worktree to default field_order after GitBranch; reconcile order with script | US-005, AC-005.1, AC-005.2, AC-005.3 |
| 3 | `tests/statusline/test_helper.bash` | create | Shared Bats helpers: disposable git repos, worktree creation, script sourcing | US-006, AC-006.5 |
| 4 | `tests/statusline/worktree.bats` | create | 14 Bats tests covering detection, rendering, caching, edge cases | US-006, AC-006.1–AC-006.4 |
| 5 | `statusline/statusline-command.sh` | modify | Add worktree detection, cache format update, segment rendering, branch replacement, narrow variant | US-001–US-004 |
| 6 | `CHANGELOG.md` | modify | Add entry for worktree statusline segment | All US |
| 7 | `docs/backlog/BL-082-statusline-worktree-segment.md` | modify | Update status to promoted | BL-082 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | field_order has Worktree after GitBranch | AC-005.1 | `cargo test -p ecc-domain field_order_contains_worktree_after_git_branch` | PASS |
| PC-002 | unit | field_order matches script rendering priority | AC-005.2 | `cargo test -p ecc-domain field_order_matches_script_rendering_priority` | PASS |
| PC-003 | unit | All existing domain tests pass | AC-005.3 | `cargo test -p ecc-domain` | PASS |
| PC-024 | bats | Worktree segment exact format string | AC-001.1 | `bats tests/statusline/worktree.bats --filter "exact format"` | PASS |
| PC-025 | bats | Worktree name with special characters | AC-001.4 | `bats tests/statusline/worktree.bats --filter "name with hyphens"` | PASS |
| PC-004 | bats | Worktree segment appears when in worktree | AC-001.1, AC-006.1 | `bats tests/statusline/worktree.bats --filter "shows worktree segment when in worktree"` | PASS |
| PC-005 | bats | No worktree segment in main working tree | AC-001.2, AC-006.1 | `bats tests/statusline/worktree.bats --filter "no worktree segment in main working tree"` | PASS |
| PC-006 | bats | Branch segment replaced by worktree | AC-001.3, AC-006.2 | `bats tests/statusline/worktree.bats --filter "branch segment is replaced"` | PASS |
| PC-007 | bats | Worktree name is basename from subdirectory | AC-001.4, AC-006.1 | `bats tests/statusline/worktree.bats --filter "worktree name is basename from subdirectory"` | PASS |
| PC-008 | bats | Detached HEAD shows (detached) | AC-002.1, AC-006.1 | `bats tests/statusline/worktree.bats --filter "detached HEAD shows detached"` | PASS |
| PC-009 | bats | Bare repo hides worktree segment | AC-002.2, AC-006.1 | `bats tests/statusline/worktree.bats --filter "bare repo hides worktree segment"` | PASS |
| PC-010 | bats | Non-git directory hides segment | AC-002.3, AC-006.1 | `bats tests/statusline/worktree.bats --filter "non-git directory hides segment"` | PASS |
| PC-011 | bats | Stale worktree hides segment | AC-002.4, AC-006.1 | `bats tests/statusline/worktree.bats --filter "stale worktree hides segment"` | PASS |
| PC-012 | bats | Cache hit within TTL | AC-003.1, AC-006.3 | `bats tests/statusline/worktree.bats --filter "cache hit within TTL"` | PASS |
| PC-013 | bats | Cache miss refreshes data | AC-003.2, AC-006.3 | `bats tests/statusline/worktree.bats --filter "cache miss refreshes"` | PASS |
| PC-014 | bats | Cache format is newline-delimited | AC-003.3, AC-006.3 | `bats tests/statusline/worktree.bats --filter "cache format is newline-delimited"` | PASS |
| PC-015 | bats | Legacy single-line cache backward compatible | AC-003.4 | `bats tests/statusline/worktree.bats --filter "legacy single-line cache"` | PASS |
| PC-016 | bats | Truncation drops worktree at rank 4 | AC-004.1, AC-006.2 | `bats tests/statusline/worktree.bats --filter "truncation drops worktree"` | PASS |
| PC-017 | bats | Narrow variant drops branch | AC-004.2, AC-006.2 | `bats tests/statusline/worktree.bats --filter "narrow variant drops branch"` | PASS |
| PC-018 | bats | All Bats tests pass | AC-006.4 | `bats tests/statusline/` | PASS |
| PC-019 | bats | Test isolation with disposable repos | AC-006.5 | `bats tests/statusline/worktree.bats` | PASS |
| PC-020 | lint | ADR-0018 exists | AC-007.1 | `test -f docs/adr/0018-worktree-replaces-branch-segment.md` | exit 0 |
| PC-021 | build | Rust build passes | AC-005.3 | `cargo build --release` | exit 0 |
| PC-022 | lint | Clippy passes | AC-005.3 | `cargo clippy -- -D warnings` | exit 0 |
| PC-023 | lint | Markdown lint passes | all docs | `npm run lint` | exit 0 |

### Coverage Check

All 23 ACs covered:

| AC | Covering PCs |
|----|-------------|
| AC-001.1 | PC-004 |
| AC-001.2 | PC-005 |
| AC-001.3 | PC-006 |
| AC-001.4 | PC-007 |
| AC-002.1 | PC-008 |
| AC-002.2 | PC-009 |
| AC-002.3 | PC-010 |
| AC-002.4 | PC-011 |
| AC-003.1 | PC-012 |
| AC-003.2 | PC-013 |
| AC-003.3 | PC-014 |
| AC-003.4 | PC-015 |
| AC-004.1 | PC-016 |
| AC-004.2 | PC-017 |
| AC-005.1 | PC-001 |
| AC-005.2 | PC-002 |
| AC-005.3 | PC-003, PC-021, PC-022 |
| AC-006.1 | PC-004, PC-005, PC-007, PC-008, PC-009, PC-010, PC-011 |
| AC-006.2 | PC-006, PC-016, PC-017 |
| AC-006.3 | PC-012, PC-013, PC-014 |
| AC-006.4 | PC-018 |
| AC-006.5 | PC-019 |
| AC-007.1 | PC-020 |

Zero uncovered ACs.

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | Statusline rendering | bash script | N/A | Worktree segment renders correctly in worktree/non-worktree | ignored | statusline-command.sh modified |

### E2E Activation Rules

Bats tests (PC-004–PC-019) serve as E2E validation for this implementation. No Rust E2E tests to activate.

## Test Strategy

TDD order (dependency-driven):

1. **PC-020** — ADR-0018 doc (no code dependency, ship first)
2. **PC-001, PC-002** — Domain tests RED (write assertions for Worktree position and full order)
3. **PC-003** — Domain GREEN (reorder field_order, verify all existing tests pass)
4. **PC-021, PC-022** — Build + clippy gate after domain change
5. **PC-004–PC-017, PC-019** — Bats tests RED (write all 14 tests, all fail since script not yet updated)
6. **PC-004–PC-017, PC-019** — Script GREEN (implement detection, caching, rendering; all Bats tests pass)
7. **PC-018** — Full Bats suite confirmation
8. **PC-023** — Markdown lint gate

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/adr/0018-worktree-replaces-branch-segment.md` | docs | Create | ADR: branch replacement decision | US-007, AC-007.1 |
| 2 | `CHANGELOG.md` | docs | Append | Worktree statusline segment entry | All US |
| 3 | `docs/backlog/BL-082-statusline-worktree-segment.md` | docs | Update | Status → promoted | BL-082 |
| 4 | `CLAUDE.md` | docs | Update | Update test count after adding Rust + Bats tests | gotchas |

## SOLID Assessment

PASS with 3 advisory findings:
1. **MEDIUM (SRP)**: Extract worktree detection into a named function (e.g., `detect_worktree`) rather than inlining into main flow
2. **MEDIUM (OCP)**: Consider data-driven segment registration in `build_output()` for future extensibility
3. **MEDIUM (SRP)**: Encapsulate cache read/write in helper functions

No CRITICAL or HIGH findings. Hexagonal boundaries respected: domain remains pure, bash adapter remains a leaf.

**VimMode clarification**: `VimMode` exists in the `StatuslineField` enum (variant 12) but is intentionally excluded from the default `field_order` — same pattern as `Worktree` was before this change. The existing test at line 242-255 asserts that 12 enum variants exist (unchanged), not that all 12 are in `field_order`. The new `field_order` has 11 entries (adds Worktree, keeps VimMode excluded). No existing test breaks.

## Robert's Oath Check

CLEAN — No oath violations. Design is clean, testable, incremental, and free of harmful side effects. TDD order produces atomic commits with the codebase in a valid state after each.

## Security Notes

CLEAR — No blocking findings. Cache test fragility mitigations: Bats tests must use `$BATS_TEST_TMPDIR` for cache files (not global `/tmp`), and TTL tests must use file mtime manipulation (`touch -t`) rather than `sleep`.

Details:
- `eval` with `jq @sh` quoting is idiomatic and safe for trusted input (Claude Code JSON)
- Cache file in `/tmp` with predictable name is acceptable for non-sensitive data
- Worktree name from `basename $(git rev-parse --show-toplevel)` is safe
- Atomic writes via `mktemp` + `mv` prevent TOCTOU races

## Rollback Plan

Reverse dependency order:
1. Revert `statusline/statusline-command.sh` — removes worktree detection and rendering
2. Revert `tests/statusline/worktree.bats` and `test_helper.bash` — removes test infrastructure
3. Revert `crates/ecc-domain/src/config/statusline.rs` — restores original field_order
4. Delete `docs/adr/0018-worktree-replaces-branch-segment.md` — removes ADR
5. Revert CHANGELOG.md and BL-082 status changes

Cache self-heals via 5s TTL — stale multi-line cache files become irrelevant within seconds.
