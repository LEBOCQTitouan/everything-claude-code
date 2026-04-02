# Spec: Dead Code Cleanup — Audit Remediation

## Problem Statement

The full-2026-03-31 codebase audit identified 1,464 lines of Rust source code across 3 files that never compile because they lack `mod` declarations, 865 lines of dead tests, 4 deprecated command files with non-standard extensions, stale documentation artifacts, an orphan build directory from the TypeScript era, and a backlog status mismatch. These dead files inflate the apparent codebase size, create phantom test coverage, and mislead developers about what code is active. Cleanup is needed now because the audit quantified the problem and every item is a risk-free deletion.

## Research Summary

- **Detection Tools**: `cargo-shear` identifies unlinked files in Rust workspaces; `cargo-machete` finds unused dependencies. The Rust compiler's dead-code warnings are crate-scoped and miss `pub` items and undeclared modules.
- **Safe Removal Pattern**: Detect → verify with `cargo build && cargo test` → commit atomically → verify downstream crates.
- **Version Control Requirement**: All removal tools require VCS safety net. Worktree isolation provides this.
- **Compiler Limitation**: Built-in Rust dead-code warnings ignore `pub` methods and operate crate-by-crate — third-party tools needed for workspace-wide coverage.
- **Git Worktree Cleanup**: Use `git worktree remove` or `git worktree prune` — manual deletion leaves stale metadata.
- **Incremental Approach**: Group deletions by concern (Rust code, content, filesystem, docs) for clean git history.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Delete `helpers_tests.rs` rather than wiring it | Tests are 865 lines for code that has evolved; re-wiring would require significant updates to match current APIs | No |
| 2 | Delete `.md.deprecated` files rather than archiving | No convention exists for deprecated commands; the git history preserves them | No |
| 3 | Group into 4 atomic commits | Each commit is independently verifiable and matches one logical concern | No |
| 4 | `ecc-rs/` and `node_modules/` are local-only artifacts not tracked by git | These exist on the developer machine but not in the repository — no git-tracked deletion needed | No |
| 5 | `valid_statuses()` and `assert_structured_json_output()` in `ecc-workflow/tests/common/mod.rs` are confirmed active | Adversary review found they are called from `init.rs` and `artifacts.rs` — NOT dead code | No |

## User Stories

### US-001: Remove undeclared Rust source files

**As a** developer, **I want** dead Rust modules removed, **so that** the codebase only contains code that compiles and runs.

#### Acceptance Criteria

- AC-001.1: Given `crates/ecc-app/src/ecc_config.rs` exists and is not declared via `mod`, when deleted, then `cargo build` passes
- AC-001.2: Given `crates/ecc-app/src/ecc_status.rs` exists and is not declared via `mod`, when deleted, then `cargo build` passes
- AC-001.3: Given `crates/ecc-app/src/merge/helpers_tests.rs` exists and is not declared via `mod` or `#[path]`, when deleted, then `cargo build` passes
- AC-001.4: Given all deletions applied, when `cargo build && cargo test` run, then both pass
- AC-001.5: Given all deletions applied, when `cargo clippy -- -D warnings` runs, then zero warnings

#### Dependencies

- Depends on: none

### US-002: Fix stale references and remove deprecated commands

**As a** developer, **I want** stale references fixed and deprecated commands cleaned up, **so that** all file references resolve correctly.

#### Acceptance Criteria

- AC-002.1: Given `agents/doc-updater.md` contains the line `npx tsx scripts/codemaps/generate.ts` (which references a non-existent script), when that line and its surrounding command block are removed, then no `scripts/codemaps` references remain in the file
- AC-002.2: Given `commands/audit.md.deprecated` exists, when deleted, then no `.md.deprecated` files remain in `commands/`
- AC-002.3: Given `commands/doc-suite.md.deprecated` exists, when deleted, then file is removed
- AC-002.4: Given `commands/e2e.md.deprecated` exists, when deleted, then file is removed
- AC-002.5: Given `commands/optimize.md.deprecated` exists, when deleted, then file is removed

#### Dependencies

- Depends on: none

### US-003: Clean filesystem artifacts

**As a** developer, **I want** stale filesystem artifacts removed, **so that** the repo has no dead weight.

#### Acceptance Criteria

- AC-003.1: Given `docs/specs/2026-03-30-three-tier-memory-system/spec-draft.md` exists alongside `spec.md`, when the draft is deleted, then only `spec.md` remains
- AC-003.2: Given `docs/pre-rebase-inventory.md` references the removed `ecc-rs/` directory, when deleted, then no stale inventory documents exist
- AC-003.3: Given orphan worktrees exist in `.claude/worktrees/`, when `git worktree prune` runs from the main worktree (not from inside an orphan), then no orphan worktree metadata remains

#### Dependencies

- Depends on: none

### US-004: Update backlog status and documentation

**As a** developer, **I want** backlog and docs accurate, **so that** project tracking reflects reality.

#### Acceptance Criteria

- AC-004.1: Given BL-088 is fully implemented but marked `open`, when status is updated to `implemented` in `docs/backlog/BACKLOG.md`, then the backlog index reflects the actual state
- AC-004.2: Given `docs/backlog/BL-088-*.md` has `status: open`, when updated to `status: implemented`, then the individual entry matches the index
- AC-004.3: Given CLAUDE.md claims 2,148 tests but actual count differs after all prior deletions, when `cargo test -- --list 2>&1 | grep -c ': test'` is run and the CLAUDE.md count is updated to match, then the documented count equals the actual count

#### Dependencies

- Depends on: US-001 (test count may change after dead code removal)

## Affected Modules

| Module | Layer | Change Type |
|--------|-------|-------------|
| `crates/ecc-app/src/ecc_config.rs` | Application | Delete |
| `crates/ecc-app/src/ecc_status.rs` | Application | Delete |
| `crates/ecc-app/src/merge/helpers_tests.rs` | Application | Delete |
| `agents/doc-updater.md` | Agent config | Edit (fix stale reference) |
| `commands/audit.md.deprecated` | Command config | Delete |
| `commands/doc-suite.md.deprecated` | Command config | Delete |
| `commands/e2e.md.deprecated` | Command config | Delete |
| `commands/optimize.md.deprecated` | Command config | Delete |
| `docs/specs/.../spec-draft.md` | Documentation | Delete |
| `docs/pre-rebase-inventory.md` | Documentation | Delete |
| `docs/backlog/BACKLOG.md` | Documentation | Edit (BL-088 status) |
| `docs/backlog/BL-088-*.md` | Documentation | Edit (status field) |
| `CLAUDE.md` | Documentation | Edit (test count) |

## Constraints

- All changes are behavior-preserving (pure deletions and metadata updates)
- `cargo build && cargo test && cargo clippy -- -D warnings` must pass after each commit
- No new code introduced
- Git history preserves all deleted content
- Commit ordering: US-001 first (Rust code), then US-002 and US-003 (independent, any order), then US-004 last (depends on US-001 for accurate test count)
- `ecc-rs/` and `node_modules/` are local-only artifacts not in git — excluded from this spec
- If `cargo build` or `cargo test` fails after a deletion, revert that deletion and investigate before proceeding

## Non-Requirements

- Wiring `helpers_tests.rs` via `#[path]` (deleting instead — tests are stale)
- Architectural changes to `ecc-workflow` port adoption (separate spec)
- Security fixes SEC-001/SEC-002 (separate spec)
- Running `cargo-machete` or `cargo-shear` for dependency-level dead code (out of scope)
- Removing `valid_statuses()` / `assert_structured_json_output()` from `ecc-workflow/tests/common/mod.rs` (confirmed active by adversary review)
- Cleaning `ecc-rs/` or `node_modules/` directories (local-only, not in git)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | N/A | No E2E impact — all changes are deletions of dead code |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Test count | CLAUDE.md | Update count to match actual | Edit |
| Backlog status | BACKLOG.md | BL-088 → implemented | Edit |
| Backlog entry | BL-088-*.md | status field | Edit |

## Open Questions

None — all questions resolved during grill-me interview.
