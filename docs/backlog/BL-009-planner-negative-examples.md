---
id: BL-009
title: Add negative examples to planner agent
tier: 2
scope: LOW
target: direct edit
status: implemented
created: 2026-03-20
file: agents/planner.md
---

## Action

The planner has 0 DO NOT lines. Add at least 3:

- "DO NOT produce plans with phases that can't be independently tested"
- "DO NOT skip the risk assessment for any phase regardless of perceived simplicity"
- "DO NOT create horizontal slices — every phase must be a vertical tracer bullet from interface to persistence"
