# Spec: Domain-Specialized Agent Generator with Pipeline Injection

## Problem Statement

ECC pipeline commands operate with generic software engineering knowledge but no deep understanding of the specific repository's domain model, module boundaries, error patterns, or naming conventions. When `/spec` analyzes a feature touching the `workflow` bounded context, it doesn't know about `WorkflowState`, `TransitionPolicy`, or the specific error handling patterns used there. This leads to generic advice instead of codebase-grounded recommendations. There is no mechanism to generate repository-specialized agents or inject their knowledge into the pipeline.

## Research Summary

- **Codified Context pattern**: Production frameworks use 19+ specialized domain-expert agents with domain knowledge embedded directly in agent specifications (~50% of agent content)
- **AGENTS.md standard**: Persistent project-specific guidance files bootstrap codebase-aware agents with conventions that can't be inferred from code alone
- **Codebase packaging tools** (Repomix, Codebase-digest): Extract structured overviews from repositories into AI-friendly formats for context injection
- **Multi-agent decomposition**: Specialize agents by domain rather than function for deeper expertise
- **Skills as domain injection**: Claude Code skills enable packaging domain expertise as loadable modules
- **RAG + static analysis**: Combine code relationship extraction with retrieval for accurate, traceable documentation

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | LLM-reads-code approach (not static analysis CLI) | Faster to implement, aligns with existing agent pattern | No |
| 2 | `agents/domain/` subdirectory for generated agents | Separates generated from hand-crafted agents | No |
| 3 | `generated: true` frontmatter marker | Enables validation, staleness detection, regeneration idempotency | Yes |
| 4 | Pipeline auto-injection via Phase 0.7 in each command | Single injection point after Phase 0.5 (Sources), before Phase 1 | Yes |
| 5 | Read-only tools (Read, Grep, Glob) for domain agents | Domain agents provide context, not code changes | No |
| 6 | Require `docs/domain/bounded-contexts.md` as input | Clean contract, avoids guessing module boundaries | No |
| 7 | Parallel generation (3 concurrent subagents) | Faster for multi-module projects | No |
| 8 | Manual re-generation with staleness detection | Auto-regeneration via hooks is too much scope for v1 | No |
| 9 | bounded-contexts.md parsed for both table rows and freestanding ### sections | File has two formats; both must be discovered | No |
| 10 | Pipeline injection is Phase 0.7 (not Phase 0.5) | Avoids collision with existing Phase 0.5 Sources Consultation | No |
| 11 | Module name matching uses exact module name match, not substring | Prevents false positives (e.g., "config" matching "configuration") | No |

## User Stories

### US-001: Domain Agent Generator Command

**As a** developer, **I want** a `/generate-domain-agents` command that reads bounded contexts and source files, **so that** I get one domain-specialized agent per module pre-loaded with its types, errors, patterns, and conventions.

#### Acceptance Criteria

- AC-001.1: Given `docs/domain/bounded-contexts.md` exists, when the command runs, then it reads all module entries (from both the Module Map table and freestanding `###` sections) and presents selection via AskUserQuestion (all or subset)
- AC-001.2: Given a selected module has no source directory at `crates/ecc-domain/src/<module>/` AND no single file at `crates/ecc-domain/src/<module>.rs`, when generation runs, then it warns "Source not found for <module> — skipping" and continues
- AC-001.3: Given a module source exists, when generation runs, then it extracts: (a) all lines matching `pub struct` or `pub enum` with their names, (b) all `#[error("...")]` attribute strings, (c) count of `#[test]` or `#[cfg(test)]` blocks, (d) module description from bounded-contexts.md
- AC-001.4: Given extracted data, when agent is written, then it goes to `agents/domain/<module>.md` with valid frontmatter (name, description, model: sonnet, effort: medium, tools: [Read, Grep, Glob], generated: true, generated_at: ISO 8601) and passes `ecc validate agents`
- AC-001.5: Given an agent file already exists at `agents/domain/<module>.md`, when the command encounters it, then it asks "Agent for <module> already exists. Overwrite?" before replacing
- AC-001.6: Given generation completes for all targets, when the command finishes, then it commits with `feat: generate domain agents for <module-list>`
- AC-001.7: Given `docs/domain/bounded-contexts.md` does not exist, when the command runs, then it exits with "bounded-contexts.md not found — run /project-foundation or create it manually"
- AC-001.8: Given bounded-contexts.md is empty or contains zero parseable module entries, when the command runs, then it exits with "No modules found in bounded-contexts.md"
- AC-001.9: Given a bounded context maps to a single file (not a directory), when generation runs, then that single file is read for type/error extraction

#### Dependencies

- Depends on: none

### US-002: Domain Agent Body Content Standard

**As a** developer, **I want** each generated agent body to contain structured domain knowledge sections, **so that** invoked agents give context-aware answers grounded in actual source code.

#### Acceptance Criteria

- AC-002.1: Agent body contains `## Domain Model` section listing each `pub struct`/`pub enum` name with its doc-comment (first line) if present, or "no doc-comment" if absent
- AC-002.2: Agent body contains `## Error Catalogue` section listing each `#[error("...")]` variant name and its error message string. If no thiserror errors exist, section says "No thiserror error types in this module"
- AC-002.3: Agent body contains `## Test Patterns` section with: (a) total `#[test]` count, (b) list of test module names (`mod tests`, `mod integration_tests`), (c) whether `assert_eq!` or `pretty_assertions` is used (grep-based detection)
- AC-002.4: Agent body contains `## Cross-Module Dependencies` listing deps from bounded-contexts.md Cross-Module Dependencies section. If none listed, section says "Independent — no cross-module dependencies"
- AC-002.5: Agent body contains `## Naming Conventions` with: (a) most common function prefix patterns (e.g., `resolve_`, `parse_`, `validate_`) counted by grep, (b) whether snake_case or other casing is used for module-level constants
- AC-002.6: Agent body contains zero occurrences of `TODO`, `<describe>`, `<fill>`, `[step`, or `[your` (verified by grep)

#### Dependencies

- Depends on: US-001

### US-003: Validate Recurses Into agents/domain/

**As a** developer, **I want** `ecc validate agents` to include `agents/domain/*.md`, **so that** generated agents are held to the same standards.

#### Acceptance Criteria

- AC-003.1: Given `agents/domain/backlog.md` with valid frontmatter, when `ecc validate agents` runs, then it reports the file as validated
- AC-003.2: Given `agents/domain/backlog.md` with missing `model` field, when `ecc validate agents` runs, then it fails with error mentioning the file path
- AC-003.3: Given `agents/domain/` does not exist, when `ecc validate agents` runs, then it succeeds silently (no error for absent optional directory)
- AC-003.4: Given `generated: true` and `generated_at: "2026-04-17T00:00:00Z"` in agent frontmatter, when validation runs, then both fields are accepted as valid optional fields

#### Dependencies

- Depends on: none

### US-004: Pipeline Domain Context Injection

**As a** developer, **I want** pipeline commands to auto-discover and inject relevant domain agent context at Phase 0.7, **so that** all pipeline stages have deep domain knowledge without manual agent invocation.

#### Acceptance Criteria

- AC-004.1: `/spec-dev`, `/spec-fix`, `/spec-refactor` have a "Phase 0.7: Domain Context" (inserted after Phase 0.5 Sources Consultation, before Phase 1) that matches modules by: (a) tokenizing the feature description into words, (b) exact-matching each word against bounded-contexts.md module names (case-insensitive). If zero matches, Phase 0.7 is skipped with log "No domain agents matched"
- AC-004.2: `/design` has a Phase 0.7 that matches affected modules by reading the spec's `## Affected Modules` table, extracting the Module column values, and comparing against `agents/domain/<module>.md` filenames
- AC-004.3: `/implement` has a Phase 0.7 that extracts module names from the design's `## File Changes` table by parsing crate paths (e.g., `crates/ecc-domain/src/workflow/` → `workflow`) and matching against `agents/domain/<module>.md`
- AC-004.4: Phase 0.7 spawns each matched domain agent as a read-only Task subagent (allowedTools: [Read, Grep, Glob]) with prompt "Summarize your domain knowledge relevant to: <feature>". The output is stored verbatim in a `## Domain Context` section prepended to the plan file.
- AC-004.5: If `agents/domain/` does not exist or contains zero `.md` files, Phase 0.7 is silently skipped
- AC-004.6: If more than 3 modules match, Phase 0.7 selects the first 3 in alphabetical order (deterministic, not heuristic)
- AC-004.7: Phase 0.7 is labeled "Phase 0.7" in all 5 modified command files (not "Phase 0.5")

#### Dependencies

- Depends on: US-001, US-002, US-003

### US-005: Staleness Detection

**As a** developer, **I want** the command to detect when generated agents are stale relative to their source code, **so that** I know to re-run after module evolution.

#### Acceptance Criteria

- AC-005.1: Given an agent with `generated_at` timestamp, when `git log --since=<generated_at> -- crates/ecc-domain/src/<module>/` returns commits, then warn "Agent for <module> may be stale — N commits since generation"
- AC-005.2: Given `--check-staleness` flag without `--overwrite`, then only check and exit 1 if any stale, exit 0 if all fresh (CI-friendly)
- AC-005.3: Given git is unavailable or not a git repo, then skip staleness check with "git not available — staleness check skipped"

#### Dependencies

- Depends on: US-001

### US-006: Domain Agents Discovery Skill

**As a** pipeline command author, **I want** a `domain-agents` skill documenting discovery and usage patterns, **so that** commands can reference it for consistent injection behavior.

#### Acceptance Criteria

- AC-006.1: Skill at `skills/domain-agents/SKILL.md` with valid frontmatter (name: domain-agents, description, origin: ECC)
- AC-006.2: Documents `agents/domain/` structure, naming convention (`<module-name>.md`), and lookup by bounded context name
- AC-006.3: Under 500 words (verified by `wc -w`)
- AC-006.4: Includes "Graceful degradation: if `agents/domain/` does not exist, skip silently" note

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `commands/generate-domain-agents.md` | Command | Create: new command |
| `agents/domain/*.md` | Agents | Create: generated output |
| `skills/domain-agents/SKILL.md` | Skill | Create: discovery docs |
| `crates/ecc-app/src/validate/agents.rs` | App | Modify: recurse into agents/domain/ |
| `crates/ecc-domain/src/config/agent_frontmatter.rs` | Domain | Modify: add generated + generated_at fields |
| `commands/spec-dev.md` | Command | Modify: add Phase 0.7 domain context injection |
| `commands/spec-fix.md` | Command | Modify: add Phase 0.7 |
| `commands/spec-refactor.md` | Command | Modify: add Phase 0.7 |
| `commands/design.md` | Command | Modify: add Phase 0.7 |
| `commands/implement.md` | Command | Modify: add Phase 0.7 |

## Constraints

- Generated agents must pass `ecc validate agents`
- Domain agents are read-only (tools: [Read, Grep, Glob])
- Requires `docs/domain/bounded-contexts.md`
- Maximum 3 domain agents per pipeline phase (alphabetical selection if >3 match)
- No placeholder text in generated bodies
- `ecc-domain` remains pure — extraction is LLM-driven, not Rust static analysis
- Module name matching uses exact word match (not substring) to prevent false positives

## Non-Requirements

- CLI-assisted static analysis (future backlog item)
- Auto-regeneration via hooks
- Cross-language support
- GUI/TUI for agent selection
- Auto-merging domain output into specs
- Substring/fuzzy matching for module discovery (exact match only)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| agents/ directory | New subdirectory | ecc validate agents integration test |
| Pipeline commands | New Phase 0.7 | Manual testing of domain injection |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| ADR | Docs | docs/adr/0066-domain-agent-generation.md | Generation architecture |
| ADR | Docs | docs/adr/0067-pipeline-domain-injection.md | Phase 0.7 pattern |
| CLI | Project | CLAUDE.md | Add /generate-domain-agents + glossary terms |
| Changelog | Docs | CHANGELOG.md | Add entry |

## Open Questions

None — all resolved during grill-me interview.
