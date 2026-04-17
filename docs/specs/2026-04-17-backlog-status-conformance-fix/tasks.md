# Tasks: Backlog Status Conformance Fix

## Status Key
- `pending` — not started
- `red@<ts>` — test written, failing (RED)
- `green@<ts>` — implementation passing (GREEN)
- `done@<ts>` — refactored and committed (DONE)
- `failed@<ts>` — failed after budget exceeded
- `skipped` — skipped by user

## Pass Conditions

| PC | Status | Description | Verifies | Wave |
|----|--------|-------------|----------|------|
| PC-001 | pending | replace_frontmatter_status updates status, preserves body | AC-001.1 | 1 |
| PC-002 | pending | No-op when status matches | AC-001.5 | 1 |
| PC-003 | pending | Error on missing status field | AC-001.6 | 1 |
| PC-004 | pending | Updates only first status: on duplicates | AC-001.7 | 1 |
| PC-005 | pending | Strips YAML quotes from status | AC-002.4 | 1 |
| PC-006 | pending | Ignores status: in body after --- | AC-001.1 | 1 |
| PC-007 | pending | from_kebab valid and invalid | AC-001.4 | 1 |
| PC-020 | pending | InMemory update_entry_status roundtrip | AC-001.1 | 2 |
| PC-021 | pending | FsBacklog atomic write | AC-001.1 | 3 |
| PC-022 | pending | FsBacklog preserves body | AC-001.1 | 3 |
| PC-008 | pending | update_status errors on invalid id | AC-001.3 | 4 |
| PC-009 | pending | update_status errors on invalid status | AC-001.4 | 4 |
| PC-010 | pending | update_status triggers reindex | AC-001.2 | 4 |
| PC-011 | pending | update_status no-op for same status | AC-001.5 | 4 |
| PC-019 | pending | Lock removal failure logged | AC-004.1 | 4 |
| PC-012 | pending | parse_index_statuses extracts map | AC-002.1 | 5 |
| PC-013 | pending | migrate_statuses dynamic divergence | AC-002.1-2.3 | 5 |
| PC-014 | pending | Migration partial failure | AC-002.7 | 5 |
| PC-015 | pending | MigrationReport structure | AC-002.6-2.7 | 5 |
| PC-016 | pending | Reindex blocks >5 without force | AC-003.1 | 5 |
| PC-017 | pending | Reindex allows with force | AC-003.2 | 5 |
| PC-018 | pending | Reindex no warning <=5 | AC-003.1 | 5 |
| PC-028 | pending | Migration idempotent proof | AC-002.5 | 5 |
| PC-029 | pending | Quoting normalized | AC-002.4 | 5 |
| PC-023 | pending | CLI update-status valid exit 0 | AC-001.2 | 6 |
| PC-024 | pending | CLI invalid id exit 1 | AC-001.3 | 6 |
| PC-025 | pending | CLI invalid status exit 1 | AC-001.4 | 6 |
| PC-026 | pending | CLI reindex safety exit 2 | AC-003.1 | 6 |
| PC-027 | pending | CLI reindex --force exit 0 | AC-003.2 | 6 |
| PC-030 | pending | cargo clippy -- -D warnings | Build | 7 |
| PC-031 | pending | cargo fmt --check | Build | 7 |
| PC-032 | pending | cargo test | Build | 7 |

## Post-TDD

| Phase | Status |
|-------|--------|
| E2E tests | pending |
| Code review | pending |
| Doc updates | pending |
| Supplemental docs | pending |
| Write implement-done.md | pending |
