# Solution: Fix worktree-safe memory path resolution

## Spec Reference
Concern: dev, Feature: Concurrent session safety — worktree isolation, serialized merge, shared state fixes (BL-065)

## Overview

Replace `std::fs::canonicalize` with `ecc_flock::resolve_repo_root` in `resolve_project_memory_dir`, then verify the resolved root is a git repo via `.git` existence check. This ensures worktree sessions and main-repo sessions produce the same `~/.claude/projects/<hash>/memory/` path.

## Architecture Changes

| # | File | Layer | Change | AC |
|---|------|-------|--------|----|
| 1 | `crates/ecc-workflow/src/commands/memory_write.rs` | App | Replace `canonicalize` with `resolve_repo_root` + `.git` check in `resolve_project_memory_dir` | AC-001.1..5 |
| 2 | `crates/ecc-workflow/tests/memory_write.rs` | Test | Update existing hash computation in daily/memory-index assertions to use `resolve_repo_root` | AC-001.4 |
| 3 | `crates/ecc-workflow/tests/worktree_memory_path.rs` | Test | New integration test: real git worktree path resolution + non-git error | AC-001.1..3,5 |
| 4 | `crates/ecc-workflow/tests/transition.rs` | Test | Add git init to temp dir, update hash computation on line 326 | AC-001.4 |
| 5 | `crates/ecc-workflow/tests/memory_lock_contention.rs` | Test | Add git init, update project_memory_dir helper on line 22-31 | AC-001.4 |

Note: `ecc-flock` is already a dependency of `ecc-workflow` (confirmed in `Cargo.toml`).

## File Changes (dependency order)

### Change 1: `resolve_project_memory_dir` (production code)

**File**: `crates/ecc-workflow/src/commands/memory_write.rs`, lines 62-77

**Before**:
```rust
fn resolve_project_memory_dir(project_dir: &Path) -> Result<PathBuf, anyhow::Error> {
    let home = std::env::var("HOME").map_err(|_| anyhow::anyhow!("HOME env var not set"))?;
    let abs = std::fs::canonicalize(project_dir).unwrap_or_else(|_| project_dir.to_path_buf());
    let abs_str = abs.to_string_lossy();
    let project_hash = abs_str.trim_start_matches('/').replace('/', "-");
    Ok(PathBuf::from(home)
        .join(".claude/projects")
        .join(project_hash)
        .join("memory"))
}
```

**After**:
```rust
fn resolve_project_memory_dir(project_dir: &Path) -> Result<PathBuf, anyhow::Error> {
    let home = std::env::var("HOME").map_err(|_| anyhow::anyhow!("HOME env var not set"))?;
    let repo_root = ecc_flock::resolve_repo_root(project_dir);
    if !repo_root.join(".git").exists() {
        return Err(anyhow::anyhow!(
            "not a git repository: {} (resolved from {})",
            repo_root.display(),
            project_dir.display(),
        ));
    }
    let abs_str = repo_root.to_string_lossy();
    let project_hash = abs_str.trim_start_matches('/').replace('/', "-");
    Ok(PathBuf::from(home)
        .join(".claude/projects")
        .join(project_hash)
        .join("memory"))
}
```

**Why**: `resolve_repo_root` uses `git rev-parse --git-common-dir` to find the main repo root, even from a worktree. The `.git` check catches the infallible fallback case where `resolve_repo_root` returns the input path unchanged (non-git directory).

**Risk**: Medium — changes path resolution for all `write_daily` and `write_memory_index` calls. Existing tests will catch regressions.

### Change 2: Update existing test hash computation

**File**: `crates/ecc-workflow/tests/memory_write.rs`, lines 186-192

The existing test computes the expected hash using `std::fs::canonicalize`. After the fix, the binary will use `resolve_repo_root`. Since the test temp dir is not a git repo, `resolve_repo_root` will fall back to the input path, and then the `.git` check will fail.

**Fix**: Initialize the temp dir as a git repo (`git init`) so `resolve_repo_root` returns the same path as before, and the `.git` check passes.

**Before** (line 186):
```rust
let abs_proj = std::fs::canonicalize(project_dir).unwrap_or_else(|_| project_dir.to_path_buf());
```

**After**: Add `git init` before the daily subcommand step, and replace `canonicalize` with `resolve_repo_root` for hash computation:
```rust
// Initialize as git repo so resolve_repo_root succeeds
std::process::Command::new("git")
    .args(["init"])
    .current_dir(project_dir)
    .output()
    .expect("git init failed");

// ... (daily subcommand call) ...

let repo_root = ecc_flock::resolve_repo_root(project_dir);
let abs_str = repo_root.to_string_lossy();
let project_hash = abs_str.trim_start_matches('/').replace('/', "-");
```

This also requires adding `ecc-flock` as a dev-dependency of `ecc-workflow` (it is already a regular dependency, so integration tests can use it). Actually, since these are integration tests in the same crate and `ecc-flock` is already a dependency, it is available directly.

### Change 3: New worktree integration test

**File**: `crates/ecc-workflow/tests/worktree_memory_path.rs` (new)

Creates a real git repo with a worktree, runs `memory-write daily` from both paths, and asserts the files land in the same `~/.claude/projects/<hash>/memory/` directory.

Tests:
- **AC-001.1/AC-001.2**: Worktree daily + memory-index resolve to same hash as main repo
- **AC-001.3/AC-001.5**: Non-git temp dir returns error (exit code != 0, stderr contains "not a git repository")
- **AC-001.4**: Main repo (non-worktree) behavior unchanged (covered by updated `memory_write.rs` test)

## Pass Conditions (TDD order)

### Phase 1: Unit test for resolve_project_memory_dir (RED then GREEN)

Layers: [App]

| PC | Type | Description | AC | Command | Expected |
|----|------|-------------|----|---------|----------|
| PC-001 | unit | `resolve_project_memory_dir` returns error when `.git` does not exist on resolved root | AC-001.3, AC-001.5 | `cargo test -p ecc-workflow resolve_project_memory_dir_errors_on_non_git -- --nocapture` | Test passes, error message contains "not a git repository" |
| PC-002 | unit | `resolve_project_memory_dir` returns correct path for a git-initialized temp dir | AC-001.4 | `cargo test -p ecc-workflow resolve_project_memory_dir_succeeds_for_git_repo -- --nocapture` | Test passes, path contains `.claude/projects/<hash>/memory` |

**Commit**: `test: add resolve_project_memory_dir unit tests for git/non-git (RED)`
**Commit**: `feat: replace canonicalize with resolve_repo_root in resolve_project_memory_dir (GREEN)`

### Phase 2: Integration test — worktree path resolution

Layers: [App, Framework]

| PC | Type | Description | AC | Command | Expected |
|----|------|-------------|----|---------|----------|
| PC-003 | integration | `memory-write daily` from worktree produces same hash dir as from main repo | AC-001.1 | `cargo test -p ecc-workflow worktree_daily_resolves_to_main_repo_hash -- --nocapture` | Both runs write to same `daily/` directory |
| PC-004 | integration | `memory-write memory-index` from worktree produces same MEMORY.md as from main repo | AC-001.2 | `cargo test -p ecc-workflow worktree_memory_index_resolves_to_main_repo_hash -- --nocapture` | MEMORY.md exists at main-repo-hash path |
| PC-005 | integration | `memory-write daily` from non-git dir exits non-zero with error | AC-001.3, AC-001.5 | `cargo test -p ecc-workflow non_git_dir_returns_error -- --nocapture` | Exit code != 0, output contains "not a git repository" |
| PC-006 | integration | Existing `memory_write_subcommands` test still passes (main repo unchanged) | AC-001.4 | `cargo test -p ecc-workflow memory_write_subcommands -- --nocapture` | Test passes |

**Commit**: `test: add worktree memory path integration tests (RED)`
**Commit**: `feat: update existing memory_write test for git-init requirement (GREEN)`

### Phase 3: Lint and build gate

Layers: [Framework]

| PC | Type | Description | AC | Command | Expected |
|----|------|-------------|----|---------|----------|
| PC-007 | lint | Zero clippy warnings across workspace | all | `cargo clippy -- -D warnings` | Exit 0 |
| PC-008 | build | Release build succeeds | all | `cargo build --release` | Exit 0 |
| PC-009 | test | Full test suite passes | all | `cargo test` | All tests pass |

**Commit**: `refactor: boy scout cleanup near memory_write.rs` (if applicable)

## E2E Assessment

- **Touches user-facing flows?** Yes — `ecc-workflow memory-write daily` and `ecc-workflow memory-write memory-index` CLI commands
- **Crosses 3+ modules end-to-end?** No — only `ecc-workflow` and `ecc-flock` (reused, unchanged)
- **New E2E tests needed?** No — the integration tests in Phase 2 cover the full binary invocation path with real git worktrees. Existing E2E suite will be run as a gate.

## Testing Strategy

- **Unit tests** (Phase 1): `resolve_project_memory_dir` with git-initialized and non-git temp dirs, added to existing `#[cfg(test)] mod tests` in `memory_write.rs`
- **Integration tests** (Phase 2): Binary-level tests in `crates/ecc-workflow/tests/worktree_memory_path.rs` using real `git init` + `git worktree add`
- **E2E tests**: Run existing suite only

## Risks & Mitigations

- **Risk**: `git` not available in CI or test environment
  - Mitigation: `git` is required for all ECC operation; CI already has it. Tests will fail fast with a clear message if missing.

- **Risk**: `resolve_repo_root` returns a relative path, breaking the hash
  - Mitigation: `resolve_repo_root` already handles relative-to-absolute conversion internally (joins relative `--git-common-dir` output with `project_dir`). Verified in source.

- **Risk**: Existing `memory_write_subcommands` test breaks because temp dir is not a git repo
  - Mitigation: Phase 2 explicitly adds `git init` to that test before the daily/memory-index steps.

- **Risk**: `memory_lock_contention.rs` and `transition.rs` also compute global hash using `canonicalize`
  - Mitigation: Both tests need `git init` added to their temp dirs and hash computation updated to use `resolve_repo_root`. Added as File Changes 4-5. PC-009 (full test suite) catches any remaining breakage.

## Success Criteria

- [ ] `resolve_project_memory_dir` uses `ecc_flock::resolve_repo_root` instead of `canonicalize`
- [ ] Non-git directory returns `Err` with "not a git repository" message
- [ ] Worktree session produces same hash as main repo session (integration test)
- [ ] Main repo (non-worktree) behavior unchanged (existing test passes)
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes (full suite)
- [ ] `cargo build --release` succeeds

## Coverage Check

| AC | Covered by PC(s) |
|----|-------------------|
| AC-001.1 | PC-003 |
| AC-001.2 | PC-004 |
| AC-001.3 | PC-001, PC-005 |
| AC-001.4 | PC-002, PC-006 |
| AC-001.5 | PC-001, PC-005 |

All ACs covered. Zero uncovered.

## E2E Test Plan

No new E2E tests needed — integration tests cover binary invocation with real git worktrees.

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CHANGELOG.md | Project | Add entry under ### Fixed | Memory writes from worktree sessions resolve to main repo hash | US-001 |
| 2 | docs/backlog/BACKLOG.md | Project | Update BL-065 status | Mark as implemented | US-001 |
| 3 | CLAUDE.md | Project | Update test count | Increment test count after adding new tests | US-001 |

## SOLID Assessment

CLEAN — All SOLID principles pass. Consistent with ADR-0024 (ecc-workflow uses ecc-flock directly). No dependency rule violations. SRP maintained — single function with one reason to change.

## Robert's Oath Check

CLEAN — 0 oath warnings. 9 pass conditions provide proof. TDD order specified. Atomic commits planned. Rework ratio 0.12 (healthy).

## Security Notes

CLEAR — No injection (git args are hardcoded literals). No path traversal (hash construction eliminates `/`). No privilege escalation. SEC-003 (lock name validation) is a separate concern not affected by this change.

## Rollback Plan

Reverse dependency order:
1. Delete `crates/ecc-workflow/tests/worktree_memory_path.rs`
2. Revert `crates/ecc-workflow/tests/memory_write.rs` (remove git init, restore canonicalize hash)
3. Revert `crates/ecc-workflow/tests/memory_lock_contention.rs` (remove git init, restore canonicalize hash)
4. Revert `crates/ecc-workflow/tests/transition.rs` (remove git init, restore canonicalize hash)
5. Revert `crates/ecc-workflow/src/commands/memory_write.rs` (restore canonicalize, remove .git check)

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | CLEAN | 0 |
| Robert | CLEAN | 0 |
| Security | CLEAR | 0 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| AC Coverage | 95 | PASS | All 5 ACs covered by PCs |
| Execution Order | 85 | PASS | TDD dependency order correct |
| Fragility | 70 | PASS | Worktree tests need initial commit (discoverable) |
| Rollback Adequacy | 90 | PASS | All 5 files tracked in reverse order |
| Architecture Compliance | 90 | PASS | Consistent with ADR-0024 |
| Blast Radius | 95 | PASS | 5 files explicitly tracked |
| Missing Pass Conditions | 80 | PASS | Optional cargo fmt PC noted |
| Doc Plan Completeness | 70 | PASS | CHANGELOG + backlog + CLAUDE.md test count |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `crates/ecc-workflow/src/commands/memory_write.rs` | modify | US-001, AC-001.1..5 |
| 2 | `crates/ecc-workflow/tests/memory_write.rs` | modify | US-001, AC-001.4 |
| 3 | `crates/ecc-workflow/tests/worktree_memory_path.rs` | create | US-001, AC-001.1..3,5 |
| 4 | `crates/ecc-workflow/tests/transition.rs` | modify | US-001, AC-001.4 |
| 5 | `crates/ecc-workflow/tests/memory_lock_contention.rs` | modify | US-001, AC-001.4 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-29-worktree-memory-path-fix/design.md | Full design + Phase Summary |
