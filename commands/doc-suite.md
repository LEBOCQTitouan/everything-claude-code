---
description: Full documentation suite — source sync, analyze, generate, validate, diagrams, coverage reporting, and codemaps. Single documentation command.
---

# Documentation Suite

Comprehensive documentation analysis, generation, validation, and coverage reporting. Produces interlinked documentation in `docs/` that helps humans and AI understand the codebase.

## What This Command Does

1. **Source Sync** — sync docs from source-of-truth files (package.json, .env, openapi.yaml, Dockerfile)
2. **Analyze** — scan codebase structure, identify public API surface, detect domain concepts, map dependencies, detect diagram opportunities
3. **Generate** — write missing doc comments into source, produce module summaries, glossary, changelog, usage examples
4. **Diagrams** — generate Mermaid diagrams from analysis data, inline markers, and manifest
5. **Validate** — check doc accuracy against code, score quality, detect contradictions and duplicates
6. **Report** — calculate coverage per module, compare against baseline, flag staleness and regressions
7. **Codemaps** — generate token-lean architecture maps in `docs/CODEMAPS/`

## Arguments

- `--scope=<path>` — limit analysis to a subdirectory (default: project root)
- `--phase=<sync|analyze|generate|validate|coverage|diagrams|codemaps|all>` — run a specific phase (default: all)
- `--base=<branch|commit>` — baseline for coverage diff (default: previous run)
- `--dry-run` — show what would be written without writing
- `--comments-only` — only add doc comments to source files

## Phase Details

### Phase 1: Source Sync

Sync documentation from source-of-truth files:

| Source | Generates |
|--------|-----------|
| `package.json` scripts (or `Makefile`, `Cargo.toml`, `pyproject.toml`) | Available commands reference table |
| `.env.example` (or `.env.template`, `.env.sample`) | Environment variable documentation (required vs optional, format, examples) |
| `openapi.yaml` / route files | API endpoint reference |
| Source code exports | Public API documentation |
| `Dockerfile` / `docker-compose.yml` | Infrastructure setup docs |

Also generates or updates:
- `docs/CONTRIBUTING.md` — setup, scripts, testing, code style, PR checklist
- `docs/RUNBOOK.md` — deployment, health checks, common issues, rollback, alerting

Rules:
- **Single source of truth**: Always generate from code, never manually edit generated sections
- **Preserve manual sections**: Only update generated sections; leave hand-written prose intact
- **Mark generated content**: Use `<!-- AUTO-GENERATED -->` markers around generated sections
- **Staleness check**: Flag docs not modified in 90+ days with recent source code changes

### Phase 2: Analyze

Scan codebase structure, identify API surface, detect concepts, map dependencies.

Output files: `docs/ARCHITECTURE.md`, `docs/API-SURFACE.md`, `docs/DEPENDENCY-GRAPH.md`, `docs/GLOSSARY.md` (draft)

### Phase 3: Generate

- Write missing JSDoc/TSDoc/docstring into source files
- Generate per-module summaries (purpose, exports, dependencies)
- Finalize glossary with cross-references
- Generate CHANGELOG from git conventional commits
- Extract usage examples from test files

### Phase 4: Diagrams

Three diagram request methods:
1. **Inline markers**: `<!-- DIAGRAM: type=flowchart scope=... title="..." direction=LR -->`
2. **Diagram manifest** in `docs/ARCHITECTURE.md` (table-based)
3. **Custom registry** in `docs/diagrams/CUSTOM.md` (regenerated from source context)

Produces `docs/diagrams/INDEX.md` catalog with all generated Mermaid diagrams.

### Phase 5: Validate

5-dimension quality rubric: **Presence**, **Accuracy**, **Completeness**, **Clarity**, **Currency**

- Contradiction detection between inline docs and project-level docs
- Duplicate concept detection
- Code example verification (compile and run)
- Quality report with grades A-F per module

### Phase 6: Report

- Documentation coverage % per module (documented exports / total exports)
- Item type breakdown (functions, classes, types, constants)
- Staleness detection (code changed, docs didn't)
- Regression tracking (coverage decreased)
- Before/after comparison if baseline provided

### Phase 7: Codemaps

Generate token-lean architecture maps in `docs/CODEMAPS/`:

| File | Contents |
|------|----------|
| `architecture.md` | High-level system diagram, service boundaries, data flow |
| `backend.md` | API routes, middleware chain, service → repository mapping |
| `frontend.md` | Page tree, component hierarchy, state management flow |
| `data.md` | Database tables, relationships, migration history |
| `dependencies.md` | External services, third-party integrations, shared libraries |

Codemap format rules:
- **Token-lean** — file paths and function signatures, not full code blocks
- **< 1000 tokens** per codemap for efficient context loading
- **Diff detection** — if changes >30%, show diff and request approval before overwriting
- **Metadata header**: `<!-- Generated: YYYY-MM-DD | Files scanned: N | Token estimate: ~N -->`

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
| `CONTRIBUTING.md` | Setup, scripts, testing, code style, PR checklist |
| `RUNBOOK.md` | Deployment, health checks, common issues, rollback |
| `diagrams/INDEX.md` | Catalog of all generated Mermaid diagrams |
| `CODEMAPS/` | Token-lean architecture maps |

### Large codebases (50+ source files)

Folders with `INDEX.md` + per-module files. All files are cross-linked with relative paths.

## Execution Flow

```
Phase 1: Source Sync (sequential — reads config files, updates generated sections)
    |
Phase 2: doc-analyzer (sequential — produces module list + structure decision + diagram manifest)
    |
Phase 3-6: doc-generator + diagram-generator + doc-validator + doc-reporter (parallel)
    |
Phase 7: Codemaps (sequential — reads project structure, generates/updates CODEMAPS/)
    |
Final: doc-orchestrator assembles INDEX.md files + cross-references
    |
Console: summary with coverage %, quality grade, diagrams generated, files written
```

## Example Usage

```
User: /doc-suite

Documentation Suite Complete
  Source Sync:       3 files updated (scripts, env vars, contributing)
  Modules analyzed:  12
  Coverage:          73% (B)
  Quality grade:     B (7.4/10)
  Doc comments added: 18
  Diagrams generated: 6
  Codemaps updated:  4 (architecture, backend, data, dependencies)
  Issues found:      7 (2 HIGH, 3 MEDIUM, 2 LOW)
  Files written:     28

  Start here: docs/ARCHITECTURE.md
  Coverage:   docs/DOC-COVERAGE.md
  Quality:    docs/DOC-QUALITY.md
  Diagrams:   docs/diagrams/INDEX.md
  Codemaps:   docs/CODEMAPS/architecture.md
```

## When to Use

- Setting up documentation for a new project
- Periodic documentation health checks
- Before major releases
- After significant refactoring
- Onboarding new team members (generate docs they can read)

## Related

- Orchestrator: `agents/doc-orchestrator.md`
- Analyzer: `agents/doc-analyzer.md`
- Generator: `agents/doc-generator.md`
- Diagram Generator: `agents/diagram-generator.md`
- Validator: `agents/doc-validator.md`
- Reporter: `agents/doc-reporter.md`
- Skills: `skills/doc-analysis/`, `skills/doc-quality-scoring/`, `skills/diagram-generation/`
