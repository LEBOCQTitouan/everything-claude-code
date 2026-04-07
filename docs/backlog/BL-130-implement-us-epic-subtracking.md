---
id: BL-130
title: "US/epic-level sub-tracking within /implement pipeline"
scope: MEDIUM
target: "/spec-dev"
status: open
created: "2026-04-07"
tags: [workflow, implement, tracking, user-stories, epics]
---

## Context

The current /implement pipeline tracks progress at PC (pass condition) level. For larger features with multiple user stories or epics, there's no way to see which US is complete vs in-progress. Users need visibility into implementation progress at the US/epic level, not just individual test conditions.

## Prompt

Add US/epic-level sub-tracking to the /implement pipeline:

1. **US progress**: Group PCs by their source US (from "Verifies AC" column mapping back to US). Display US-level completion percentage.
2. **Epic decomposition**: For EPIC-scope items, support sub-grouping by US within the tasks.md tracker.
3. **Progress dashboard**: `ecc workflow progress` command showing US-level completion bars.
4. **Resume at US level**: When re-entering /implement, show which US is current and its completion state.

## Acceptance Criteria

- [ ] tasks.md groups PCs by US
- [ ] Progress display shows per-US completion
- [ ] Re-entry identifies current US
- [ ] `ecc workflow progress` shows US-level dashboard
