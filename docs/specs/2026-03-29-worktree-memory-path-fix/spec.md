# Spec: Fix worktree-safe memory path resolution

## Problem Statement

`resolve_project_memory_dir` in `crates/ecc-workflow/src/commands/memory_write.rs` uses `std::fs::canonicalize(project_dir)` to compute the `~/.claude/projects/<hash>/memory/` path. In a git worktree, `project_dir` is the worktree path (e.g., `.claude/worktrees/ecc-session-xxx/`), producing a different hash than the main repo root. This means `write_daily` and `write_memory_index` writes from worktree sessions are invisible to main-repo sessions and vice versa, fragmenting global Claude memory.

## Research Summary

- `ecc_flock::resolve_repo_root()` already exists and uses `git rev-parse --git-common-dir` to resolve the main repo root from any worktree
- Cargo's flock.rs uses the same pattern for worktree-safe lock paths
- The `write_action` and `write_work_item` functions are NOT affected — they use `project_dir.join("docs/memory")` which is worktree-local and merged via git
- ADR-0024 documents the design intent: "Lock paths resolve to main repo root (worktree-safe via `git rev-parse --git-common-dir`)"
- Audit finding SEC-003 (lock name path traversal) is tangentially related but not in scope

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Use `ecc_flock::resolve_repo_root` in `resolve_project_memory_dir` | Reuses existing worktree-safe resolution, consistent with lock path resolution | No |
| 2 | Hard error if not in a git repo | ECC requires git — detect via `.git` existence check on resolved root | No |
| 7 | Detection: check `.git` dir exists on resolved root | `resolve_repo_root` is infallible (falls back to input); verify result by checking `resolved_root.join(".git").exists()` — if false, return error | No |
| 3 | Both unit + integration tests with real git worktrees | Thorough coverage of the path resolution fix | No |
| 4 | Accept breaking change for non-git usage | ECC always runs inside git repos | No |
| 5 | Security review flagged | Shells out to `git rev-parse` — reuses trusted pattern from ecc-flock | No |
| 6 | CHANGELOG entry only, no ADR addendum | Small bugfix to existing ADR-0024 design | No |

## User Stories

### US-001: Worktree-safe global memory path resolution

**As a** developer running concurrent Claude Code sessions in worktrees, **I want** `write_daily` and `write_memory_index` to resolve to the same `~/.claude/projects/<hash>/memory/` directory regardless of whether I'm in a worktree or the main repo, **so that** daily memory and memory index are not fragmented across worktree-specific locations.

#### Acceptance Criteria

- AC-001.1: Given a session in a worktree, when `write_daily` runs, then it writes to the same `~/.claude/projects/<hash>/memory/daily/` as a main-repo session
- AC-001.2: Given a session in a worktree, when `write_memory_index` runs, then it updates the same `~/.claude/projects/<hash>/memory/MEMORY.md` as a main-repo session
- AC-001.3: Given a session NOT in a git repo, when `write_daily` or `write_memory_index` runs, then it returns an error (hard fail)
- AC-001.4: Given a session in the main repo (not a worktree), when `write_daily` runs, then behavior is unchanged from before the fix
- AC-001.5: Given `resolve_repo_root` returns a path where `.git` does not exist (non-git fallback), when `write_daily` or `write_memory_index` runs, then it returns `Err` with a descriptive message

#### Dependencies

- Depends on: none (ecc-flock::resolve_repo_root already exists)

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `crates/ecc-workflow/src/commands/memory_write.rs` | app (ecc-workflow) | Replace `canonicalize` with `resolve_repo_root` in `resolve_project_memory_dir` |
| `crates/ecc-workflow/Cargo.toml` | app (ecc-workflow) | Add `ecc-flock` dependency (if not already present) |
| `crates/ecc-integration-tests/` | test | Add worktree path resolution integration test |

## Constraints

- Must use `ecc_flock::resolve_repo_root` — no new git invocations
- Hard error on non-git-repo usage (no fallback to canonicalize)
- Security review required before merge (shells out to git)

## Non-Requirements

- Changing `write_action` or `write_work_item` path resolution (correctly worktree-local, merged via git)
- Changing `ecc_flock::resolve_repo_root` implementation (it stays infallible; git-check happens in the caller)
- Adding concurrency protection to the now-shared `~/.claude/projects/<hash>/memory/` directory (already protected by existing flock locks from Sub-Spec B of BL-065)
- ADR addendum (CHANGELOG only)
- Fixing SEC-003 (lock name path traversal) — separate concern

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| ecc-flock (resolve_repo_root) | Reused, not changed | None |
| File system (memory writes) | Path resolution changed | Daily/memory-index files land in correct location |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Bugfix | CHANGELOG | CHANGELOG.md | Add entry under ### Fixed |
| BL-065 status | Backlog | docs/backlog/BACKLOG.md | Mark BL-065 implemented |

## Open Questions

None — all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Scope boundaries | Audit all path resolution in ecc-workflow, fix resolve_project_memory_dir | User |
| 2 | Fallback on resolve_repo_root failure | Hard error — detect via `.git` existence check on resolved root | User |
| 3 | Test strategy | Both unit test + integration test with real git worktree | User |
| 4 | Breaking changes | Accept breaking change for non-git usage | User |
| 5 | Security review | Flag for security-reviewer | User |
| 6 | Domain concepts | No new terms needed (covered by ADR-0024) | Recommended |
| 7 | ADR needed | CHANGELOG only | User |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Worktree-safe global memory path resolution | 5 | none |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | Worktree write_daily resolves to same hash as main repo | US-001 |
| AC-001.2 | Worktree write_memory_index resolves to same hash as main repo | US-001 |
| AC-001.3 | Non-git-repo usage returns hard error | US-001 |
| AC-001.4 | Main repo (non-worktree) behavior unchanged | US-001 |
| AC-001.5 | resolve_repo_root non-git fallback detected and returns Err | US-001 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Ambiguity | 80 | PASS | Detection mechanism clarified in Decision 7 |
| Edge Cases | 75 | PASS | AC-001.5 covers non-git fallback path |
| Scope | 90 | PASS | Tightly focused, non-requirements explicit |
| Dependencies | 85 | PASS | Single dependency on ecc-flock, already exists |
| Testability | 80 | PASS | Both unit + integration tests specified |
| Decisions | 80 | PASS | Infallible function contradiction resolved |
| Rollback | 90 | PASS | Simple revert, old behavior via canonicalize |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-29-worktree-memory-path-fix/spec.md | Full spec + Phase Summary |
