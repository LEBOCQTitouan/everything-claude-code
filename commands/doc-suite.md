---
description: "Run the full documentation pipeline: analysis, generation, validation, cartography, coverage reporting, and README sync"
allowed-tools: [Bash, Task, Read, Write, Edit, Grep, Glob, LS, TodoWrite, TodoRead, AskUserQuestion, Agent]
---

# Doc Suite

Run the documentation pipeline by delegating to the `doc-orchestrator` agent.

## Arguments

`$ARGUMENTS` supports all doc-orchestrator arguments:
- `--scope=<path>` — directory to analyze (default: project root)
- `--phase=<plan|analyze|cartography|generate|validate|coverage|diagrams|readme|claude-md|all>` — run specific phase (default: all)
- `--base=<branch|commit>` — baseline for coverage diff
- `--dry-run` — report what would be written without writing
- `--comments-only` — only write doc comments into source
- `--skip-plan` — skip Phase 0 plan approval

## Execution

Dispatch to the `doc-orchestrator` agent with the user's arguments:

```
Launch doc-orchestrator agent with:
  $ARGUMENTS
```

The doc-orchestrator handles all phases including cartography delta processing (Phase 1.5). See `agents/doc-orchestrator.md` for full pipeline details and `skills/cartography-processing/SKILL.md` for the cartography phase.
