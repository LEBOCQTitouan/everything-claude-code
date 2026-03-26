---
id: BL-067
title: Deterministic spec/design artifact validation — AC format, PC table, coverage mapping
status: open
scope: HIGH
target: /spec dev
created: 2026-03-26
tags: [deterministic, validation, spec, design, rust-cli]
related: [BL-029]
---

# BL-067: Deterministic Spec/Design Artifact Validation

## Problem

The solution-adversary and spec-adversary agents spend significant LLM tokens on mechanical validation:

1. **AC format validation** — checking `AC-NNN.N` numbering is sequential, no gaps/duplicates
2. **PC table format validation** — checking PC rows have 6 columns, IDs are sequential
3. **AC coverage mapping** — extracting ACs from spec, mapping to PCs in solution, reporting uncovered ACs
4. **PC dependency order** — verifying PCs that modify the same file are in correct dependency order

These are all pattern-matching and graph operations — perfectly suited for deterministic code.

## Proposed Solution

### `ecc validate spec <path>`
- Parse markdown to find `## Acceptance Criteria` section
- Extract `AC-NNN.N: <description>` lines via regex
- Validate sequential numbering, no gaps, no duplicates
- Return structured list of ACs

### `ecc validate solution <path> [--spec <spec-path>]`
- Parse PC table from solution.md
- Validate column count (6), sequential PC IDs, required fields
- If `--spec` provided: cross-reference ACs, report uncovered ACs
- Parse File Changes table, build file-overlap matrix
- Topological sort PCs by file dependencies, flag ordering violations

### Output Format
```json
{
  "ac_count": 12,
  "pc_count": 8,
  "uncovered_acs": ["AC-003.2"],
  "ordering_violations": [{"pc": "PC-005", "depends_on": "PC-002", "reason": "both modify src/lib.rs"}]
}
```

## Impact

- **Reliability**: Binary pass/fail on structural correctness (LLM ~85% accuracy on large specs)
- **Speed**: < 100ms validation vs 10-30s LLM analysis
- **Agent simplification**: solution-adversary.md can focus on semantic review, not structural checks
- **Compound**: Every spec/design cycle saves ~2K tokens on validation alone

## Research Context

OpenHands pattern: typed events with schema validation at boundaries.
Augment Code: "Deterministic guardrails validate, then LLM reasons."
