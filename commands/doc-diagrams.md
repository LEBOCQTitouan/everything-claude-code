---
description: Generate Mermaid diagrams from codebase analysis data. Supports inline markers, manifest-based, and auto-detected diagrams.
---

# Diagram Generation

Generate Mermaid diagrams that visualize codebase architecture, data flow, type hierarchies, and process sequences. Reads analysis output from `/doc-analyze` and produces diagrams in `docs/diagrams/` or inline in doc files.

## What This Command Does

1. Read analysis data from `docs/ARCHITECTURE.md`, `docs/API-SURFACE.md`, `docs/DEPENDENCY-GRAPH.md`
2. Scan doc files for `<!-- DIAGRAM: ... -->` markers
3. Read diagram manifest (if present) from `docs/ARCHITECTURE.md`
4. Auto-detect diagram opportunities based on codebase structure
5. Generate Mermaid diagrams and write to `docs/diagrams/` or inline at markers
6. Build `docs/diagrams/INDEX.md` catalog

## Arguments

- `--scope=<path>` — limit diagram generation to a subdirectory
- `--type=<flowchart|sequence|class|er|state>` — generate only one diagram type
- `--markers-only` — only process explicit `<!-- DIAGRAM: ... -->` markers, skip auto-detection
- `--dry-run` — report what would be generated without writing

## Prerequisites

Requires `docs/ARCHITECTURE.md` from a previous `/doc-analyze` run. If missing, suggests running `/doc-analyze` first.

## How to Request Specific Diagrams

### Inline Markers

Add HTML comment markers to any doc file:

```markdown
<!-- DIAGRAM: type=flowchart scope=src/lib title="Module Dependencies" direction=LR -->
```

Parameters: `type` (required), `scope`, `title`, `direction` (TD/LR/BT/RL).

The generated diagram appears directly below the marker, wrapped in `DIAGRAM-START`/`DIAGRAM-END` fences. Re-running the command updates the diagram in place.

### Diagram Manifest

Add a `## Diagram Manifest` section to `docs/ARCHITECTURE.md`:

```markdown
## Diagram Manifest

| ID | Type | Scope | Title | Target Doc |
|----|------|-------|-------|------------|
| dep-graph | flowchart | src/lib | Module Dependencies | DEPENDENCY-GRAPH.md |
| install-flow | sequence | install-orchestrator | Install Pipeline | ARCHITECTURE.md |
```

Manifest diagrams are written to `docs/diagrams/<id>.md`.

## When to Use

- After `/doc-analyze` to visualize the codebase structure
- When adding `<!-- DIAGRAM: ... -->` markers to doc files
- As part of the full `/doc-suite`
- After major refactoring to update architectural diagrams

## Related

- Full suite: `/doc-suite`
- Prerequisite: `/doc-analyze`
- Agent: `agents/diagram-generator.md`
- Skill: `skills/diagram-generation/SKILL.md`
