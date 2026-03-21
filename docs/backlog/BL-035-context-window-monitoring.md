---
id: BL-035
title: Add context window usage monitoring
status: open
created: 2026-03-21
scope: MEDIUM
target_command: skills/strategic-compact, hooks
tags: [gsd, context, monitoring, quality, compaction]
---

## Optimized Prompt

Enhance the strategic-compact skill and suggest-compact hook to estimate context window usage from message count and approximate token length. Warn at 70% capacity with a yellow indicator. At 85%, strongly suggest splitting to a new session or running /compact. Currently the hook is time/edit-based only — it doesn't know how full the context is. Add a `context-usage` utility function to the hook that counts conversation turns and estimates tokens (rough: 4 chars per token). This prevents the quality degradation observed in long sessions where later outputs are measurably worse than early ones.

## Framework Source

- **GSD**: Monitors context window fill percentage and warns when quality may degrade

## Related Backlog Items

- None
