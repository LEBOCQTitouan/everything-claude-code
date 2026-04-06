---
id: BL-050
title: Deferred pipeline summary tables — coverage delta, bounded contexts, per-test-name
status: implemented
created: 2026-03-22
promoted_to: ""
tags: [spec, design, implement, summary, deferred, coverage, bounded-contexts]
scope: MEDIUM
target_command: /spec-dev
---

## Optimized Prompt

Three pipeline summary tables were deferred from BL-048 because they require logic changes beyond instruction additions:

1. **Coverage delta table** for `/implement` — requires `cargo tarpaulin` or equivalent coverage tooling to compute before/after test coverage percentages. Needs new tooling integration.

2. **Bounded contexts table** for `/design` — requires a new analysis step to enumerate bounded contexts affected by the design. Currently not collected during the design phase.

3. **Per-test-name inventory** for `/implement` — requires subagent return schema changes to report individual test names from each RED-GREEN cycle, not just pass/fail counts.

Each item can be implemented independently. All three were explicitly carved out of BL-048 scope per spec Decision #1 to avoid violating the "instruction additions only, no phase logic changes" constraint.
