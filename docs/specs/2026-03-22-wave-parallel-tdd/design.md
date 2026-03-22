# Design: Wave-based parallel TDD execution (BL-032)

## Overview

Add wave-based parallel PC dispatch to `/implement` Phase 2-3 and update Phase 7 output schema. The wave grouper scans PCs left-to-right, groups adjacent PCs with no file overlap, caps at 4 per wave, and dispatches parallel PCs via `isolation: "worktree"`. Merge, regression, failure handling, and progress tracking are all wave-aware. Single-PC waves behave identically to current sequential dispatch (backward compatible).

## File Changes

| # | File | Action | Spec Ref | Dependency Order |
|---|------|--------|----------|------------------|
| 1 | `commands/implement.md` | modify | US-001..US-005 | — |
| 2 | `tests/test-wave-parallel.sh` | create | US-001..US-005 | after #1 |
| 3 | `docs/adr/0012-wave-parallel-tdd.md` | create | US-006 AC-006.1 | after #1 |
| 4 | `docs/domain/glossary.md` | modify | US-006 AC-006.2 | after #1 |
| 5 | `CHANGELOG.md` | modify | US-006 AC-006.3 | after #1 |

## Architecture

No Rust code changes. No tdd-executor agent changes. All changes are in `commands/implement.md` (Markdown command content) plus documentation.

### Sections Modified in implement.md

| Section | Change | Lines Added (est.) |
|---------|--------|--------------------|
| Phase 2: Parse Solution | Add "Wave Analysis" subsection after existing extraction | ~60 |
| Phase 3: TDD Loop | Replace sequential dispatch with wave-aware dispatch | ~120 (net ~80 after removing ~40 sequential lines) |
| Phase 7: implement-done.md | Add Wave column to TDD Log + Subagent Execution | ~15 |
| Constraints | Add wave-related constraints | ~5 |

**Total net addition**: ~160 lines. implement.md goes from ~412 to ~572 (well under 800).

## Pass Conditions

| PC ID | Type | Description | Verifies AC | Command | Expected |
|-------|------|-------------|-------------|---------|----------|
| PC-001 | content | Wave Analysis subsection exists in Phase 2 | AC-001.1, AC-001.2 | `bash tests/test-wave-parallel.sh test_wave_analysis_section` | PASS |
| PC-002 | content | Left-to-right scan algorithm documented | AC-001.2, AC-001.3 | `bash tests/test-wave-parallel.sh test_wave_grouping_algorithm` | PASS |
| PC-003 | content | Max 4 sub-batch splitting documented | AC-001.4 | `bash tests/test-wave-parallel.sh test_max_concurrency_cap` | PASS |
| PC-004 | content | Wave plan display format documented | AC-001.5 | `bash tests/test-wave-parallel.sh test_wave_plan_display` | PASS |
| PC-005 | content | Degenerate cases documented (all overlap, all independent, single PC) | AC-001.6, AC-001.7, AC-001.8 | `bash tests/test-wave-parallel.sh test_degenerate_cases` | PASS |
| PC-006 | content | Worktree isolation dispatch documented | AC-002.1 | `bash tests/test-wave-parallel.sh test_worktree_dispatch` | PASS |
| PC-007 | content | Sequential merge in PC-ID order documented | AC-002.2 | `bash tests/test-wave-parallel.sh test_sequential_merge` | PASS |
| PC-008 | content | Single-PC wave backward compatibility documented | AC-002.3 | `bash tests/test-wave-parallel.sh test_single_pc_backward_compat` | PASS |
| PC-009 | content | Prior PC Results scoping for parallel waves documented | AC-002.4 | `bash tests/test-wave-parallel.sh test_prior_results_scoping` | PASS |
| PC-010 | content | Merge conflict handling documented | AC-002.5, AC-002.6 | `bash tests/test-wave-parallel.sh test_merge_error_handling` | PASS |
| PC-011 | content | Wave-level regression replaces per-PC regression | AC-003.1, AC-003.2, AC-003.3, AC-003.4 | `bash tests/test-wave-parallel.sh test_wave_regression` | PASS |
| PC-012 | content | Let-wave-finish failure semantics documented | AC-004.1, AC-004.2 | `bash tests/test-wave-parallel.sh test_failure_semantics` | PASS |
| PC-013 | content | Re-entry with wave awareness documented | AC-004.3 | `bash tests/test-wave-parallel.sh test_reentry_wave_aware` | PASS |
| PC-014 | content | Git tags at wave boundaries documented | AC-004.4, AC-004.5 | `bash tests/test-wave-parallel.sh test_git_tags` | PASS |
| PC-015 | content | Wave-aware tasks.md status updates documented | AC-005.1, AC-005.2 | `bash tests/test-wave-parallel.sh test_wave_tasks_tracking` | PASS |
| PC-016 | content | Wave column in implement-done.md schema (conditional) | AC-005.3, AC-005.4 | `bash tests/test-wave-parallel.sh test_implement_done_wave_column` | PASS |
| PC-017 | content | implement.md stays under 800 lines | all | `bash tests/test-wave-parallel.sh test_line_count` | PASS |
| PC-018 | doc | ADR 0012 exists with required sections | AC-006.1 | `bash tests/test-wave-parallel.sh test_adr_0012` | PASS |
| PC-019 | doc | Glossary has Wave and Wave Plan terms | AC-006.2 | `bash tests/test-wave-parallel.sh test_glossary_terms` | PASS |
| PC-020 | doc | CHANGELOG has BL-032 entry | AC-006.3 | `bash tests/test-wave-parallel.sh test_changelog_entry` | PASS |
| PC-021 | meta | All existing pipeline tests still pass | backward compat | `bash tests/test-pipeline-summaries.sh` | PASS (0 failures) |

## Test Strategy (TDD Order)

The TDD order groups related PCs to build up implement.md incrementally. Each PC adds grep-testable content.

### Wave 1: Foundation (PC-001, PC-002, PC-003, PC-004, PC-005)
All touch only `commands/implement.md` Phase 2 section and `tests/test-wave-parallel.sh`. No file overlap between them because they test different grep patterns in the same file. However, since they all modify `commands/implement.md`, they must be sequential.

**Order**: PC-001 -> PC-002 -> PC-003 -> PC-004 -> PC-005

### Wave 2: Dispatch (PC-006, PC-007, PC-008, PC-009, PC-010)
All touch `commands/implement.md` Phase 3 section. Sequential for same reason.

**Order**: PC-006 -> PC-007 -> PC-008 -> PC-009 -> PC-010

### Wave 3: Regression + Failure (PC-011, PC-012, PC-013, PC-014)
All touch `commands/implement.md` Phase 3 section. Sequential.

**Order**: PC-011 -> PC-012 -> PC-013 -> PC-014

### Wave 4: Progress + Schema (PC-015, PC-016)
Touch `commands/implement.md` Phase 3 and Phase 7. Sequential.

**Order**: PC-015 -> PC-016

### Wave 5: Guard (PC-017)
Line count validation. Runs after all implement.md changes.

**Order**: PC-017

### Wave 6: Docs (PC-018, PC-019, PC-020)
Independent files: ADR, glossary, CHANGELOG. Could be parallel but kept sequential for simplicity.

**Order**: PC-018 -> PC-019 -> PC-020

### Wave 7: Backward Compat (PC-021)
Runs existing test suite.

**Order**: PC-021

## Detailed Content Design

### Phase 2 Addition: Wave Analysis (after "Generate tasks.md")

New subsection `### Wave Analysis` containing:

1. **File overlap computation**: For each PC, extract "Files to Modify" from File Changes table. Two PCs overlap if they share any file path.
2. **Left-to-right scan**: Walk PCs in Test Strategy order. Start wave W=1. For each PC: if no file overlaps with any PC in current wave, add to wave. If overlap, close wave, increment W, start new wave with this PC.
3. **Sub-batch splitting**: If a wave has > 4 PCs, split into sub-batches of 4. Sub-batches within a wave run sequentially (each sub-batch of 4 runs in parallel, then next sub-batch).
4. **Wave plan display**: Print wave plan showing PC IDs, files, parallelism factor.
5. **Degenerate case handling**:
   - All PCs share a file -> each wave has 1 PC (fully sequential, wave machinery adds no overhead)
   - All PCs independent -> one wave (sub-batched by 4)
   - Single PC -> one wave, one PC

**Key constant**: `MAX_CONCURRENT_SUBAGENTS = 4`

### Phase 3 Replacement: Wave-Aware TDD Loop

Replace the current sequential "For each PC" with wave-based dispatch:

```
For each wave W in wave plan:
  1. Create git tag: wave-W-start
  2. If wave has 1 PC:
     - Dispatch sequentially (current behavior, no worktree)
  3. If wave has > 1 PC:
     - For each sub-batch (max 4):
       - Update tasks.md: red@timestamp for all PCs in sub-batch
       - Launch N tdd-executor subagents concurrently with isolation: "worktree"
       - Context brief: "Prior PC Results" includes only PCs from waves 1..W-1
       - Wait for all to complete (let-finish semantics)
       - If any failed: merge successful branches in PC-ID order, discard failed, STOP
       - If all succeeded: merge branches in PC-ID order
  4. Wave-level regression: run ALL PC commands from waves 1..W
  5. Update tasks.md: done@timestamp for all PCs in wave W, mark [x]
  6. Update TodoWrite + TaskUpdate for all PCs in wave W
```

**Merge conflict handling**: If merge fails, STOP with: conflicting PCs, files, suggestion to resolve manually.

**Worktree failure handling**: If worktree creation fails (disk space, git error), STOP with error and remediation.

### Phase 7 Update: Conditional Wave Column

In implement-done.md schema:
- If any wave had > 1 PC: add `Wave` column to TDD Log and Subagent Execution tables
- If all waves had 1 PC: omit Wave column (backward compatible)

### Re-Entry (Phase 0 Update)

Add to Phase 0 re-entry logic:
- On re-entry, re-derive wave plan from Test Strategy + File Changes
- Skip completed PCs (marked `[x]` in tasks.md)
- Re-dispatch only failed/incomplete PCs in their original wave positions
- If a wave is partially complete, dispatch only the remaining PCs from that wave

## E2E Activation Rules

No E2E tests required. Pure command file content changes with no port/adapter boundaries crossed.

## Doc Update Plan

| # | Doc File | Level | Action |
|---|----------|-------|--------|
| 1 | `docs/adr/0012-wave-parallel-tdd.md` | architecture | Create — worktree isolation, wave grouping, failure handling, rollback |
| 2 | `docs/domain/glossary.md` | domain | Add "Wave" and "Wave Plan" terms |
| 3 | `CHANGELOG.md` | project | Add BL-032 feature entry |
