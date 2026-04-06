---
description: Generate multi-channel DevRel content from codebase. Delegates to comms-generator agent.
allowed-tools: Read, Write, Edit, Bash, Grep, Glob, Agent, AskUserQuestion, TodoWrite
---

# /comms-generate

Generate multi-channel DevRel content drafts. Delegates to `comms-generator`.

## Usage

`/comms-generate [channel...]` — Valid: `social`, `blog`, `devblog`, `docs-site`. Default: all.

> **Tracking**: TodoWrite checklist. If unavailable, proceed without tracking.

## Steps

1. **Parse** channel args (default: all 4). Unknown → error.
2. **Check** `comms/` existence (agent scaffolds if needed).
3. **Delegate** to `comms-generator` with channel list.
4. **Report**: generated files, channels processed, any blocked output (CRITICAL redaction).

## Notes

- Output to `comms/drafts/{channel}/` — never published directly
- CRITICAL redaction (secrets) blocks output
- Review/promote via `/comms drafts`
