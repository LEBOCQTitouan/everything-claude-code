---
id: BL-070
title: Deterministic wave grouping algorithm — PC parallelization from file-overlap analysis
status: implemented
scope: MEDIUM
target: /spec dev
created: 2026-03-26
tags: [deterministic, tdd, wave-dispatch, rust-cli]
related: [BL-032]
---

# BL-070: Deterministic Wave Grouping Algorithm

## Problem

The wave-dispatch skill describes a wave grouping algorithm for parallel TDD execution:
- Group consecutive PCs with no file overlap into waves (max 4 per wave)
- LLM manually reads PC file changes, evaluates overlaps, forms groups

This is a graph algorithm (file-overlap matrix + greedy left-to-right scan) — perfectly suited for deterministic code.

## Proposed Solution

### `ecc wave-plan <solution-path>`

1. Parse solution.md File Changes table
2. Build PC → files mapping
3. Compute file-overlap matrix: `overlap(PC-A, PC-B) = files(A) ∩ files(B) ≠ ∅`
4. Greedy left-to-right scan:
   - Start new wave
   - Add next PC if no file overlap with any PC in current wave AND wave size < 4
   - Otherwise start new wave
5. Output wave plan:

```json
{
  "waves": [
    {"id": 1, "pcs": ["PC-001", "PC-002", "PC-003"]},
    {"id": 2, "pcs": ["PC-004"]},
    {"id": 3, "pcs": ["PC-005", "PC-006"]}
  ],
  "sequential_fallback": false
}
```

### Integration
- `/implement` Phase 3 calls `ecc wave-plan` instead of LLM evaluation
- LLM only handles override decisions (e.g., forcing sequential due to semantic dependencies)

## Impact

- **Reliability**: Deterministic grouping prevents conservative single-PC waves (LLM often under-parallelizes)
- **Speed**: < 10ms computation vs 5-10s LLM analysis
- **Correctness**: File overlap detection is binary — no judgment needed

## Research Context

This matches the spectrum model: deterministic routing + LLM override for edge cases.
