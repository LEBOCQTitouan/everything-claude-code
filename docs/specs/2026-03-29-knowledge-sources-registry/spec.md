# Spec: Knowledge Sources Registry

## Problem Statement

ECC commands like /spec, /audit, /implement, and /design currently do ad-hoc web research with no persistent record of authoritative sources. This creates duplicated research effort across sessions and no shared vocabulary of trusted references. The registry solves this by making sources a first-class project artifact committed to docs/, with Rust CLI support for management and deterministic bi-directional command integrations.

## Research Summary

- **ThoughtWorks Technology Radar** is the canonical model. Four rings (Adopt, Trial, Assess, Hold) express confidence; quadrants categorize source types.
- **"Everything-as-code" Tech Radars** store entries as Markdown with YAML frontmatter metadata, enabling git-tracked curation and diffable changes.
- **`bkmr` (Rust crate)** is closest prior art for CLI-based curated registry. Uses SQLite; ECC should use flat Markdown for transparency.
- **Minimum viable schema is critical** — over-engineering metadata kills adoption. Start with: name, url, type, quadrant, subject, added-by, dates.
- **Integration with existing workflows** matters more than standalone features. Programmatic CLI queries serve agentic consumers.
- **Living artifacts with lifecycle management** — entries move between rings over time, with history preserved.
- **Single source of truth with ownership governance** — each entry tracks added-date, last-checked, and reviewer.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | All 3 phases in one spec | User wants complete feature, not incremental | No |
| 2 | Shell-out to curl for reachability | No new port trait needed. Simple, portable. | No |
| 3 | Parser in domain layer | Pure fn(&str) -> Result. Matches backlog pattern exactly. | No |
| 4 | Inbox processing part of reindex | Single command handles both reindex and inbox | No |
| 5 | Explicit module mapping table | Most precise subject-to-module mapping | No |
| 6 | HTTP non-200 = stale; 90d warning; 180d warning | Two-tier warning system with hard stale flag | No |
| 7 | Full bi-directional command integrations | Commands read AND update metadata deterministically | No |
| 8 | ADR-0031: Sources bounded context | New bounded context with Technology Radar vocabulary | Yes |
| 9 | `added_by` is free-text, typically "human" or "agent:<command-name>" | Simple, no validation overhead | No |
| 10 | `check` exits 0 always (advisory mode) | Not a CI gate; advisory for human review | No |
| 11 | curl failures (timeout, DNS, SSL) all treated as "unreachable" | Simplifies error handling; specific error logged to stderr | No |
| 12 | Module mapping is advisory; invalid module paths not validated at parse time | Avoids coupling sources to codebase structure | No |

## User Stories

### US-001: Sources Domain Model

**As a** developer, **I want** a pure domain model for knowledge sources (entry, quadrant, source type, lifecycle state, module mapping), **so that** all business rules are enforced without I/O.

#### Acceptance Criteria

- AC-001.1: Given a source entry, when created, then it contains: url, title, type (repo/doc/blog/package/talk/paper), quadrant (Adopt/Trial/Assess/Hold), subject, added_by, added_date, last_checked, optional deprecation_reason
- AC-001.2: Given a source with deprecation_reason, when queried, then its lifecycle state is Deprecated (entry remains, not removed)
- AC-001.3: Given identical inputs, when any lifecycle operation runs twice, then the result is identical (deterministic)
- AC-001.4: Given a source URL, when validated, then it must be a valid URL format (domain validates structure, not reachability)
- AC-001.5: Given a source name, when validated, then it must be non-empty

#### Dependencies

- Depends on: none

### US-002: Sources Markdown Parser and Serializer

**As a** developer, **I want** to parse `docs/sources.md` into the domain model and serialize back to Markdown, **so that** the file is the single source of truth with lossless round-trips.

#### Acceptance Criteria

- AC-002.1: Given a well-formed sources.md, when parsed, then all entries in Inbox, four quadrant sections, and module mapping table are loaded
- AC-002.2: Given a parsed registry, when serialized, then output matches canonical format (Inbox, then quadrants with subject subsections, then module mapping)
- AC-002.3: Given a parse-serialize round-trip, when compared, then output is semantically identical
- AC-002.4: Given malformed entries, when parsed, then errors are reported per-entry without aborting the entire parse
- AC-002.5: Given an empty file or missing sections, when parsed, then defaults to empty collections (no crash)

#### Dependencies

- Depends on: US-001

### US-003: Sources App Use Cases

**As a** developer, **I want** app-layer use cases for listing, adding, checking, and reindexing sources, **so that** CLI and commands orchestrate through port traits.

#### Acceptance Criteria

- AC-003.1: Given `list` with optional quadrant/subject filters, when called, then matching entries are returned
- AC-003.2: Given valid parameters, when `add` is called, then entry is appended to correct quadrant/subject (atomic write)
- AC-003.3: Given a non-existent sources.md, when `add` is called, then the file is created with the entry
- AC-003.4: Given inbox entries, when `reindex` is called, then inbox entries are moved to correct quadrant/subject sections
- AC-003.5: Given `reindex` with --dry-run, when called, then changes are displayed but not written
- AC-003.6: Given `check` is called, when curl returns non-200, then the source is flagged as stale in the file
- AC-003.7: Given `check` is called, when last_checked > 90 days, then a WARN-level message is displayed (e.g., "WARN: <title> last checked 95 days ago")
- AC-003.8: Given `check` is called, when last_checked > 180 days, then an ERROR-level message is displayed (e.g., "ERROR: <title> last checked 200 days ago — consider reviewing")
- AC-003.9: Given a duplicate URL, when `add` is called, then the command warns and rejects (no duplicate entries)
- AC-003.10: Given `check`, when curl takes > 10 seconds for a URL, then it is treated as unreachable and the next URL is processed
- AC-003.11: Given a source flagged stale, when `check` runs again and curl returns 200, then the stale flag is cleared

#### Dependencies

- Depends on: US-001, US-002

### US-004: Sources CLI Subcommands

**As a** user, **I want** `ecc sources list/add/check/reindex` CLI commands, **so that** I can manage the registry from the terminal.

#### Acceptance Criteria

- AC-004.1: Given `ecc sources list [--quadrant <q>] [--subject <s>]`, when run, then matching entries are printed in tabular format
- AC-004.2: Given `ecc sources add <url> --title <t> --type <type> --quadrant <q> --subject <s>`, when run, then entry is added to docs/sources.md
- AC-004.3: Given `ecc sources check`, when run, then all URLs are checked via curl and stale/warning results are displayed
- AC-004.4: Given `ecc sources reindex [--dry-run]`, when run, then docs/sources.md is regenerated with inbox entries classified

#### Dependencies

- Depends on: US-003

### US-005: Sources File Bootstrap and CLAUDE.md

**As a** project maintainer, **I want** an initial docs/sources.md with the canonical schema, module mapping table, and a CLAUDE.md pointer, **so that** the registry is ready for use.

#### Acceptance Criteria

- AC-005.1: Given bootstrap, when docs/sources.md is created, then it has: Inbox section, four quadrant sections, module mapping table
- AC-005.2: Given bootstrap, when CLAUDE.md is updated, then it has a single-line pointer in the doc hierarchy section
- AC-005.3: Given the initial file, when seeded with ECC's actual sources, then at least one entry per quadrant exists

#### Dependencies

- Depends on: US-002

### US-006: Command Integrations (bi-directional)

**As a** developer, **I want** /spec, /implement, /design, /audit, /review, /catchup to both consult and update sources, **so that** the registry stays current as a living artifact.

#### Acceptance Criteria

- AC-006.1: Given /spec Phase 0, when the spec's subject matches a source entry's `subject` field (case-insensitive exact match) OR the affected module appears in the module mapping table, then a "Consulted sources" section is produced AND last_checked is updated
- AC-006.2: Given /implement, when modifying a module with mapped sources, then relevant sources are surfaced AND last_checked is updated
- AC-006.3: Given /design, when architectural sources exist, then they are referenced AND last_checked is updated
- AC-006.4: Given /audit (audit-evolution, audit-web), when run, then sources are re-interrogated and stale flags updated
- AC-006.5: Given /review, when sources exist for the reviewed module (via module mapping), then those sources are listed as reference context for the reviewer
- AC-006.6: Given /catchup, when run, then entries in docs/sources.md modified since the last git commit that modified docs/sources.md on the current branch are summarized
- AC-006.7: All integrations are deterministic — same input produces same output
- AC-006.8: Given docs/sources.md does not exist, then commands work normally without sources (graceful degradation)

#### Dependencies

- Depends on: US-004, US-005

### US-007: ADR for Sources Bounded Context

**As a** project maintainer, **I want** an ADR documenting the sources bounded context and Technology Radar vocabulary, **so that** the architectural decision is recorded.

#### Acceptance Criteria

- AC-007.1: Given the ADR, when it describes the bounded context, then it explains sources as an independent domain with Technology Radar vocabulary
- AC-007.2: Given the ADR, when it describes the decision, then it references BL-086, the backlog pattern reuse, and the explicit module mapping approach

#### Dependencies

- None

## Affected Modules

| Module | Layer | Change Type |
|--------|-------|-------------|
| `crates/ecc-domain/src/sources/` | Domain | New module (entry, parser, serializer, check model) |
| `crates/ecc-app/src/sources.rs` | App | New use cases (list, add, check, reindex) |
| `crates/ecc-cli/src/commands/sources.rs` | CLI | New subcommand tree |
| `commands/spec-dev.md` | Command (markdown) | Add sources consultation + update step |
| `commands/spec-fix.md` | Command (markdown) | Add sources consultation + update step |
| `commands/spec-refactor.md` | Command (markdown) | Add sources consultation + update step |
| `commands/implement.md` | Command (markdown) | Add sources consultation + update step |
| `commands/design.md` | Command (markdown) | Add sources consultation + update step |
| `commands/review.md` | Command (markdown) | Add sources consultation + update step |
| `commands/catchup.md` | Command (markdown) | Add sources summary step |
| `commands/audit-web.md` | Command (markdown) | Add sources re-interrogation step |
| `docs/sources.md` | Documentation | New file |
| `docs/adr/0031-sources-bounded-context.md` | Documentation | New ADR |
| `CLAUDE.md` | Documentation | Add pointer + CLI commands |
| `docs/domain/bounded-contexts.md` | Documentation | Add sources bounded context |

## Constraints

- Domain crate must have zero I/O imports (enforced by hook)
- Parser in domain as pure function; file I/O in app through FileSystem port
- Replicate backlog module pattern exactly (same crate involvement, same port reuse)
- Atomic writes for reindex (mktemp + mv)
- Shell-out to curl for reachability (no new port trait)
- docs/sources.md stays under 800 lines; split only if exceeded

## Non-Requirements

- No automated source scraping or content ingestion
- No automatic context preloading on every command
- No web UI or external service
- No SQLite or database — flat Markdown file only
- No HTTP client port trait — curl shell-out is sufficient
- No file splitting for docs/sources.md in v1 — 800-line constraint is a guideline, not enforced
- No custom source types or quadrant names beyond the initial set in v1

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| FileSystem | Read/write new file format | Existing FileSystem E2E coverage applies |
| CLI parsing | New `ecc sources` subcommand tree | Verify Clap routing |
| Shell (curl) | New shell-out for reachability check | Verify curl invocation |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New bounded context | docs/domain/ | bounded-contexts.md | Add sources entry |
| New CLI commands | CLAUDE.md | CLI commands section | Add ecc sources commands |
| New ADR | docs/adr/ | ADR-0031 | Create |
| New file | docs/ | sources.md | Create with schema |
| Doc hierarchy | CLAUDE.md | Doc hierarchy section | Add pointer to sources.md |
| Changelog | CHANGELOG.md | — | Add entry |

## Open Questions

None — all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Scope? | All 3 phases (file format, CLI, command integrations, check) | User |
| 2 | Network port for check? | Shell-out to curl via existing executor | Recommended |
| 3 | Parser location? | Domain layer (pure fn, matches backlog pattern) | Recommended |
| 4 | Inbox processing trigger? | Part of ecc sources reindex | Recommended |
| 5 | Subject-to-module mapping? | Explicit mapping table in sources.md | User |
| 6 | Stale detection? | HTTP non-200 = stale; 90d WARN; 180d ERROR | User |
| 7 | Command integration depth? | Full bi-directional + deterministic | User |
| 8 | ADR? | One ADR (0031): Sources bounded context | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Sources Domain Model | 5 | None |
| US-002 | Sources Markdown Parser and Serializer | 5 | US-001 |
| US-003 | Sources App Use Cases | 11 | US-001, US-002 |
| US-004 | Sources CLI Subcommands | 4 | US-003 |
| US-005 | Sources File Bootstrap and CLAUDE.md | 3 | US-002 |
| US-006 | Command Integrations (bi-directional) | 8 | US-004, US-005 |
| US-007 | ADR for Sources Bounded Context | 2 | None |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | Source entry fields (url, title, type, quadrant, subject, dates) | US-001 |
| AC-001.2 | Deprecated lifecycle state | US-001 |
| AC-001.3 | Deterministic operations | US-001 |
| AC-001.4 | URL format validation | US-001 |
| AC-001.5 | Non-empty name validation | US-001 |
| AC-002.1 | Parse all sections (Inbox, quadrants, mapping) | US-002 |
| AC-002.2 | Serialize to canonical format | US-002 |
| AC-002.3 | Lossless round-trip | US-002 |
| AC-002.4 | Per-entry error reporting | US-002 |
| AC-002.5 | Empty/missing section defaults | US-002 |
| AC-003.1 | List with filters | US-003 |
| AC-003.2 | Add with atomic write | US-003 |
| AC-003.3 | Create file on first add | US-003 |
| AC-003.4 | Reindex moves inbox entries | US-003 |
| AC-003.5 | Reindex dry-run | US-003 |
| AC-003.6 | Check flags stale on non-200 | US-003 |
| AC-003.7 | Check WARN at 90 days | US-003 |
| AC-003.8 | Check ERROR at 180 days | US-003 |
| AC-003.9 | Duplicate URL rejection | US-003 |
| AC-003.10 | Curl timeout > 10s = unreachable | US-003 |
| AC-003.11 | Stale flag cleared on successful recheck | US-003 |
| AC-004.1 | CLI list with filters | US-004 |
| AC-004.2 | CLI add | US-004 |
| AC-004.3 | CLI check | US-004 |
| AC-004.4 | CLI reindex | US-004 |
| AC-005.1 | Bootstrap file with all sections | US-005 |
| AC-005.2 | CLAUDE.md pointer | US-005 |
| AC-005.3 | Seed entries per quadrant | US-005 |
| AC-006.1 | /spec sources consultation + update | US-006 |
| AC-006.2 | /implement sources consultation + update | US-006 |
| AC-006.3 | /design sources consultation + update | US-006 |
| AC-006.4 | /audit sources re-interrogation | US-006 |
| AC-006.5 | /review sources as reference context | US-006 |
| AC-006.6 | /catchup sources diff summary | US-006 |
| AC-006.7 | Deterministic integrations | US-006 |
| AC-006.8 | Graceful degradation without sources.md | US-006 |
| AC-007.1 | ADR bounded context explanation | US-007 |
| AC-007.2 | ADR decision references | US-007 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Decision Completeness | 85 | PASS | Decisions 9-12 close all gaps |
| Ambiguity | 75 | PASS | Subject matching, warning levels, catchup diff defined |
| Testability | 75 | PASS | US-006 ACs now have concrete matching rules |
| Edge Cases | 72 | PASS | Curl timeout, stale recovery, file splitting deferred |
| Rollback & Failure | 70 | PASS | Stale flag recovery, atomic writes |
| Scope Creep Risk | 65 | PASS | Non-requirements bound types/quadrants/splitting |
| Dependency Gaps | 78 | PASS | DAG clean, implicit ordering clarified |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-29-knowledge-sources-registry/spec.md | Full spec + phase summary |
