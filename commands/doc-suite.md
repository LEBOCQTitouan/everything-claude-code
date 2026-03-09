---
description: Full documentation suite — analyze, generate, validate, and report on codebase documentation. Invokes the doc-orchestrator agent.
---

# Documentation Suite

Comprehensive documentation analysis, generation, validation, and coverage reporting. Produces interlinked documentation in `docs/` that helps humans and AI understand the codebase.

## What This Command Does

1. **Analyze** — scan codebase structure, identify public API surface, detect domain concepts, map dependencies, detect diagram opportunities
2. **Generate** — write missing doc comments into source, produce module summaries, glossary, changelog, usage examples
3. **Diagrams** — generate Mermaid diagrams from analysis data, inline markers, and manifest
4. **Validate** — check doc accuracy against code, score quality, detect contradictions and duplicates
5. **Report** — calculate coverage per module, compare against baseline, flag staleness and regressions

## Arguments

- `--scope=<path>` — limit analysis to a subdirectory (default: project root)
- `--phase=<analyze|generate|validate|coverage|diagrams|all>` — run a specific phase (default: all)
- `--base=<branch|commit>` — baseline for coverage diff (default: previous run)
- `--dry-run` — show what would be written without writing
- `--comments-only` — only add doc comments to source files

## Output

Adaptive structure based on codebase size:

### Small codebases (<50 source files)

Single markdown files in `docs/`:

| File | Contents |
|------|----------|
| `ARCHITECTURE.md` | Project profile, module overview, system diagram |
| `API-SURFACE.md` | All public exports with doc status |
| `DEPENDENCY-GRAPH.md` | Module dependency graph with doc annotations |
| `GLOSSARY.md` | Domain and infrastructure term definitions |
| `MODULE-SUMMARIES.md` | Per-module purpose, exports, dependencies |
| `DOC-QUALITY.md` | Quality scores, contradictions, issues |
| `DOC-COVERAGE.md` | Coverage %, trends, staleness report |
| `CHANGELOG.md` | Generated from git conventional commits |
| `diagrams/INDEX.md` | Catalog of all generated Mermaid diagrams |
| `diagrams/<id>.md` | Individual diagram files (flowchart, sequence, class, etc.) |

### Large codebases (50+ source files)

Folders with `INDEX.md` + per-module files. Each folder (api-surface/, module-summaries/, glossary/, doc-quality/, doc-coverage/, diagrams/) has an index linking to individual module docs.

All files are cross-linked with relative paths.

## Execution Flow

```
Phase 1: doc-analyzer (sequential)
    ↓ produces module list + structure decision + diagram manifest
Phase 2: doc-generator + doc-validator + doc-reporter + diagram-generator (parallel)
    ↓ each agent writes to its own files/directories
Phase 3: doc-orchestrator assembles INDEX.md files + cross-references
    ↓
Console: summary with coverage %, quality grade, diagrams generated, files written
```

## Example Usage

```
User: /doc-suite

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Documentation Suite Complete
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Modules analyzed:    12
  Coverage:            73% (B)
  Quality grade:       B (7.4/10)
  Doc comments added:  18
  Diagrams generated:  6
  Issues found:        7 (2 HIGH, 3 MEDIUM, 2 LOW)
  Files written:       24
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  Start here: docs/ARCHITECTURE.md
  Coverage:   docs/DOC-COVERAGE.md
  Quality:    docs/DOC-QUALITY.md
  Diagrams:   docs/diagrams/INDEX.md
```

## Individual Phase Commands

Run phases independently:
- `/doc-analyze` — analysis only
- `/doc-generate` — generation only (requires analysis first)
- `/doc-diagrams` — diagram generation only (requires analysis first)
- `/doc-validate` — validation only (requires analysis first)
- `/doc-coverage` — coverage reporting only (requires analysis first)

## When to Use

- Setting up documentation for a new project
- Periodic documentation health checks
- Before major releases
- After significant refactoring
- Onboarding new team members (generate docs they can read)

## Difference from /update-docs

| | `/update-docs` | `/doc-suite` |
|---|---|---|
| Scope | Script reference, env vars, CONTRIBUTING, RUNBOOK | Full codebase analysis + generation + validation |
| Output | Updates existing doc sections | Generates complete doc suite in `docs/` |
| Depth | Surface-level (reads config files) | Deep (reads all source code, tests, git history) |
| Validation | Staleness check only | Accuracy, quality scoring, contradictions |
| Coverage | None | Per-module %, trends, regressions |

Use `/update-docs` for quick config-based doc refreshes. Use `/doc-suite` for comprehensive documentation.

## Related

- Orchestrator: `agents/doc-orchestrator.md`
- Analyzer: `agents/doc-analyzer.md`
- Generator: `agents/doc-generator.md`
- Diagram Generator: `agents/diagram-generator.md`
- Validator: `agents/doc-validator.md`
- Reporter: `agents/doc-reporter.md`
- Skills: `skills/doc-analysis/`, `skills/doc-quality-scoring/`, `skills/diagram-generation/`
