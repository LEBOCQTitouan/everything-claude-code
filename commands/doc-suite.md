---
description: Full documentation suite — plan, source sync, analyze, generate, validate, diagrams, coverage reporting, codemaps, README sync, and CLAUDE.md challenge. Single documentation command.
---

# Documentation Suite

### Phase 0: Prompt Refinement

Before executing, analyze the user's input using the `prompt-optimizer` skill:
1. Identify intent and match to available ECC skills/commands/agents
2. Check for ambiguity or missing context
3. Rewrite the task description for clarity and specificity
4. Display the refined prompt to the user

If the refined prompt differs significantly, show both original and refined versions.
Proceed with the refined version unless the user objects.

**FIRST ACTION**: Unless `--skip-plan` is passed, call the `EnterPlanMode` tool immediately. This enters Claude Code plan mode which restricts tools to read-only exploration while you scan the codebase and draft a documentation plan. After presenting the plan, call `ExitPlanMode` to proceed with execution after user approval.

Comprehensive documentation analysis, generation, validation, and coverage reporting. Produces interlinked documentation in `docs/` that helps humans and AI understand the codebase.

## What This Command Does

0. **Plan** — scan codebase, draft documentation plan manifest, display DOCUMENTATION GUIDELINES, wait for approval
1. **Source Sync** — sync docs from source-of-truth files (package.json, .env, openapi.yaml, Dockerfile)
2. **Analyze** — scan codebase structure, identify public API surface, detect domain concepts, map dependencies, detect diagram opportunities
3. **Generate** — write missing doc comments into source, produce module summaries, glossary, changelog, usage examples
4. **Diagrams** — generate Mermaid diagrams from analysis data, inline markers, and manifest
5. **Validate** — check doc accuracy against code, score quality, detect contradictions and duplicates
6. **Report** — calculate coverage per module, compare against baseline, flag staleness and regressions
7. **Codemaps** — generate token-lean architecture maps in `docs/CODEMAPS/`
8. **README Sync** — update README.md with current project state
9. **CLAUDE.md Challenge** — verify every factual claim in CLAUDE.md against the codebase

## Arguments

- `--scope=<path>` — limit analysis to a subdirectory (default: project root)
- `--phase=<plan|sync|analyze|generate|validate|coverage|diagrams|codemaps|readme|claude-md|all>` — run a specific phase (default: all)
- `--base=<branch|commit>` — baseline for coverage diff (default: previous run)
- `--dry-run` — show what would be written without writing
- `--comments-only` — only add doc comments to source files
- `--skip-plan` — skip Phase 0 plan approval and execute directly (backward compat)

## Reference Skills

- `skills/doc-guidelines/SKILL.md` — CAPITALISED documentation guidelines and quality gate thresholds
- `skills/doc-quality-scoring/SKILL.md` — scoring rubric (Presence, Accuracy, Completeness, Clarity, Currency)

## Phase Details

### Phase 0: Plan

**Only runs if `--skip-plan` is NOT passed.** Enters plan mode for user review before execution.

1. **Lightweight codebase scan**:
   - Glob source files to count and classify (small vs large codebase)
   - Inventory existing docs in `docs/`
   - Check for `README.md`, `CLAUDE.md`, `.env.example`, `openapi.yaml`, `Dockerfile`
2. **Produce plan manifest**:
   - Files to create (with estimated purpose)
   - Files to update (with what changes)
   - Phases to run (which are relevant for this codebase)
   - Estimated scope (number of modules, files)
3. **Display DOCUMENTATION GUIDELINES** from `skills/doc-guidelines/SKILL.md`:
   - Show all CAPITALISED rules
   - Highlight which guidelines are currently unmet
4. **Wait for user approval**, then call `ExitPlanMode`

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

**Naming Expressiveness Audit** (sub-phase):
- Scan all exported identifiers (functions, classes, types, constants) for generic names: `Manager`, `Handler`, `Processor`, `Data`, `Info`, `Helper`, `Util`, `Service` (without domain prefix), `Base`, `Abstract` (as standalone)
- Calculate per-module naming score: `(total_exports - generic_names) / total_exports`
- Flag modules with naming score < 0.8 as "naming expressiveness concern"
- Include in Phase 2 output alongside API surface data

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
- File size validation per `skills/doc-guidelines/SKILL.md` thresholds

**Comment Quality Classification**:
Classify every comment in changed/analyzed files:

| Category | Action | Example |
|----------|--------|---------|
| **Informative** | Keep | `// RFC 7231 requires this header format` |
| **Redundant** | Flag for removal | `// increment counter` above `counter += 1` |
| **Misleading** | CRITICAL — fix immediately | Comment says X, code does Y |
| **Apologetic** | Track as tech debt | `// sorry, this is a hack` |
| **Mandated** | Validate accuracy | Required JSDoc/rustdoc for public API |
| **Journaling** | Flag for removal | `// Added by John on 2024-01-15` |

Include comment quality summary in validation report: count per category, highlight CRITICAL misleading comments.

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

### Phase 8: README Sync

Update `README.md` with current project state:

- Project description and badges
- Commands table (verify all commands still exist, update descriptions)
- Repository structure tree (add new directories, remove deleted ones)
- Test count (from latest test run)
- Agent list (scan `agents/` for current agents)
- Installation and usage instructions (verify accuracy)

Uses `doc-updater` agent scoped to `README.md`.

### Phase 9: CLAUDE.md Challenge

Verify every factual claim in `CLAUDE.md` against the codebase:

- **Test commands**: Do `npm run build`, `npm test`, individual test commands actually work?
- **npm scripts table**: Does it match `package.json` scripts?
- **Directory structure**: Do listed directories exist? Are descriptions accurate?
- **Command table**: Do listed commands match `commands/*.md`?
- **File counts**: Are stated counts (test count, agent count) accurate?
- **Development notes**: Are all stated conventions still true?

Severity levels:
- **HIGH**: Commands or scripts that would fail if copy-pasted
- **MEDIUM**: Outdated counts, missing entries, stale descriptions
- **LOW**: Minor wording drift, style inconsistencies

Auto-fix non-controversial items (counts, directory listings). Flag ambiguous findings for user review.

**"The Last Page" Check** — verify claims are actual, not aspirational:
- **Convention claims**: For each stated convention (e.g., "we use immutable patterns"), grep the codebase for counter-examples. If violations > 10%, flag as "aspirational, not actual"
- **Dependency graph claims**: Compare the described dependency graph with actual import analysis. Flag mismatches.
- **Test coverage claims**: Compare stated coverage targets with actual coverage numbers. Flag gaps > 5%.
- **Architecture claims**: Verify that described architecture patterns (e.g., "hexagonal architecture") match the actual directory structure and import patterns.

The goal is to ensure CLAUDE.md describes what the codebase **is**, not what it **aspires to be**.

Uses `doc-validator` agent with `--target=CLAUDE.md`.

## Quality Gates

After all phases complete, check quality gates from `skills/doc-guidelines/SKILL.md`:

### Blocking (pipeline fails)

| Condition | Threshold |
|-----------|-----------|
| Accuracy score | < 4 (out of 10) |
| CLAUDE.md contradictions | Any HIGH severity |

### Warning (pipeline passes with warnings)

| Condition | Threshold |
|-----------|-----------|
| Quality grade | Below B (< 7.0/10) |
| Staleness | Any doc > 90 days stale |
| File size violation | Outside 20-500 line range |
| Coverage | Below 70% documented exports |

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
Phase 0: Plan (unless --skip-plan) — scan codebase, draft manifest, show guidelines, wait for approval
    |
Phase 1: Source Sync (sequential — reads config files, updates generated sections)
    |
Phase 2: doc-analyzer (sequential — produces module list + structure decision + diagram manifest)
    |
Phase 3-6: doc-generator + diagram-generator + doc-validator + doc-reporter (parallel)
    |
Phase 7: Codemaps (sequential — reads project structure, generates/updates CODEMAPS/)
    |
Phase 8: README Sync (sequential — doc-updater scoped to README.md)
    |
Phase 9: CLAUDE.md Challenge (sequential — doc-validator with --target=CLAUDE.md)
    |
Quality Gates: Check blocking/warning thresholds from doc-guidelines skill
    |
Final: doc-orchestrator assembles INDEX.md files + cross-references
    |
Console: summary with coverage %, quality grade, diagrams generated, files written,
         README sync status, CLAUDE.md results, quality gate outcome, top 3 guidelines
```

## Example Usage

```
User: /doc-suite

[Phase 0: Plan mode — scans codebase, presents plan manifest]

Documentation Plan
  Source files:        47 (small codebase)
  Existing docs:       8 files in docs/
  Phases to run:       all (1-9)
  Files to create:     6
  Files to update:     12

  DOCUMENTATION GUIDELINES:
  - ALWAYS DOCUMENT PUBLIC API ENDPOINTS AND THEIR REQUEST/RESPONSE SCHEMAS
  - ALWAYS DOCUMENT ENVIRONMENT VARIABLES WITH REQUIRED VS OPTIONAL STATUS
  - ALWAYS DOCUMENT SETUP AND ONBOARDING STEPS
  ... (7 more)

  Currently unmet: 3 guidelines (ADRs, error codes, deployment procedures)

Approve? [y/n]

User: y

[Phases 1-9 execute]

Documentation Suite Complete
  Source Sync:         3 files updated (scripts, env vars, contributing)
  Modules analyzed:    12
  Coverage:            73% (B)
  Quality grade:       B (7.4/10)
  Doc comments added:  18
  Diagrams generated:  6
  Codemaps updated:    4 (architecture, backend, data, dependencies)
  README synced:       yes (test count, commands table, structure updated)
  CLAUDE.md challenge: 2 fixes applied, 0 flagged
  Quality gates:       PASSED (1 warning: coverage below 70% in hooks/)
  Issues found:        7 (2 HIGH, 3 MEDIUM, 2 LOW)

  Top guidelines to address:
  1. ALWAYS DOCUMENT ERROR CODES AND THEIR MEANINGS
  2. ALWAYS DOCUMENT DEPLOYMENT AND ROLLBACK PROCEDURES
  3. ALWAYS DOCUMENT ARCHITECTURAL DECISIONS AS ADRS

  Start here: docs/ARCHITECTURE.md
  Coverage:   docs/DOC-COVERAGE.md
  Quality:    docs/DOC-QUALITY.md
  Diagrams:   docs/diagrams/INDEX.md
  Codemaps:   docs/CODEMAPS/architecture.md
```

### Skip plan mode

```
User: /doc-suite --skip-plan

[Phases 1-9 execute directly without approval step]
```

## Commit Cadence

| Trigger | Commit Message |
|---------|---------------|
| After Phase 1 (Source Sync) | `docs: sync documentation from source files` |
| After Phase 3 (Generate) | `docs: generate module documentation` |
| After Phase 4 (Diagrams) | `docs: add diagrams` |
| After Phase 7 (Codemaps) | `docs: update codemaps` |
| After Phase 8 (README Sync) | `docs: sync README` |
| After Phase 9 (if fixes applied) | `fix: resolve CLAUDE.md inconsistencies` |

Each commit is atomic — one concern per commit. Do not batch multiple phases into a single commit.

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
- Skills: `skills/doc-analysis/`, `skills/doc-quality-scoring/`, `skills/doc-guidelines/`, `skills/diagram-generation/`
