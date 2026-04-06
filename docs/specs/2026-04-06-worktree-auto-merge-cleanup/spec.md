# Spec: Worktree Auto-Merge and Cleanup Enforcement

## Problem Statement

After completing work in a worktree, the merge to main is best-effort (warns on failure, never blocks) and the worktree directory is deliberately preserved after merge to avoid orphaning Claude Code's CWD. This leads to stale worktrees accumulating on disk, branches lingering, and work potentially stranded on unmerged branches. The cleanup is deferred to `ecc worktree gc` which runs at next session start but uses coarse heuristics (age + PID) without checking merge status or safety. The system needs mandatory merge with safety-checked auto-deletion, enforced in the workflow.

## Research Summary

- **Two-phase delete pattern**: Mark worktree as merged, then delete when process is no longer using it. Safest for session-scoped worktrees.
- **Atomic merge-then-cleanup**: Canonical sequence is rebase -> verify -> ff-merge -> safety check -> remove worktree + delete branch. All as a single logical operation with rollback on failure.
- **Never delete unclean worktrees**: Git itself refuses `worktree remove` if dirty unless `--force`. CLI tools should check 3+ conditions before removal.
- **GC sweep as complement**: Periodic GC sweep catches edge cases (crashed sessions, failed cleanups). Not a replacement for post-merge cleanup.
- **Branch naming enables safe detection**: Session-scoped branches with predictable prefixes (`ecc-session-*`) allow GC to distinguish from user branches.
- **SQLite/marker files for state**: Track worktree state (merged, pending-delete) to handle multi-session scenarios.
- **Config sync is a hidden pitfall**: Each worktree shares `.git/` but not working directory configs — important for safety check scope.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Merge is mandatory/blocking at session end | "Force" means always attempt, never silently skip. On failure: preserve worktree, report clearly. | Yes (combined) |
| 2 | 5-point safety check before deletion | Uncommitted changes, untracked files, unmerged commits, stash, remote push. Maximum paranoia per user preference. | Yes (combined) |
| 3 | CWD changed to main repo root before deletion | User chose this over deferred-to-gc. Feasible since `ecc-workflow merge` already operates on repo root for ff-merge. | Yes (combined) |
| 4 | No backwards-compat flag — just change the default | Merge now always cleans up. Simpler, no `--no-cleanup` needed. Only caller is the session hook. | Yes (combined) |
| 5 | New `WorktreeManager` port (ISP) | Keeps `GitInfo` focused (single method). Worktree ops are a cohesive set. | Yes (combined) |
| 6 | GC refactored to use `WorktreeManager` port | Eliminates raw shell git calls in app layer (pre-existing arch smell). | Yes (combined) |
| 7 | `ecc worktree status` command added | Shows all worktrees with merge status, safety info, and age. Useful for debugging. | No (CLI addition) |
| 8 | Single ADR covering all decisions | Architecturally significant: new port, changed default behavior, domain rule. | Yes |
| 9 | ecc-workflow merge gathers safety data via raw git commands (not WorktreeManager port) | ecc-workflow is a standalone binary using std::process::Command; adding port dependency would couple it to the hex stack, contradicting the deferred migration in Non-Requirements. The domain `assess_safety` function receives gathered data as a struct. | Yes (combined) |
| 10 | Safety check collects all failures (not short-circuit) | AC-003.5 requires listing all failed checks, so collect-all is necessary. Return type is `Vec<UnsafeReason>`, empty means safe. | No |
| 11 | CWD change uses `Command::current_dir()`, not `std::env::set_current_dir` | Process-global CWD change is unsafe in concurrent/test contexts. Per-command `current_dir()` is isolated and testable. | No |

## User Stories

### US-001: Domain Safety Check Rule

**As a** developer, **I want** the system to determine whether a worktree is safe to delete based on 5 conditions, **so that** no work is ever lost during automated cleanup.

#### Acceptance Criteria

- AC-001.1: Given a worktree with uncommitted changes, when safety is assessed, then status is `UnsafeUncommittedChanges`
- AC-001.2: Given a worktree with untracked files, when safety is assessed, then status is `UnsafeUntrackedFiles`
- AC-001.3: Given a worktree with commits not reachable from main, when safety is assessed, then status is `UnsafeUnmergedCommits`
- AC-001.4: Given a worktree with stashed changes, when safety is assessed, then status is `UnsafeStashedChanges`
- AC-001.5: Given a worktree branch not pushed to remote, when safety is assessed, then status is `UnsafeUnpushedCommits`
- AC-001.6: Given a clean worktree with all commits merged, pushed, no stash, and no untracked files, then status is `SafeToDelete`
- AC-001.7: The safety assessment is a pure function in `ecc-domain` with zero I/O. Input is a `WorktreeSafetyInput` struct with fields: `has_uncommitted_changes: bool`, `has_untracked_files: bool`, `unmerged_commit_count: u64`, `has_stash: bool`, `is_pushed_to_remote: bool`
- AC-001.8: Given a worktree with multiple unsafe conditions, all conditions are returned (not short-circuited). Return type is `Vec<UnsafeReason>`, empty means safe

#### Dependencies

- Depends on: none

### US-002: WorktreeManager Port

**As a** developer, **I want** a dedicated port trait for worktree operations, **so that** worktree management follows hexagonal architecture and is testable.

#### Acceptance Criteria

- AC-002.1: A `WorktreeManager` trait exists in `ecc-ports` with methods: `has_uncommitted_changes`, `has_untracked_files`, `unmerged_commit_count`, `has_stash`, `is_pushed_to_remote`, `remove_worktree`, `delete_branch`, `list_worktrees`
- AC-002.2: An `OsWorktreeManager` implementation exists in `ecc-infra` using git CLI commands
- AC-002.3: A `MockWorktreeManager` exists in `ecc-test-support` for unit testing
- AC-002.4: All methods use `--` before user-supplied paths/branches (argument injection prevention)

#### Dependencies

- Depends on: none

### US-003: Post-Merge Cleanup in ecc-workflow

**As a** developer, **I want** `ecc-workflow merge` to automatically delete the worktree and branch after a successful merge, **so that** stale worktrees don't accumulate.

#### Acceptance Criteria

- AC-003.1: After successful ff-merge, the safety check is run against the worktree
- AC-003.2: If all 5 safety checks pass (`SafeToDelete`), the worktree directory is removed via `git worktree remove`
- AC-003.3: If safety check passes, the branch is deleted via `git branch -d`
- AC-003.4: Before deletion, CWD is changed to the main repo root
- AC-003.5: If any safety check fails, the worktree and branch are preserved with a clear warning listing which checks failed
- AC-003.6: If `git worktree remove` fails (e.g., permission error), the error is reported but does not cause the merge to be considered failed
- AC-003.7: The success message changes from "preserved (cleanup deferred)" to "cleaned up successfully"
- AC-003.8: If branch deletion fails after successful worktree removal, the error is logged as a warning. The overall cleanup is still considered successful. The orphaned branch is left for GC to clean up
- AC-003.9: If CWD change to repo root fails (e.g., directory missing or inaccessible), cleanup is aborted and worktree is preserved with a warning
- AC-003.10: Safety check data is gathered via `std::process::Command` in ecc-workflow (consistent with existing patterns), NOT via the WorktreeManager port. The domain `assess_safety` function receives gathered data as a `WorktreeSafetyInput` struct
- AC-003.11: Safety check runs AFTER merge lock acquisition, inside the critical section, to prevent concurrent safety-check-vs-write races
- AC-003.12: If worktree directory does not exist (manually deleted), `git worktree remove` is still called to prune git metadata, and branch deletion proceeds

#### Dependencies

- Depends on: US-001, US-002 (domain types only — ecc-workflow uses raw commands but the `WorktreeSafetyInput` and `assess_safety` from US-001 are consumed directly)

### US-004: GC Refactored to Use WorktreeManager Port

**As a** developer, **I want** the existing GC use case to use the `WorktreeManager` port instead of raw shell commands, **so that** the app layer doesn't construct git CLI invocations directly.

#### Acceptance Criteria

- AC-004.1: `ecc-app/src/worktree.rs` GC function accepts `&dyn WorktreeManager` instead of `&dyn ShellExecutor` for worktree operations
- AC-004.2: `list_worktrees` replaces the manual `git worktree list --porcelain` parsing
- AC-004.3: `remove_worktree` + `delete_branch` replace the manual git commands
- AC-004.4: Existing GC tests are migrated to use `MockWorktreeManager`
- AC-004.5: GC behavior is unchanged (staleness by age + PID)
- AC-004.6: GC also checks merge status for worktrees (enhancement: don't delete unmerged worktrees even if stale). "Unmerged" means `git branch --merged main` does not include the worktree branch
- AC-004.7: The `--force` flag overrides merge-status safety, restoring pre-existing behavior of deleting stale worktrees regardless of merge status

#### Dependencies

- Depends on: US-002

### US-005: Worktree Status Command

**As a** developer, **I want** `ecc worktree status` to show all worktrees with their merge state, safety info, and age, **so that** I can inspect worktree health before manual intervention.

#### Acceptance Criteria

- AC-005.1: `ecc worktree status` lists all session worktrees with columns: Name, Branch, Age, Commits Ahead, Clean, Stash, Pushed, Status (merged/unmerged/stale)
- AC-005.2: Non-session worktrees are excluded from the output
- AC-005.3: Output is human-readable table format
- AC-005.4: Command returns exit code 0 on success
- AC-005.5: Output uses tab-separated columns. A snapshot test validates the exact header and column order

#### Dependencies

- Depends on: US-002

### US-006: Session Hook Update

**As a** developer, **I want** the session-end merge hook to invoke the cleanup flow and report results clearly, **so that** the merge+cleanup is mandatory and visible.

#### Acceptance Criteria

- AC-006.1: The session_end_merge handler calls `ecc-workflow merge` (which now includes cleanup)
- AC-006.2: On successful merge+cleanup, the message says "merged and cleaned up"
- AC-006.3: On merge failure, behavior is unchanged (preserve worktree, write recovery file)
- AC-006.4: On merge success but cleanup failure (safety check blocked), the message lists which checks failed
- AC-006.5: Bypass via `ECC_WORKFLOW_BYPASS=1` still works

#### Dependencies

- Depends on: US-003

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `ecc-domain/src/worktree.rs` | Domain | Add `WorktreeSafetyStatus` enum + `assess_safety` pure function |
| `ecc-ports/src/worktree.rs` (new) | Ports | New `WorktreeManager` trait |
| `ecc-ports/src/lib.rs` | Ports | Re-export `worktree` module |
| `ecc-infra/src/os_worktree.rs` (new) | Infra | `OsWorktreeManager` implementation |
| `ecc-test-support/src/mock_worktree.rs` (new) | Test Support | `MockWorktreeManager` |
| `ecc-workflow/src/commands/merge.rs` | Standalone Binary | Add safety check + cleanup after merge |
| `ecc-app/src/worktree.rs` | Application | Refactor GC to use `WorktreeManager` port |
| `ecc-app/src/hook/handlers/tier3_session/session_merge.rs` | Application | Update messages for cleanup flow |
| `ecc-cli/src/commands/worktree.rs` | CLI | Add `status` subcommand |

## Constraints

- `ecc-domain` must have zero I/O imports — safety assessment is pure
- `ecc-workflow` binary uses `std::process::Command` directly (pre-existing pattern, acceptable)
- CWD change before deletion must work from the `ecc-workflow` binary context
- Merge lock serialization must be maintained (concurrent sessions)

## Non-Requirements

- Migrating `ecc-workflow merge` into the hexagonal stack (Phase 2, deferred)
- Remote branch deletion
- Interactive confirmation prompts
- Retry logic on merge failure
- Windows-specific worktree cleanup (junction point handling)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| `WorktreeManager` (new) | New port + adapter | New integration tests with real git repos |
| `GitInfo` | Unchanged | No impact |
| Session hook | Updated behavior | Existing hook tests updated |
| CLI worktree | New subcommand | New CLI integration test |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New port | Architecture | ARCHITECTURE.md | Add WorktreeManager to port list |
| New command | CLI Reference | CLAUDE.md | Add `ecc worktree status` |
| Behavior change | Gotchas | CLAUDE.md | Update worktree lifecycle description |
| ADR | Decision log | docs/adr/ | New ADR for auto-deletion |

## Open Questions

None — all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | What is explicitly OUT of scope? | Exclude GC refactor migration of merge into hex stack and remote branch deletion. Include status command and GC refactor to WorktreeManager port. | User |
| 2 | What does "force merge" mean? | Mandatory + blocking: always attempt merge, never silently skip. On failure: preserve worktree, report clearly. | Recommended |
| 3 | Which safety checks before deletion? | All 5: uncommitted changes, untracked files, unmerged commits, stash, remote push. Maximum paranoia. | User |
| 4 | How to handle CWD orphaning? | cd to main repo root before deleting worktree. | User |
| 5 | Test strategy split? | 100% on safety + orchestration, 80% on CLI/status/GC. Real git repos for integration tests. | Recommended |
| 6 | Security implications? | No security concerns. Local-only git ops, no auth/secrets/network. | Recommended |
| 7 | Breaking changes approach? | Just change the default. No backwards-compat flag. | User |
| 8 | ADR decisions? | Single ADR covering all decisions (port, domain rule, CWD handling, default change). | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Domain Safety Check Rule | 8 | none |
| US-002 | WorktreeManager Port | 4 | none |
| US-003 | Post-Merge Cleanup in ecc-workflow | 12 | US-001, US-002 |
| US-004 | GC Refactored to Use WorktreeManager Port | 7 | US-002 |
| US-005 | Worktree Status Command | 5 | US-002 |
| US-006 | Session Hook Update | 5 | US-003 |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | Uncommitted changes → UnsafeUncommittedChanges | US-001 |
| AC-001.2 | Untracked files → UnsafeUntrackedFiles | US-001 |
| AC-001.3 | Unmerged commits → UnsafeUnmergedCommits | US-001 |
| AC-001.4 | Stashed changes → UnsafeStashedChanges | US-001 |
| AC-001.5 | Unpushed commits → UnsafeUnpushedCommits | US-001 |
| AC-001.6 | All clean → SafeToDelete | US-001 |
| AC-001.7 | Pure function with WorktreeSafetyInput struct | US-001 |
| AC-001.8 | Collect all failures (Vec), not short-circuit | US-001 |
| AC-002.1 | WorktreeManager trait with 8 methods | US-002 |
| AC-002.2 | OsWorktreeManager in ecc-infra | US-002 |
| AC-002.3 | MockWorktreeManager in ecc-test-support | US-002 |
| AC-002.4 | Argument injection prevention (--) | US-002 |
| AC-003.1 | Safety check after ff-merge | US-003 |
| AC-003.2 | SafeToDelete → git worktree remove | US-003 |
| AC-003.3 | SafeToDelete → git branch -d | US-003 |
| AC-003.4 | CWD to repo root before deletion | US-003 |
| AC-003.5 | Unsafe → preserve + list failed checks | US-003 |
| AC-003.6 | Worktree remove failure → warning, not merge failure | US-003 |
| AC-003.7 | Success message: "cleaned up successfully" | US-003 |
| AC-003.8 | Branch delete failure → warning, cleanup still successful | US-003 |
| AC-003.9 | CWD change failure → abort cleanup, preserve | US-003 |
| AC-003.10 | Raw commands in ecc-workflow, not port | US-003 |
| AC-003.11 | Safety check inside merge lock critical section | US-003 |
| AC-003.12 | Missing worktree dir → prune metadata + branch delete | US-003 |
| AC-004.1 | GC accepts &dyn WorktreeManager | US-004 |
| AC-004.2 | list_worktrees replaces porcelain parsing | US-004 |
| AC-004.3 | Port methods replace manual git commands | US-004 |
| AC-004.4 | Tests migrated to MockWorktreeManager | US-004 |
| AC-004.5 | GC behavior unchanged (age + PID) | US-004 |
| AC-004.6 | Don't delete unmerged (branch --merged main) | US-004 |
| AC-004.7 | --force overrides merge-status safety | US-004 |
| AC-005.1 | Status table with 8 columns | US-005 |
| AC-005.2 | Non-session worktrees excluded | US-005 |
| AC-005.3 | Human-readable table format | US-005 |
| AC-005.4 | Exit code 0 on success | US-005 |
| AC-005.5 | Tab-separated + snapshot test | US-005 |
| AC-006.1 | Hook calls ecc-workflow merge (now with cleanup) | US-006 |
| AC-006.2 | Success: "merged and cleaned up" | US-006 |
| AC-006.3 | Merge failure: unchanged behavior | US-006 |
| AC-006.4 | Cleanup failure: list failed checks | US-006 |
| AC-006.5 | Bypass via ECC_WORKFLOW_BYPASS=1 | US-006 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Ambiguity | 72→85 | PASS | Input struct and collect-all semantics now specified |
| Edge Cases | 48→80 | PASS | Partial cleanup, CWD failure, missing dir, concurrent races addressed |
| Scope Creep Risk | 85 | PASS | Well-bounded by specific non-requirements + Windows exclusion |
| Dependency Gaps | 40→82 | PASS | Decision 9 resolves ecc-workflow / port boundary |
| Testability | 70→80 | PASS | Command::current_dir() avoids process-global; snapshot test specified |
| Decision Completeness | 58→85 | PASS | Decisions 9-11 close all gaps |
| Rollback & Failure | 62→80 | PASS | AC-003.8/9 cover partial cleanup and CWD failure paths |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-06-worktree-auto-merge-cleanup/spec.md | Full spec + Phase Summary |
