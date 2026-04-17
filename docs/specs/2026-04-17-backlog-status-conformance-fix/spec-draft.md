# Spec: Backlog Status Conformance Fix

## Problem Statement

The ECC backlog system stores entry status in two locations: YAML frontmatter in individual `docs/backlog/BL-*.md` files, and a generated `BACKLOG.md` index table. There is no programmatic mechanism to transition an entry's status (e.g., `open` -> `implemented`). Manual edits update only BACKLOG.md, while `ecc backlog reindex` reads from individual files — creating a dual source-of-truth that has caused entries to diverge: many entries marked "open" in files are "implemented" in the index, plus archived mismatches and other discrepancies. Running `reindex` today would revert these status changes. Additionally, YAML serialization quoting is inconsistent (57 quoted, 43 unquoted) because two different write paths produce different output.

## Research Summary

- Individual files should be the single source of truth; the index is a derived/cached artifact (agilemarkdown, Obsidian, markdown-vault-mcp patterns)
- Status updates must parse and rewrite frontmatter in-place, preserving the markdown body below the `---` delimiter
- YAML round-trip corruption is a known pitfall — only write files when values actually changed; avoid full-file re-serialization that changes quoting/key order
- Content-hash change detection prevents unnecessary rewrites and noisy git diffs
- The `implemented` status is a valid `BacklogStatus` enum variant but is not documented as a valid transition in the backlog-management skill
- Field ownership pattern: tooling should own machine-derived fields (id, status), humans own narrative fields (title, notes)

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Individual files are the canonical source of truth | Aligns with existing `reindex` architecture (reads files, generates index) | No |
| 2 | Add `update-status` CLI command rather than batch script | Prevents future drift by providing a repeatable, tested code path | No |
| 3 | Use in-place frontmatter rewrite: only modify the `status:` line within YAML frontmatter, return full content with all other bytes unchanged | Avoids YAML quoting/ordering corruption and body loss. Not a full serde round-trip. | No |
| 4 | Normalize YAML quoting during migration | Makes grep reliable; 1-time noisy diff is acceptable. Non-status YAML fields are NOT re-quoted. | No |
| 5 | Add reindex safety warning (>5 status changes) | Safeguard against accidental status reversion | No |
| 6 | Fix ERR-004 (swallowed lock removal error) — log `tracing::warn!` and continue | Already touching the module; addresses audit debt | No |
| 7 | Update backlog-curator agent + spec templates | Prevents drift from recurring via manual edits | No |
| 8 | Any-to-any status transitions are allowed | No transition validation graph is enforced. The CLI validates the target is a known status, but does not restrict source→target pairs. | No |
| 9 | Domain function uses `&str` for status (not `BacklogStatus`) | The function operates on raw file content, not parsed structs. Callers in the app layer validate against `BacklogStatus` before calling. | No |
| 10 | `save_entry` serialization quoting is a separate concern | This spec only normalizes existing files via migration. `save_entry` still uses `serde_saphyr::to_string` for new entries. | No |

## User Stories

### US-001: Update backlog entry status via CLI

**As a** developer, **I want** to run `ecc backlog update-status BL-NNN <status>`, **so that** the individual file and index stay in sync without manual editing.

#### Acceptance Criteria

- AC-001.1: Given a valid BL-NNN id and valid status, when `update-status` is run, then the individual file's YAML frontmatter `status:` line is updated (markdown body below closing `---` preserved character-for-character)
- AC-001.2: Given a successful frontmatter update, when `update-status` completes, then `reindex` is automatically run to regenerate BACKLOG.md
- AC-001.3: Given an invalid BL-NNN id (not found in backlog dir), when `update-status` is run, then a clear error is returned with exit code 1
- AC-001.4: Given a status value not in {open, implemented, archived, promoted, in-progress}, when `update-status` is run, then an error listing the 5 valid statuses is returned. The `Unknown` BacklogStatus variant is not a valid target.
- AC-001.5: Given the entry already has the requested status, when `update-status` is run, then no file write occurs and exit code 0 is returned (no-op guard)
- AC-001.6: Given a file with valid frontmatter but no `status:` field, when `replace_frontmatter_status` is called, then `BacklogError::MalformedYaml("status field not found")` is returned
- AC-001.7: Given a file with duplicate `status:` keys in YAML frontmatter, when `replace_frontmatter_status` is called, then only the first `status:` line within the frontmatter block is updated

#### Dependencies

- Depends on: none

### US-002: Sync divergent entries via migration

**As a** developer, **I want** the entries with mismatched statuses to be corrected, **so that** files and index agree and `reindex` is idempotent.

#### Acceptance Criteria

- AC-002.1: The migration dynamically computes divergent entries by comparing each file's frontmatter `status` against the BACKLOG.md index row. No hardcoded entry counts or IDs.
- AC-002.2: Given entries where files say `open` but index says `implemented`, when migration runs, then files are updated to `implemented`
- AC-002.3: Given BL-039 and BL-042 where files say `open` but should be `archived`, when migration runs, then files are updated to `archived`
- AC-002.4: Given all entry files, when migration completes, then the `status:` field's YAML quoting is normalized to unquoted values (e.g., `status: implemented` not `status: "implemented"`). Only the `status` field is normalized — other YAML fields are not re-quoted.
- AC-002.5: Given the migration, when `ecc backlog reindex --dry-run` is run after, then the output matches the current BACKLOG.md content (idempotent proof)
- AC-002.6: The migration is implemented as an internal function `migrate_statuses(store, index_store, backlog_dir) -> Result<MigrationReport>` in `ecc-app`. It is NOT individual CLI invocations.
- AC-002.7: Given migration fails on any entry (malformed YAML, missing file, permission error), when the error occurs, then the failing entry ID and error are logged, remaining entries are still processed (best-effort), and the final `MigrationReport` reports successes and failures separately.
- AC-002.8: Migration is committed as a single atomic git commit to enable clean revert

#### Dependencies

- Depends on: US-001 (uses `replace_frontmatter_status` domain function)

### US-003: Reindex safety warning

**As a** developer, **I want** `ecc backlog reindex` to warn me when it would change >5 statuses, **so that** I don't accidentally revert manual edits.

#### Acceptance Criteria

- AC-003.1: Given reindex detects >5 status changes between current index and what files contain, when run without `--force`, then a warning listing changed entries is printed to stderr, no write occurs, and exit code 2 is returned. Rationale for exit 2 (not 0): the command was asked to reindex and did not — this is a non-success condition that CI scripts should detect.
- AC-003.2: Given reindex detects >5 status changes, when run with `--force`, then the write proceeds with a logged warning listing the changed entries

#### Dependencies

- Depends on: none

### US-004: Fix ERR-004 (swallowed lock removal error)

**As a** developer, **I want** lock removal failures to be logged rather than silently swallowed, **so that** stale lock accumulation is diagnosable.

#### Acceptance Criteria

- AC-004.1: Given lock removal fails during stale lock cleanup in `ecc-app/src/backlog.rs`, when the failure occurs, then a warning is logged via `tracing::warn!` containing the lock ID and error message, and execution continues

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `ecc-domain/src/backlog/entry.rs` | Domain | Add `replace_frontmatter_status(content: &str, new_status: &str) -> Result<String, BacklogError>` pure function |
| `ecc-ports/src/backlog.rs` | Ports | Add `update_entry_status(&self, backlog_dir, id, new_status) -> Result<(), BacklogError>` to `BacklogEntryStore` |
| `ecc-infra/src/fs_backlog.rs` | Infra | Implement `update_entry_status`: locate file by id prefix, read content, call domain function, write atomically (tmp+rename) to same path |
| `ecc-app/src/backlog.rs` | App | Add `update_status()` use case + `migrate_statuses()` function + reindex safety check. Fix ERR-004. |
| `ecc-cli/src/commands/backlog.rs` | CLI | Add `UpdateStatus { id: String, status: String }` + `Migrate` variants to `BacklogAction` + dispatch |
| `ecc-test-support/src/in_memory_backlog.rs` | Test | Implement `update_entry_status` for in-memory double |
| `agents/backlog-curator.md` | Agent | Reference `ecc backlog update-status` for promote/archive |
| `skills/backlog-management/SKILL.md` | Skill | Document `implemented` as valid transition |

## Constraints

- Must not change the `BacklogEntry` struct or serde derive (backward compat)
- In-place frontmatter rewrite must preserve markdown body exactly (character-for-character below closing `---` delimiter)
- YAML quoting normalization applies only to the `status` field for the 5 known status strings
- `ecc-domain` must remain pure (no I/O) — `replace_frontmatter_status` operates on strings only
- The `BacklogEntryStore` port trait cannot import filesystem types
- `update_entry_status` infra implementation must use atomic write (write to temp file, rename) to prevent partial writes on crash
- Concurrent access to `update-status` on the same file is not supported (single-user CLI tool assumption, documented in help text)
- Migration must not be blocked by the reindex safety warning (US-003). Migration calls reindex once at the end with `--force`.

## Non-Requirements

- Not restructuring the backlog file format
- Not adding a full backlog CRUD API (only status updates)
- Not automating status transitions from the spec pipeline (future backlog entry)
- Not addressing swallowed errors outside `ecc-app/src/backlog.rs`
- Not re-quoting non-status YAML fields during migration

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| BacklogEntryStore | New method | Need integration test with real FS adapter |
| BacklogIndexStore | No change | Existing reindex tests sufficient |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New CLI commands | Project | CLAUDE.md | Add `ecc backlog update-status` and `ecc backlog migrate` to CLI section |
| Skill update | Skill | skills/backlog-management/SKILL.md | Document `implemented` transition + CLI reference |
| Agent update | Agent | agents/backlog-curator.md | Reference CLI for status changes |
| Changelog | Docs | CHANGELOG.md | Add fix entry |

## Open Questions

None — all resolved during grill-me interview and adversarial review.
