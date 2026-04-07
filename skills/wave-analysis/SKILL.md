---
name: wave-analysis
description: Wave grouping algorithm for parallel TDD execution — left-to-right scan, adjacent grouping, concurrency cap
origin: ECC
---

# Wave Analysis

After generating tasks.md, analyze the dependency graph between PCs to enable parallel execution.

## Algorithm

Scan PCs left-to-right in Test Strategy order. Start a new wave. For each subsequent PC, add it to the current wave if it shares no files (from "Files to Modify") with any PC already in the wave. If it does share files, close the current wave and start a new one. This produces waves of adjacent, file-independent PCs.

## Concurrency Cap

If a wave contains more than 4 PCs, split it into sub-batches of maximum 4 concurrent subagents.

## Wave Plan Display

Before proceeding to Phase 3, display the wave plan to the user:

> **Wave Plan**: Show each wave with its PC IDs, files affected, and parallelism factor. Example: "Wave 1: PC-003, PC-004, PC-005 [3 parallel] | Wave 2: PC-006 [sequential]"

## Degenerate Cases

- **All PCs overlap** (same file): each PC gets its own wave — fully sequential. Wave machinery is skipped.
- **All PCs independent**: one wave containing all PCs (split into sub-batches of 4 if > 4).
- **Single-PC implementation**: one wave with one PC — no overhead.

## Same-File Batching

After wave grouping, within each wave, identify PCs that share the same primary target file AND have no inter-PC dependency. These are candidates for **batched dispatch** — multiple PCs sent to a single tdd-executor invocation.

### Independence Definition

Two PCs are "independent" if neither references output artifacts (test files, implementation files, types) created by the other. Specifically:
- PC-A's test file must not import or reference anything created by PC-B
- PC-A's implementation must not depend on PC-B's implementation existing first
- No shared mutable state between the two PCs' test or implementation files

### Batch Identification

For each wave:
1. Group PCs by their primary `## Files to Modify` target
2. Within each group, check pairwise independence
3. Mark fully independent same-file groups as "batchable"
4. Non-independent groups stay as separate sequential dispatches within the wave

### Batch Limits

Maximum 4 PCs per batch (same as wave concurrency cap). Larger groups are split into sub-batches.
