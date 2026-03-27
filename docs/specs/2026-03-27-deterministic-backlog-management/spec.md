# Spec: Deterministic Backlog Management CLI

## Problem Statement

The backlog-curator agent currently wastes LLM tokens on three fully mechanical operations: counting IDs, comparing titles for duplicates, and editing a markdown index table. These operations are slow (5-10s LLM round-trip vs <100ms deterministic), occasionally inaccurate (LLM miscounts IDs), and add unnecessary complexity to the agent prompt. Moving them to compiled Rust CLI commands makes backlog management faster, more reliable, and cheaper.

## Research Summary

- **YAML frontmatter parsing**: Add `serde_yaml` (or `serde_yml`) to `ecc-domain` Cargo.toml; extract the YAML block between `---` delimiters and deserialize
- **Fuzzy string matching**: Hand-roll Levenshtein distance (< 30 lines) rather than adding a crate dependency — the algorithm is simple and the corpus is tiny (< 100 entries)
- **Keyword intersection**: Split titles on whitespace/hyphens, lowercase, compare sets — standard set intersection, no external crate needed
- **Markdown table generation**: Use `format!()` with padding — no crate needed for simple pipe-delimited tables
- **Atomic file writes**: Follow existing pattern: write to tempfile, rename to target (already used in statusline caching)
- **CLI subcommand pattern**: Follow `ecc dev` model — `BacklogArgs` with `BacklogAction` enum, wire in `main.rs`

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Hand-roll Levenshtein instead of adding `strsim` crate | Algorithm is < 30 lines, corpus is tiny, avoids new dependency | No |
| 2 | Index is derived data — reindex is authoritative | BACKLOG.md should never be manually maintained; any manual rows not backed by files are dropped | No |
| 3 | next-id uses max+1, never fills gaps | Predictable, monotonic IDs; gaps from deletions are expected and harmless | No |
| 4 | Frontmatter parsing tolerates missing optional fields | Skip entries with malformed YAML with a warning; never crash on bad input | No |
| 5 | No agent changes in this spec | Backlog-curator will be updated separately to call CLI commands | No |
| 6 | Add `serde_yaml` to ecc-domain | Not currently in workspace; needed for frontmatter deserialization | No |
| 7 | Scoring formula: `0.7 * levenshtein + 0.3 * jaccard + tag_boost` | Clear, reproducible, testable composite score | No |
| 8 | `check-duplicates` filters to `open` and `in-progress` only | Both represent active entries that could be duplicated | No |

## User Stories

### US-001: Generate next sequential backlog ID

**As a** backlog-curator agent, **I want** to get the next available BL-NNN ID via `ecc backlog next-id`, **so that** I don't waste tokens counting IDs and never produce duplicates.

#### Acceptance Criteria

- AC-001.1: Given a `docs/backlog/` directory with BL-001.md through BL-075.md, when `ecc backlog next-id` runs, then stdout prints `BL-076`
- AC-001.2: Given an empty `docs/backlog/` directory, when `ecc backlog next-id` runs, then stdout prints `BL-001`
- AC-001.3: Given files with gaps (BL-001, BL-003, BL-010), when `ecc backlog next-id` runs, then stdout prints `BL-011` (max+1, no gap filling)
- AC-001.4: Given files with non-BL markdown files mixed in, when `ecc backlog next-id` runs, then non-BL files are ignored
- AC-001.5: Given a `docs/backlog/` directory that doesn't exist, when `ecc backlog next-id` runs, then it exits with error code 1 and prints an error message

#### Dependencies

- Depends on: none

### US-002: Detect duplicate backlog entries

**As a** backlog-curator agent, **I want** to check a proposed title against existing entries via `ecc backlog check-duplicates`, **so that** I can warn users about potential duplicates before creating new entries.

#### Acceptance Criteria

- AC-002.1: Given existing entry "Replace hooks with Rust binaries" and query "Replace hooks with compiled Rust", when `ecc backlog check-duplicates "Replace hooks with compiled Rust"` runs, then JSON output contains a candidate with score >= 0.6
- AC-002.2: Given `--tags rust,hooks` flag matching existing entry tags, when check-duplicates runs, then the score is boosted by +0.15 per matching tag (capped at +0.3)
- AC-002.3: Given no entries with any keyword overlap, when check-duplicates runs, then JSON output is an empty array `[]`
- AC-002.4: Only entries with `status: open` or `status: in-progress` are checked; `implemented`, `archived`, and `promoted` entries are excluded
- AC-002.5: Output format is JSON: `[{"id": "BL-052", "title": "...", "score": 0.78}]` sorted by score descending
- AC-002.6: Given malformed frontmatter in some files, when check-duplicates runs, then those files are skipped with a stderr warning
- AC-002.7: Scoring formula is `score = 0.7 * normalized_levenshtein_similarity + 0.3 * keyword_jaccard_index + tag_boost` where `normalized_levenshtein_similarity = 1.0 - (edit_distance / max(len_a, len_b))`, `keyword_jaccard_index = |intersection| / |union|` of lowercase keyword sets, and `tag_boost = min(0.3, 0.15 * matching_tag_count)`
- AC-002.8: Given an empty query string, when check-duplicates runs, then it exits with error code 1 and prints a usage error

#### Dependencies

- Depends on: none

### US-003: Regenerate backlog index

**As a** developer or agent, **I want** to regenerate BACKLOG.md from backlog entry files via `ecc backlog reindex`, **so that** the index is always consistent with the actual files.

#### Acceptance Criteria

- AC-003.1: Given 75 BL-*.md files with valid frontmatter, when `ecc backlog reindex` runs, then BACKLOG.md is regenerated with all 75 entries in a sorted table
- AC-003.2: The generated table has columns: `ID | Title | Tier | Scope | Target | Status | Created` (matching current format)
- AC-003.3: Entries are sorted by numeric ID (BL-001 first, BL-075 last)
- AC-003.4: A `## Stats` section is auto-generated with counts: Total, Open, Implemented, Archived (and any other statuses found)
- AC-003.5: The `## Dependency Graph` section from the existing BACKLOG.md is preserved (read from current file, appended after the table)
- AC-003.6: Given `--dry-run` flag, when reindex runs, then the new content is printed to stdout without modifying BACKLOG.md
- AC-003.7: Write is atomic: tempfile + rename pattern
- AC-003.8: Given files with malformed frontmatter, when reindex runs, then those files are skipped with a stderr warning and the rest are indexed
- AC-003.9: Given BACKLOG.md does not exist yet, when reindex runs, then it creates the file with header, table, and stats (no Dependency Graph section preserved since there is none)

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `ecc-domain/src/backlog/` (new) | Domain | `BacklogEntry` struct, frontmatter parser, Levenshtein distance, index generator |
| `ecc-app/src/backlog.rs` (new) | App | `next_id()`, `check_duplicates()`, `reindex()` use cases |
| `ecc-cli/src/commands/backlog.rs` (new) | CLI | `BacklogArgs`, `BacklogAction` enum, `run()` handler |
| `ecc-cli/src/main.rs` | CLI | Add `Backlog` variant to `Command` enum |
| `ecc-cli/src/commands/mod.rs` | CLI | Add `pub mod backlog;` |

## Constraints

- `ecc-domain` must have zero I/O imports — frontmatter parsing operates on `&str`, not files
- All file I/O goes through `&dyn FileSystem` port in the app layer
- All tests use `InMemoryFileSystem` — no real filesystem in unit tests
- Output format must be machine-parseable: `next-id` prints plain text, `check-duplicates` prints JSON, `reindex` prints nothing on success (or `--dry-run` prints the generated content)
- BACKLOG.md table format must match the current format exactly (column names, separator style)
- `serde_yaml` must be added to `ecc-domain/Cargo.toml` (not currently in workspace); run `cargo audit` after adding
- Required frontmatter fields for `reindex`: `id`, `title`, `status`, `created`; optional: `scope`, `tier`, `target`/`target_command`, `tags`
- Required frontmatter fields for `check-duplicates`: `title`, `status`; optional: `tags`

## Non-Requirements

- No LLM-powered semantic duplicate detection
- No migration of existing entries to a new frontmatter schema
- No backlog-curator agent changes (separate future work)
- No interactive prompts in CLI commands
- No Windows-specific path handling (Unix-only like the rest of the project)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| FileSystem | Read existing | Uses `read_dir`, `read_to_string`, `write` — all existing methods |
| TerminalIO | New output | `next-id` and `check-duplicates` write to stdout |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New CLI commands | CLAUDE.md | `## CLI Commands` section | Add `ecc backlog next-id\|check-duplicates\|reindex` |
| New domain module | MODULE-SUMMARIES | `docs/MODULE-SUMMARIES.md` | Add ecc-domain/backlog entry |

## Open Questions

None — all questions resolved in grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Scope boundaries | 4 exclusions: no semantic detection, no schema migration, no agent changes, no interactive CLI | Recommended |
| 2 | Edge cases | Skip bad files with warning, max+1 IDs, warn on duplicates, reindex is authoritative | Recommended |
| 3 | Test strategy | 100% domain, 80% app, integration-only CLI; InMemoryFileSystem for all tests | Recommended |
| 4 | Performance | <100ms for 100-entry backlog; no lazy loading needed | Recommended |
| 5 | Security | No concerns — local filesystem only, no secrets, no network | Recommended |
| 6 | Breaking changes | None — additive CLI, same table format, read-only frontmatter | Recommended |
| 7 | Domain concepts | BacklogEntry + DuplicateCandidate types in ecc-domain/src/backlog/ | Recommended |
| 8 | ADR decisions | No ADR needed — standard CLI addition | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Generate next sequential backlog ID | 5 | none |
| US-002 | Detect duplicate backlog entries | 8 | none |
| US-003 | Regenerate backlog index | 9 | none |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | next-id returns BL-076 for 75 entries | US-001 |
| AC-001.2 | next-id returns BL-001 for empty dir | US-001 |
| AC-001.3 | next-id uses max+1, no gap filling | US-001 |
| AC-001.4 | next-id ignores non-BL files | US-001 |
| AC-001.5 | next-id errors on missing dir | US-001 |
| AC-002.1 | Fuzzy match scores >= 0.6 for similar titles | US-002 |
| AC-002.2 | Tag boost +0.15 per match, capped at +0.3 | US-002 |
| AC-002.3 | Empty array for no matches | US-002 |
| AC-002.4 | Filters to open/in-progress only | US-002 |
| AC-002.5 | JSON output sorted by score desc | US-002 |
| AC-002.6 | Skips malformed files with warning | US-002 |
| AC-002.7 | Explicit scoring formula defined | US-002 |
| AC-002.8 | Empty query errors with usage message | US-002 |
| AC-003.1 | Regenerates table for all valid entries | US-003 |
| AC-003.2 | Table columns match current format | US-003 |
| AC-003.3 | Sorted by numeric ID | US-003 |
| AC-003.4 | Auto-generated Stats section | US-003 |
| AC-003.5 | Preserves Dependency Graph section | US-003 |
| AC-003.6 | --dry-run prints without writing | US-003 |
| AC-003.7 | Atomic write via tempfile + rename | US-003 |
| AC-003.8 | Skips malformed files with warning | US-003 |
| AC-003.9 | Creates BACKLOG.md if absent | US-003 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Ambiguity | 62→85 | PASS (after fixes) | Scoring formula, tag boost, status filtering now explicit |
| Edge Cases | 72 | PASS | Main paths covered; empty query + missing BACKLOG.md added |
| Scope | 82 | PASS | Well-bounded, 3 subcommands, no agent changes |
| Dependencies | 55→80 | PASS (after fixes) | serde_yaml dependency acknowledged, no other external deps |
| Testability | 85 | PASS | All ACs testable with InMemoryFileSystem |
| Decisions | 78 | PASS | 8 decisions documented with rationale |
| Rollback | 75 | PASS | Additive changes, clean revert possible |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-27-deterministic-backlog-management/spec.md | Full spec |
