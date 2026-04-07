---
name: wave-dispatch
description: Wave dispatch logic for parallel TDD execution — worktree isolation, sequential merge, wave regression, failure handling
origin: ECC
---

# Wave Dispatch

For each wave in the wave plan:

## Pre-Wave Setup

1. Create a git tag `wave-N-start` (where N is the wave number) for rollback safety
2. Update tasks.md: mark all PCs in the wave as `red@<timestamp>`

## Single-PC Wave (Backward Compatible)

If the wave contains exactly one PC, dispatch it using the existing sequential process (no worktree isolation). This preserves current behavior exactly.

> Before dispatching each subagent, tell the user which PC is being implemented, what AC it covers, and what to expect from the TDD cycle.

1. Build the context brief using the standard headings (PC Spec, File Paths, Files to Modify, Prior PC Results, Commit Rules, TDD Cycle Rules)
2. Launch a Task with the `tdd-executor` agent (allowedTools: [Read, Write, Edit, MultiEdit, Bash, Grep, Glob])
3. The subagent executes RED → GREEN → REFACTOR and commits atomically
4. Receive the subagent's structured result: pc_id, status, red_result, green_result, refactor_result, commits, files_changed, error
   - **Commit SHA Accumulator**: After each successful subagent, accumulate all commit SHA hashes and messages. Also append each SHA and message to the campaign manifest's `## Commit Trail` table (parent orchestrator only, never subagents).
5. If the subagent returns `RED_ALREADY_PASSES` → investigate
6. If the subagent crashes or times out → report to the user immediately. Crashes do NOT consume a fix round (per Fix-Round Budget in implement.md). Do NOT auto-revert.
7. If the subagent returns `failure` → apply Fix-Round Budget (see implement.md § Fix-Round Budget). Increment the PC's `fix_round_count`. If budget not exceeded, re-dispatch with the same context brief. If budget exceeded, present diagnostic report and AskUserQuestion per the budget protocol. If user selects "Skip this PC" or "Abort", handle accordingly. Do NOT hard-STOP on first failure.

## Multi-PC Wave (Parallel Dispatch)

If the wave contains 2+ PCs, dispatch all concurrently:

1. For each PC in the wave, launch a Task with the `tdd-executor` agent using `isolation: "worktree"` on the Agent call
2. Each subagent operates in its own git worktree
3. Build context briefs as before, but "Prior PC Results" includes only PCs from completed prior waves (not same-wave PCs)
4. Wait for ALL subagents in the wave to complete before proceeding

## Post-Wave Merge

After all subagents in a wave complete:

1. Merge each subagent's worktree branch into the main branch, sequentially in PC-ID order
2. If a merge conflict is detected, STOP and report
3. If a worktree creation failed, STOP and report
4. Claude Code's automatic worktree cleanup handles temporary worktrees

## Wave Regression Verification

After all subagents in a wave complete and branches are merged:

> After each wave completes, report how many prior PCs were re-verified and the result.

1. Run ALL PC commands from waves 1 through the current wave W
2. For the first wave, only verify the wave's own PCs
3. If regression passes, mark all PCs in the wave as complete
4. If regression fails, STOP and report all PCs in wave W as potential culprits

## Wave Failure Handling

If one or more PCs in a wave fail:

1. Let all other subagents in the wave finish (their work is valid — PCs are independent)
2. Merge successful PCs' branches. Discard failed PCs' branches.
3. For each failed PC, apply Fix-Round Budget (see implement.md § Fix-Round Budget). If budget not exceeded, re-dispatch the failed PC. If budget exceeded, present diagnostic report and AskUserQuestion per the budget protocol. If user selects "Skip this PC", mark as failed and proceed. If "Abort", stop the pipeline. If "Keep trying", grant 2 more rounds and re-dispatch. Merge conflicts, worktree failures, and regression failures remain immediate STOPs — budget logic applies ONLY to subagent test failures.
4. On re-entry, re-derive the wave plan from tasks.md. Skip completed PCs. Re-dispatch only failed/incomplete PCs in the first incomplete wave.

## Batched Dispatch

When wave-analysis marks a group of PCs as "batchable" (same primary file, independent):

1. Build a **combined context brief** containing all batched PCs' specs under separate `## PC Spec` headings
2. Launch a single Task with `tdd-executor` for the entire batch
3. The tdd-executor executes each PC's RED-GREEN-REFACTOR cycle sequentially within one invocation
4. Commits are per-PC (not per-batch) — each PC gets its own commit(s)

### Fix-Round Budget in Batches

- Fix-round budget is tracked per-PC, not per-batch
- If one PC in the batch fails its fix-round budget, sibling PCs continue execution
- The failed PC is reported individually; successful PCs are committed normally
- On re-entry, only the failed PC is re-dispatched (not the whole batch)

### Fallback

If batched dispatch fails (subagent crash), fall back to sequential single-PC dispatch for each PC in the batch. This preserves existing behavior.
