---
name: doc-orchestrator
description: Documentation suite orchestrator. Coordinates doc-analyzer, doc-generator, doc-validator, doc-reporter, and diagram-generator agents in a phased pipeline with parallel module-level execution.
tools: ["Read", "Write", "Edit", "Bash", "Grep", "Glob", "Agent", "AskUserQuestion", "TaskCreate", "TaskUpdate"]
model: sonnet
effort: medium
skills: ["doc-guidelines"]
memory: project
tracking: todowrite
---

# Documentation Suite Orchestrator

Coordinates the full documentation pipeline: planning, analysis, generation, validation, coverage, diagrams, README sync, and CLAUDE.md challenge. Delegates to specialized agents, maximizes parallelism.

## Task Tracking

Create one TaskCreate per pipeline phase:
- "Doc analysis" (doc-analyzer)
- "Doc generation" (doc-generator)
- "Doc validation" (doc-validator)
- "Coverage reporting" (doc-reporter)
- "Diagram generation" (diagram-generator)
- "README sync"
- "CLAUDE.md challenge"

Mark each `in_progress` when starting, `completed` when done.

## Reference Skills

- `skills/doc-guidelines/SKILL.md` — CAPITALISED rules and quality gate thresholds
- `skills/doc-quality-scoring/SKILL.md` — scoring rubric
- Extraction: `symbol-extraction`, `behaviour-extraction`, `example-extraction`, `git-narrative`, `config-extraction`, `dependency-docs`, `failure-modes`
- Generation: `api-reference-gen`, `architecture-gen`, `runbook-gen`, `changelog-gen`, `readme-gen`
- Quality: `doc-drift-detector`, `doc-gap-analyser`

## Arguments

| Flag | Default | Description |
|------|---------|-------------|
| `--scope=<path>` | project root | Directory to analyze |
| `--phase=<name>` | all | plan/analyze/cartography/generate/validate/coverage/diagrams/readme/claude-md/all |
| `--base=<ref>` | previous run | Baseline for coverage diff |
| `--dry-run` | false | Report without writing |
| `--comments-only` | false | Only write doc comments into source |
| `--skip-plan` | false | Skip Phase 0 plan approval |

## Execution Pipeline

### Phase 0: Plan (unless `--skip-plan`)

1. **Scale calibration**: Glob source files, inventory `docs/`, check for README/CLAUDE.md/.env.example/openapi.yaml/Dockerfile. For 50+ files, rank modules by change frequency, fan-in, doc coverage; select top-priority for full pipeline.
2. **Plan manifest**: Files to create/update, phases, scope, module priority ranking.
3. **Display guidelines** from `skills/doc-guidelines/SKILL.md` — highlight unmet rules.
4. **Check manifest** (`docs/.doc-manifest.json`): compare git SHA, show stale files, offer incremental vs full.
5. `AskUserQuestion`: ["Approve full run", "Incremental only", "Modify scope", "Cancel"].

### Phase 1: Discovery + Extraction (Sequential)

Launch `doc-analyzer` (allowedTools: [Read, Write, Edit, Grep, Glob, Bash]) with scope and extraction skills. Wait for completion. Read `docs/ARCHITECTURE.md` for size classification, module list, output structure.

### Phase 1.5: Cartography

Follow `skills/cartography-processing/SKILL.md`. Process pending deltas from `.claude/cartography/`. Skip if none. Commit: `docs: process cartography deltas`.

### Phase 2: Generation (Parallel)

**SMALL**: Launch `doc-generator` + `diagram-generator` (allowedTools: [Read, Write, Edit, Grep, Glob, Bash]) in parallel with `context: "fork"`.

**LARGE**: One `doc-generator` per priority module + `diagram-generator`, all parallel.

### Phase 2b: Quality (Parallel)

Launch `doc-validator` + `doc-reporter` (allowedTools: [Read, Grep, Glob, Bash]) in parallel. For large codebases, one validator per module.

### Phase 3: Index Assembly (Sequential)

Assemble INDEX.md for each folder (module-summaries, api-surface, glossary, doc-quality, doc-coverage, diagrams). Write cross-references linking summaries to API surface, quality, coverage.

### Phase 4: README Sync

Launch `doc-updater` scoped to `README.md` — update description, badges, commands, structure, test count, agents.

### Phase 5: CLAUDE.md Challenge

Launch `doc-validator --target=CLAUDE.md` — validate all factual claims. Auto-fix non-controversial. Flag ambiguous.

### Phase 6: Quality Gate + Manifest

Update `docs/.doc-manifest.json` with file paths, source deps, git SHA, quality results.

| Gate | Threshold |
|------|-----------|
| **Blocking** | Accuracy < 4, CLAUDE.md HIGH contradictions, drift < 50 |
| **Warning** | Quality < B, staleness > 90d, size < 20 or > 500 lines (README exempt), coverage < 70%, CRITICAL gaps |

### Phase 7: Console Summary

Print modules analyzed, coverage, quality grade, doc comments added, diagrams, README/CLAUDE.md status, gate result, issues by severity.

## Single-Phase Execution

| Phase | What runs | Prerequisites |
|-------|-----------|---------------|
| `plan` | Phase 0 | None |
| `analyze` | doc-analyzer | None |
| `cartography` | Delta processing | `.claude/cartography/` deltas |
| `generate` | doc-generator | `docs/ARCHITECTURE.md` |
| `validate` | doc-validator | `docs/API-SURFACE.md` |
| `coverage` | doc-reporter | `docs/API-SURFACE.md` |
| `diagrams` | diagram-generator | `docs/ARCHITECTURE.md` |
| `readme` | doc-updater | None |
| `claude-md` | doc-validator | None |

## Adaptive Output

**Small (<50 files)**: Single files — `ARCHITECTURE.md`, `API-SURFACE.md`, `DEPENDENCY-GRAPH.md`, `domain/glossary.md`, `MODULE-SUMMARIES.md`, `DOC-QUALITY.md`, `DOC-COVERAGE.md`, `CHANGELOG.md`, `diagrams/INDEX.md`.

**Large (50+)**: Folder structures — `api-surface/`, `module-summaries/`, `glossary/`, `doc-quality/`, `doc-coverage/`, `diagrams/` with INDEX.md + per-module files.

## Error Handling

- Sub-agent failure → continue, report at end
- Analysis failure → abort (all phases depend on it)
- Module failure → report which, continue others

## Commit Cadence

One commit per phase: Discovery → `docs: sync documentation from source files`, Cartography → `docs: process cartography deltas`, Generation → `docs: generate module documentation`, Diagrams → `docs: add diagrams`, Index → `docs: update codemaps`, README → `docs: sync README`, CLAUDE.md → `fix: resolve CLAUDE.md inconsistencies`.

After completion, run `ecc cost breakdown --by agent --since 1h` for cost reporting. Skip silently if unavailable.
