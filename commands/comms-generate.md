---
description: Generate multi-channel DevRel content from codebase. Delegates to comms-generator agent.
allowed-tools: Read, Write, Edit, Bash, Grep, Glob, Agent, AskUserQuestion, TodoWrite
---

# /comms-generate

Generate multi-channel DevRel content drafts from the current codebase. Delegates to the `comms-generator` agent.

## Usage

```
/comms-generate [channel...]
```

Valid channels: `social`, `blog`, `devblog`, `docs-site`. Default: all channels.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `channel...` | No | One or more channels to generate. Omit to generate all 4. |

## What This Command Does

> **Tracking**: Create a TodoWrite checklist. If TodoWrite is unavailable, proceed without tracking.

TodoWrite items:
- "Parse channel arguments"
- "Check comms repo existence"
- "Delegate to comms-generator agent"
- "Report results"

Mark each item complete as the step finishes.

1. **Parse** the channel arguments from the command input. If no channels are provided, default to all four: `social`, `blog`, `devblog`, `docs-site`. If unknown channels are given, report an error listing valid options and stop.

2. **Check** for the existence of the `comms/` directory in the current project. If it does not exist, the `comms-generator` agent will scaffold it in Phase 1 — no pre-flight needed here.

3. **Delegate** to the `comms-generator` agent, passing the resolved channel list as context:
   - Instruct the agent to run only for the specified channels
   - The agent handles: scaffold (if needed), context gathering, draft generation, redaction, and calendar update

4. **Report** results after the agent completes:
   - List files generated (path per draft)
   - Channels processed
   - Any blocked output due to CRITICAL redaction patterns

## Examples

```
# Generate all channels
/comms-generate

# Generate only social media content
/comms-generate social

# Generate blog and devblog
/comms-generate blog devblog
```

## Notes

- All output goes to `comms/drafts/{channel}/` — never published directly
- If a CRITICAL redaction pattern is found (API keys, secrets), output is blocked for that draft
- To review and promote drafts, use `/comms drafts`
- No Plan Mode — this is quick generation, not a pipeline
