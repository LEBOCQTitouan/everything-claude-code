# Spec: Universal App Cartography System — Sub-Spec A (Journeys + Flows + Trigger)

## Problem Statement

The codebase accumulates knowledge across sessions but lacks a unified, automatically-maintained map of user journeys, data flows, and system elements. Developers must read source code to understand behavior. New contributors have no structured way to navigate the system by "what it does" rather than "where files are." The existing BL-056 Phase 7.5 only generates module summaries and diagrams for Rust projects during `/implement`, leaving all other commands and languages undocumented.

## Research Summary

- **Diagram-as-Code**: Use Mermaid stored in Git alongside source for diff-tracking and CI integration. C4 model provides multi-level architecture visualization.
- **Incremental Updates**: Apply delta merge semantics — track changes at component level rather than regenerating entire docs. Section markers (`<!-- CARTOGRAPHY: ... -->`) enable deterministic merge boundaries.
- **AI-Powered Analysis**: LLMs analyze repository structure, dependencies, and component relationships. Let AI generate summaries and Mermaid diagrams from code analysis.
- **Multi-Level Documentation**: C4 model (Context, Container, Component, Code) provides views at different detail levels. High-level overview + deep detail for every artifact.
- **Automation Pattern**: Scripts that generate diagrams from code ensure docs always reflect reality. Treat diagrams like version-controlled code.
- **Pitfall**: Avoid manual static documentation that diverges from code. Enforce deterministic regeneration.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Two-phase cartography: Stop hook writes delta, next session agent processes | Stop hooks have 5-10s timeout, cannot invoke AI agents. Manifest-queue is reliable and non-blocking | Yes |
| 2 | Session-scoped delta files (pending-delta-<session_id>.json) | Eliminates concurrent write races at hook level entirely | Yes |
| 3 | No CartographyStore port — domain types are pure, app layer uses FileSystem + FileLock for persistence | Output is Markdown; domain defines the merge algorithm and types (zero I/O), app layer handles reading/writing via existing port traits | Yes |
| 4 | cartography as independent bounded context in ecc-domain | No imports from workflow, session, or hook_runtime domains. Clean separation | Yes |
| 5 | New language-agnostic agents, not extensions of BL-056 | module-summary-updater and diagram-updater are Rust-specific. Cartography must work on any project | No |
| 6 | Element registry deferred to Sub-Spec B | Depends on stable journey + flow formats. Reduces rework risk | No |
| 7 | Section markers + git diff for delta merge | Markers for generated sections, git diff for detecting manual edits. Complementary strategies | No |
| 8 | Session-context flows for incremental updates; full scan deferred to backlog | "Every data movement" not achievable incrementally. One-time scan tracked as separate backlog item | No |
| 9 | Include staleness detection + coverage dashboard in v1 | User requested these two enhancements. Other 3 enhancements deferred to backlog | No |
| 10 | Cartography agent invoked as blocking SessionStart hook handler (Rust, not AI agent) | The `session:start` hook already exists and runs synchronously. The cartography start handler reads pending deltas and invokes the cartography agent inline. If no deltas exist, it exits in <10ms. Heavy generation runs as a background Task subagent dispatched by the handler | Yes |
| 11 | Slug derivation: lowercase filename, replace non-alphanumeric with hyphens, collapse multiples, max 60 chars, derived from the primary changed file or command name | Consistent naming across all registries. Same pattern as spec slug generation | No |
| 12 | Processed delta retention: keep for 30 days in `.claude/cartography/processed/`, then auto-prune | Enables debugging and re-processing if needed, avoids unbounded growth | No |
| 13 | Slug input: first changed file's parent directory name (for commands: command name; for crates: crate name; fallback: first directory in delta) | Deterministic, reproducible across sessions, supports idempotent re-processing | No |

## User Stories

### US-001: Cartography Directory Scaffold and Schema Definition

**As a** developer using ECC on any project, **I want** a well-defined cartography directory structure with versioned schemas, **so that** generators have a stable target format and files can be validated.

#### Acceptance Criteria

- AC-001.1: Given ECC runs on a project, when `docs/cartography/` does not exist, then `docs/cartography/journeys/`, `docs/cartography/flows/` directories are created with README placeholder
- AC-001.2: Given a journey file exists, when `ecc validate cartography` runs, then it validates required sections: Overview, Mermaid Diagram, Steps, Related Flows
- AC-001.3: Given a flow file exists, when `ecc validate cartography` runs, then it validates required sections: Overview, Mermaid Diagram, Source-Destination, Transformation Steps, Error Paths
- AC-001.4: Given scaffold runs on a project with existing `docs/cartography/`, then existing content is not overwritten

#### Dependencies
- Depends on: none

### US-002: Delta Merge Engine

**As a** cartography generator agent, **I want** a reliable delta merge mechanism using section markers and git diff, **so that** sessions accumulate knowledge without clobbering each other.

#### Acceptance Criteria

- AC-002.1: Given a journey file has a `<!-- CARTOGRAPHY: step-N -->` marker, when the generator adds a new step, then the step is appended inside the section without replacing existing steps
- AC-002.2: Given a step with the same step ID exists, when the generator produces an updated version, then the existing step is replaced in-place preserving surrounding content
- AC-002.3: Given a partial cartography update is in progress, when a session is interrupted, then uncommitted changes are detected via `git status` and discarded on next run
- AC-002.4: Given concurrent sessions attempt to write, when file locking detects a conflict, then the second writer queues its update (delta file persists for next run)
- AC-002.5: Given manually-authored content exists outside markers, when the merge engine runs, then manual content is preserved untouched

#### Dependencies
- Depends on: none

### US-003: Project-Agnostic Change Detection via Stop Hook

**As a** cartography system, **I want** to detect changed files at session end without assuming a specific language, **so that** the correct cartography files can be targeted on any project.

#### Acceptance Criteria

- AC-003.1: Given a session ends, when the `stop:cartography` hook runs, then it writes a `pending-delta-<session_id>.json` to `.claude/cartography/` containing: session_id, timestamp, changed file paths, project_type (enum: `rust`, `javascript`, `typescript`, `python`, `go`, `java`, `unknown`)
- AC-003.2: Given a Rust workspace (Cargo.toml at root), when change detection runs, then project_type is `rust` and files are classified by crate name
- AC-003.3: Given a JS/TS project (package.json at root), when change detection runs, then project_type is `javascript` or `typescript` and files are classified by package path
- AC-003.4: Given no recognized build file, when change detection runs, then project_type is `unknown` and files are classified by top-level directory
- AC-003.5: Given zero committed changes, when the hook runs, then it exits with passthrough (no delta file written)
- AC-003.6: Given the hook runs, when it writes the delta file, then it completes within 10 seconds (async timeout)
- AC-003.7: Given `CLAUDE_SESSION_ID` is not set, when the hook runs, then it generates a fallback session ID from timestamp + PID (same pattern as ecc-cli main.rs)
- AC-003.8: Given the project has no git repository, when the hook runs, then it exits with passthrough and logs a warning (cartography requires git)
- AC-003.9: Given a corrupt or truncated delta file exists from a prior crash, when the next hook run detects invalid JSON, then it deletes the corrupt file and logs a warning (the current session's delta is written independently)

#### Dependencies
- Depends on: none

### US-004: User Journey Registry Generator

**As a** developer reading project documentation, **I want** journey files auto-generated in `docs/cartography/journeys/`, **so that** I can understand who interacts with the system and what outcomes they produce.

#### Acceptance Criteria

- AC-004.1: Given a delta references changed command/handler files, when the cartography agent runs, then a journey file is created at `docs/cartography/journeys/<slug>.md` that passes schema validation (AC-001.2)
- AC-004.2: Given a session modifies an existing command, when the agent runs, then the journey file is delta-merged: new steps appended inside markers, changed steps updated, existing steps preserved
- AC-004.3: Given a journey file is created, then it contains a `## Mermaid Diagram` section with syntactically valid Mermaid (parseable by `mmdc`) and a `## Steps` section with detailed breakdown
- AC-004.4: Given a journey file references related flows, then each flow is linked by relative path to `docs/cartography/flows/<slug>.md`
- AC-004.5: Given no existing journey files, when the generator runs first time, then it creates files only for artifacts changed in the current delta — no full backfill
- AC-004.6: Given the journey generator cannot determine the actor or trigger from the changed files, then it creates the journey file with a `<!-- GAP: actor unknown, infer from context -->` marker in the actor field

#### Dependencies
- Depends on: US-002 (delta merge), US-003 (change detection)

### US-005: Data Flow Registry Generator

**As a** developer tracing data movement, **I want** flow files auto-generated in `docs/cartography/flows/`, **so that** I can audit data paths without instrumenting the running system.

#### Acceptance Criteria

- AC-005.1: Given a delta references files that cross module boundaries (for `rust`: different crates; for `javascript`/`typescript`: different packages; for `unknown`: different top-level directories), when the flow generator runs, then a flow file is created at `docs/cartography/flows/<slug>.md` that passes schema validation (AC-001.3)
- AC-005.2: Given external I/O is added (file, HTTP, database), when the flow generator runs, then the flow file captures: external system, direction, transformation
- AC-005.3: Given a flow file is created, then it contains a `## Mermaid Diagram` section with syntactically valid Mermaid (parseable by `mmdc`) and a `## Transformation Steps` section with detailed breakdown
- AC-005.4: Given a previously documented flow, when a session modifies it, then only changed steps are updated via delta merge inside markers
- AC-005.5: Given a flow the generator cannot fully infer, then it inserts a `<!-- GAP: <description of what is unknown> -->` marker at the uncertain point rather than silently omitting

#### Dependencies
- Depends on: US-002 (delta merge), US-003 (change detection)

### US-006: Cartography Agent — SessionStart Processing

**As a** cartography system, **I want** pending deltas processed at the start of each session, **so that** documentation is updated before the developer begins new work.

#### Acceptance Criteria

- AC-006.1: Given pending delta files exist in `.claude/cartography/`, when a new session starts and the `session:start` hook runs, then the cartography agent reads all pending deltas in chronological order (sorted by timestamp field)
- AC-006.2: Given the cartography agent commits updates, then it stages only `docs/cartography/` files (`git add docs/cartography/`) and creates a single atomic commit: `docs(cartography): update registries for <session-slug>` — never staging unrelated developer work
- AC-006.3: Given a successful commit, when the agent archives deltas, then processed deltas are moved to `.claude/cartography/processed/` (commit BEFORE archive to prevent data loss)
- AC-006.4: Given cartography generation fails, when the session continues, then the failure is logged to stderr but does not block session start
- AC-006.5: Given no pending deltas exist, when session starts, then the cartography agent exits immediately (< 10ms)
- AC-006.6: Given two sessions start simultaneously with the same pending deltas, when both try to process, then the first to acquire the `cartography-merge` file lock processes; the second skips and the deltas remain pending for the next session
- AC-006.7: Given a delta file has already been processed (exists in `processed/`), when the agent encounters it again (re-entry), then it skips it (idempotent processing)
- AC-006.8: Given `git commit` fails (non-zero exit), when the agent handles the error, then it runs `git reset HEAD docs/cartography/` to unstage, logs the error, and leaves the pending deltas unarchived for the next session to retry

#### Dependencies
- Depends on: US-004 (journey generator), US-005 (flow generator)

### US-007: Staleness Detection

**As a** developer maintaining documentation, **I want** cartography entries flagged when their source code changed but the entry wasn't updated, **so that** I can identify outdated documentation.

#### Acceptance Criteria

- AC-007.1: Given a cartography entry has a `<!-- CARTOGRAPHY-META: last_updated=YYYY-MM-DD, sources=path1,path2 -->` marker, when any listed source file has a git commit date newer than `last_updated`, then the entry is marked `<!-- STALE: last_updated=YYYY-MM-DD, source_modified=YYYY-MM-DD -->`
- AC-007.2: Given `ecc validate cartography` runs, when stale entries exist, then it reports them with the staleness delta in days
- AC-007.3: Given the cartography agent processes a delta that touches a stale entry's source, when it updates the entry, then the stale marker is removed

#### Dependencies
- Depends on: US-004 (journeys exist to check), US-005 (flows exist to check)

### US-008: Coverage Dashboard

**As a** project maintainer, **I want** a summary showing what percentage of the project is covered by cartography, **so that** I can identify documentation gaps.

#### Acceptance Criteria

- AC-008.1: Given `ecc validate cartography --coverage` runs, then it outputs: total source files, files referenced in at least one journey or flow, coverage percentage
- AC-008.2: Given coverage is below 50%, then the output includes a "Priority gaps" section listing the top 10 undocumented files by change frequency
- AC-008.3: Given the coverage report runs, then it completes in under 5 seconds for projects with up to 500 source files

#### Dependencies
- Depends on: US-004 (journeys exist), US-005 (flows exist)

### US-009: /spec Integration — Actor Registry

**As a** developer running `/spec`, **I want** `/spec` to read actor categories from existing journey files, **so that** new user stories reference established actors.

#### Acceptance Criteria

- AC-009.1: Given journey files exist, when `/spec` runs its actor-identification step, then it presents existing actors as suggestions
- AC-009.2: Given no journey files exist, then `/spec` proceeds without suggestions
- AC-009.3: Given a user picks an existing actor, then the actor name exactly matches the journey file
- AC-009.4: Given a new actor is introduced, then `/spec` adds a note to create a journey entry

#### Dependencies
- Depends on: US-001 (schema), US-004 (journey files exist)

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `ecc-domain/src/cartography/` | Domain | New bounded context: SessionDelta, CartographyManifest, merge algorithm |
| `ecc-domain/src/lib.rs` | Domain | Add `pub mod cartography;` |
| `ecc-app/src/hook/handlers/tier3_session/cartography.rs` | Application | New: stop:cartography (delta writer) + start:cartography (delta processor) handlers |
| `ecc-app/src/hook/mod.rs` | Application | Add dispatch match arms for both stop:cartography and start:cartography |
| `hooks/hooks.json` | Config | Add Stop hook entry for stop:cartography + extend SessionStart with start:cartography |
| `agents/cartographer.md` | Agent | New: orchestrates journey + flow generation from deltas |
| `agents/cartography-journey-generator.md` | Agent | New: generates journey docs |
| `agents/cartography-flow-generator.md` | Agent | New: generates flow docs |
| `commands/spec-dev.md` | Command | Add actor registry reading (US-009) |
| `ecc-cli/src/commands/validate.rs` | CLI | Add `cartography` subcommand |
| `ecc-app/src/validate_cartography.rs` | Application | New: schema validation + staleness + coverage |

## Constraints

- Non-blocking: cartography never delays session completion
- Delta merge only: no overwrites, markers + git diff
- Language-agnostic: must work on any project
- Partial updates discarded on interrupted sessions
- Stop hook must complete within 10 seconds
- Independent bounded context: no cross-domain imports in ecc-domain
- Sub-Spec A scope: journeys + flows + trigger + staleness + coverage. Element registry deferred to Sub-Spec B

## Non-Requirements

- Element registry and cross-reference matrix (Sub-Spec B)
- Interactive HTML navigation
- Diff view in commit messages
- Change impact analysis (pre-session)
- Extending existing BL-056 agents (new agents instead)
- Full-project scan on every session (one-time scan is separate command)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| FileSystem | Used by new hook handler | E2E test: delta file written to correct path |
| ShellExecutor | Used for git diff in hook | E2E test: changed files detected correctly |
| Environment | Used for CLAUDE_SESSION_ID, CLAUDE_PROJECT_DIR | E2E test: env vars read correctly |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New bounded context | ARCHITECTURE.md | Add cartography bounded context | Edit |
| New feature | CLAUDE.md | Add `ecc validate cartography` to CLI commands | Edit |
| Architecture decisions | docs/adr/ | 4 new ADRs | Create |
| New bounded context | docs/domain/bounded-contexts.md | Add cartography context | Edit |
| Feature entry | CHANGELOG.md | Add cartography entry | Edit |

## Open Questions

None — all resolved during grill-me interview.
