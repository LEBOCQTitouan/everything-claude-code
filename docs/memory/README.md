# Cross-Session Memory System

File-based, deterministic memory for ECC. Records action metadata and work item artifacts across Claude Code sessions.

## Directory Structure

```
docs/memory/
├── README.md              # This file (checked into git)
├── action-log.json        # Append-only action log (gitignored)
└── work-items/            # Per-work-item artifacts (gitignored)
    └── YYYY-MM-DD-slug/
        ├── plan.md
        ├── solution.md
        └── implementation.md
```

## Action Log Schema

File: `action-log.json` — JSON array, append-only.

Each entry:

```json
{
  "timestamp": "ISO-8601",
  "session_id": "from CLAUDE_SESSION_ID",
  "action_type": "plan | solution | implement | verify | fix | audit | review | other",
  "description": "feature description from state.json",
  "artifacts": ["relative/path/to/file"],
  "outcome": "success | partial | failed | skipped",
  "tags": []
}
```

Rules: append-only, no mutations, no secrets, no file contents.

## Work Item Files

Directory: `work-items/YYYY-MM-DD-<slug>/`

Each file has fixed H2 sections:

- **plan.md**: ## Context, ## Decisions, ## User Stories, ## Outcome
- **solution.md**: ## Context, ## File Changes, ## Pass Conditions, ## Outcome
- **implementation.md**: ## Context, ## Changes Made, ## Test Results, ## Outcome

Rules: deterministic paths (date + slug), deterministic sections, write-once (re-entry appends ## Revision block).

## Consumer Access Rules

Only these designated consumers should read memory files:

- **drift-checker** agent — reads action log to detect plan drift over time
- **catchup** command (BL-017) — reads action log + work items to reconstruct session context
- **robert** agent (BL-004) — reads past implementation summaries for negative examples

No other agents or commands should read or reference memory files. Memory is NOT injected into orchestrators, planners, or general hooks. Consumers opt-in by explicitly reading the files they need.

## Writer

Memory is written by `.claude/hooks/memory-writer.sh`, called from `phase-transition.sh` after each workflow phase completes.
