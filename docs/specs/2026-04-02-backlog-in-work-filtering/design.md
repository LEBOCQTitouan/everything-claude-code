# Solution: BL-097 Backlog In-Work Filtering for /spec Picker

## Spec Reference
Concern: dev, Feature: BL-097 Backlog in-work filtering

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `.gitignore` | Modify | Add `docs/backlog/.locks/` to prevent lock files from being committed | US-003 |
| 2 | `commands/spec.md` | Modify | Add worktree scan, lock check, active sessions display, --show-all flag, lock write to Phase 0 | US-001, US-002, US-003 |
| 3 | `commands/implement.md` | Modify | Add lock release step after implement-done.md is written | US-003, AC-003.2 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | spec.md contains worktree scan reference | AC-001.1 | `grep -q '.claude/worktrees' commands/spec.md` | Exit 0 |
| PC-002 | unit | spec.md contains BL-NNN regex pattern | AC-001.1 | `grep -q 'bl-.*\\d\\{3\\}\|(?i)bl' commands/spec.md` | Exit 0 |
| PC-003 | unit | spec.md contains filtering of matched items | AC-001.2 | `grep -qi 'exclude.*matched\|filter.*claimed\|remove.*in-progress' commands/spec.md` | Exit 0 |
| PC-004 | unit | spec.md shows unmatched worktrees as info | AC-001.3 | `grep -qi 'active sessions\|unmatched.*worktree\|no BL-NNN' commands/spec.md` | Exit 0 |
| PC-005 | unit | spec.md checks lock files | AC-001.4 | `grep -q 'docs/backlog/.locks' commands/spec.md` | Exit 0 |
| PC-006 | unit | spec.md contains --show-all flag | AC-001.5 | `grep -q '\-\-show-all' commands/spec.md` | Exit 0 |
| PC-007 | unit | spec.md contains active sessions display | AC-002.1 | `grep -qi 'Active sessions' commands/spec.md` | Exit 0 |
| PC-008 | unit | spec.md contains hidden items count | AC-002.2 | `grep -qi 'hidden\|in progress' commands/spec.md` | Exit 0 |
| PC-009 | unit | spec.md writes lock file on selection | AC-003.1 | `grep -q '.lock' commands/spec.md` | Exit 0 |
| PC-010 | unit | implement.md contains lock release | AC-003.2 | `grep -qi 'lock.*release\|lock.*remove\|lock.*delete\|remove.*lock' commands/implement.md` | Exit 0 |
| PC-011 | unit | spec.md contains stale lock cleanup | AC-003.3 | `grep -qi 'stale\|24.*hour\|TTL' commands/spec.md` | Exit 0 |
| PC-012 | unit | spec.md contains orphaned lock cleanup | AC-003.4 | `grep -qi 'orphan\|worktree.*no longer\|worktree.*deleted' commands/spec.md` | Exit 0 |
| PC-013 | unit | .gitignore contains .locks | - | `grep -q '.locks' .gitignore` | Exit 0 |
| PC-014 | integration | ecc validate commands passes | AC-001.1 | `./target/release/ecc validate commands` | Exit 0 |
| PC-015 | build | cargo build | All | `cargo build --workspace` | Exit 0 |
| PC-016 | lint | cargo clippy | All | `cargo clippy --workspace -- -D warnings` | Exit 0 |
| PC-017 | integration | cargo test | All | `cargo test --workspace` | Exit 0 |

### Coverage Check

All 11 ACs covered:

| AC | Covering PCs |
|---|---|
| AC-001.1 | PC-001, PC-002 |
| AC-001.2 | PC-003 |
| AC-001.3 | PC-004 |
| AC-001.4 | PC-005 |
| AC-001.5 | PC-006 |
| AC-002.1 | PC-007 |
| AC-002.2 | PC-008 |
| AC-003.1 | PC-009 |
| AC-003.2 | PC-010 |
| AC-003.3 | PC-011 |
| AC-003.4 | PC-012 |

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | /spec picker | Command | N/A | Verify filtering with active worktrees | ignored | spec.md Phase 0 modified |

### E2E Activation Rules
No automated E2E tests — command behavior is verified manually.

## Test Strategy

TDD order:
1. **PC-013** — .gitignore (foundation)
2. **PC-001 to PC-012** — spec.md and implement.md content checks
3. **PC-014** — ecc validate commands
4. **PC-015 to PC-017** — Build/clippy/test gates

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/adr/0041-session-detection-pattern.md` | MEDIUM | Create | ADR for worktree-scan + lock file convention | Decision #1 |
| 2 | `docs/adr/README.md` | LOW | Modify | Add 0041 entry | Decision #1 |
| 3 | `CHANGELOG.md` | LOW | Modify | Add BL-097 entry | All |

## SOLID Assessment
**PASS** — Content-only (command Markdown edits). No domain, port, or adapter changes.

## Robert's Oath Check
**CLEAN** — 17 PCs provide proof. Phased approach (gitignore → spec.md → implement.md). No mess.

## Security Notes
**CLEAR** — Lock files contain only worktree name + timestamp. No secrets, no auth, no user data.

## Rollback Plan

1. Revert `commands/implement.md` lock release addition
2. Revert `commands/spec.md` Phase 0 filtering additions
3. Revert `.gitignore` .locks entry
4. Delete `docs/backlog/.locks/` if created

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | PASS | 0 |
| Robert | CLEAN | 0 |
| Security | CLEAR | 0 |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `.gitignore` | Modify | US-003 |
| 2 | `commands/spec.md` | Modify | US-001, US-002, US-003 |
| 3 | `commands/implement.md` | Modify | US-003 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| `docs/specs/2026-04-02-backlog-in-work-filtering/design.md` | Full design + Phase Summary |
