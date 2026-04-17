# Tasks: BL-129 Bidirectional Pipeline Transitions

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
| PC-001 | pending | Full forward lifecycle test (backfill) | AC-004.1 | 0 |
| PC-003 | pending | TransitionPolicy::default() has 6 forward pairs | AC-001.1 | 1 |
| PC-020 | pending | Direction serde roundtrip | AC-003.2 | 1 |
| PC-023 | pending | MissingJustification error message | AC-002.7 | 1 |
| PC-002 | pending | All 30 existing tests pass unchanged | AC-001.3, AC-004.2 | 2 |
| PC-004 | pending | Forward via TransitionResolver trait | AC-001.2 | 2 |
| PC-005 | pending | Backward with justification via policy | AC-001.4 | 2 |
| PC-006 | pending | Backward missing justification error | AC-002.2 | 2 |
| PC-007 | pending | Empty justification error | AC-002.7 | 2 |
| PC-008 | pending | Whitespace justification error | AC-002.7 | 2 |
| PC-021 | pending | Done backward still illegal | AC-001.1 | 2 |
| PC-022 | pending | Forward accepts None justification | AC-001.2 | 2 |
| PC-009 | pending | clear_artifacts implement→solution | AC-002.3, AC-002.8 | 3 |
| PC-010 | pending | clear_artifacts solution→plan | AC-002.4, AC-002.8 | 3 |
| PC-011 | pending | clear_artifacts implement→plan | AC-002.6, AC-002.8 | 3 |
| PC-013 | pending | TransitionRecord serde roundtrip | AC-003.2 | 3 |
| PC-014 | pending | history defaults to [] for old JSON | AC-003.1, AC-003.2 | 3 |
| PC-012 | pending | Forward re-entry re-stamps timestamp | AC-002.5 | 4 |
| PC-015 | pending | Binary backward impl→solution | AC-002.1, AC-002.3, AC-003.1 | 4 |
| PC-016 | pending | Binary backward no --justify blocks | AC-002.2 | 4 |
| PC-017 | pending | History displays chronologically | AC-003.3 | 5 |
| PC-018 | pending | History --json outputs JSON array | AC-003.4 | 5 |
| PC-019 | pending | Reset preserves history in archive | AC-003.5 | 6 |
| PC-024 | pending | clippy ecc-domain | Build | 7 |
| PC-025 | pending | clippy ecc-workflow | Build | 7 |
| PC-026 | pending | clippy ecc-cli | Build | 7 |
| PC-027 | pending | cargo test (full workspace) | AC-001.3, AC-004.2 | 7 |
| PC-028 | pending | cargo fmt --check | Build | 7 |
| PC-029 | pending | cargo build | Build | 7 |

## Post-TDD

| Phase | Status |
|-------|--------|
| E2E tests | pending |
| Code review | pending |
| Doc updates | pending |
| Supplemental docs | pending |
| Write implement-done.md | pending |
