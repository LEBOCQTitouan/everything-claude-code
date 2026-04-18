# Tasks: Domain-Specialized Agent Generator

## Status Key
- `pending` — not started
- `red@<ts>` — test written, failing (RED)
- `green@<ts>` — implementation passing (GREEN)
- `done@<ts>` — refactored and committed (DONE)

## Pass Conditions

| PC | Status | Description | Verifies | Wave |
|----|--------|-------------|----------|------|
| PC-001 | pending | AgentFrontmatter with generated fields validates | AC-003.4 | 1 |
| PC-002 | pending | AgentFrontmatter without generated fields (compat) | AC-003.4 | 1 |
| PC-003 | pending | validate agents passes for agents/domain/ valid | AC-003.1 | 1 |
| PC-004 | pending | validate agents fails for agents/domain/ invalid | AC-003.2 | 1 |
| PC-005 | pending | validate agents succeeds when agents/domain/ absent | AC-003.3 | 1 |
| PC-006 | pending | Validated count includes domain subdir | AC-003.1 | 1 |
| PC-007 | pending | clippy ecc-domain | Build | 1 |
| PC-008 | pending | clippy ecc-app | Build | 1 |
| PC-009 | pending | cargo build | Build | 1 |
| PC-010 | pending | Skill passes validation | AC-006.1 | 2 |
| PC-011 | pending | Correct name + origin | AC-006.1 | 2 |
| PC-012 | pending | Documents agents/domain/ | AC-006.2 | 2 |
| PC-013 | pending | Under 500 words | AC-006.3 | 2 |
| PC-014 | pending | Graceful degradation note | AC-006.4 | 2 |
| PC-015 | pending | Command passes validation | AC-001.4 | 3 |
| PC-016 | pending | bounded-contexts + AskUserQuestion | AC-001.1 | 3 |
| PC-017 | pending | Missing bounded-contexts.md handled | AC-001.7 | 3 |
| PC-018 | pending | Empty bounded-contexts.md handled | AC-001.8 | 3 |
| PC-019 | pending | Missing source skip warning | AC-001.2 | 3 |
| PC-020 | pending | Single-file modules handled | AC-001.9 | 3 |
| PC-021 | pending | Extracts pub struct/enum, error, test | AC-001.3 | 3 |
| PC-022 | pending | Writes to agents/domain/ with generated frontmatter | AC-001.4 | 3 |
| PC-023 | pending | Asks before overwriting | AC-001.5 | 3 |
| PC-024 | pending | Commits with conventional message | AC-001.6 | 3 |
| PC-025 | pending | Domain Model section | AC-002.1 | 4 |
| PC-026 | pending | Error Catalogue section | AC-002.2 | 4 |
| PC-027 | pending | Test Patterns section | AC-002.3 | 4 |
| PC-028 | pending | Cross-Module Dependencies section | AC-002.4 | 4 |
| PC-029 | pending | Naming Conventions section | AC-002.5 | 4 |
| PC-030 | pending | Verification step for 5 patterns | AC-002.6 | 4 |
| PC-045 | pending | No leaked placeholders | AC-002.6 | 4 |
| PC-034 | pending | spec-dev.md Phase 0.7 | AC-004.1 | 5 |
| PC-035 | pending | spec-fix.md Phase 0.7 | AC-004.1 | 5 |
| PC-036 | pending | spec-refactor.md Phase 0.7 | AC-004.1 | 5 |
| PC-037 | pending | design.md Phase 0.7 | AC-004.2 | 5 |
| PC-038 | pending | implement.md Phase 0.7 | AC-004.3 | 5 |
| PC-039 | pending | Exact match in Phase 0.7 | AC-004.1 | 5 |
| PC-040 | pending | design Phase 0.7 Affected Modules | AC-004.2 | 5 |
| PC-041 | pending | implement Phase 0.7 File Changes | AC-004.3 | 5 |
| PC-042 | pending | Read-only subagent tools | AC-004.4 | 5 |
| PC-043 | pending | Graceful degradation absent dir | AC-004.5 | 5 |
| PC-044 | pending | Cap at 3 alphabetically | AC-004.6 | 5 |
| PC-046 | pending | Pipeline commands validation | AC-004.7 | 5 |
| PC-047 | pending | Domain Context output | AC-004.4 | 5 |
| PC-048 | pending | Tokenize + bounded-contexts | AC-004.1 | 5 |
| PC-031 | pending | Staleness git log --since | AC-005.1 | 6 |
| PC-032 | pending | --check-staleness flag | AC-005.2 | 6 |
| PC-033 | pending | Git unavailable handling | AC-005.3 | 6 |
| PC-049 | pending | Exit codes for staleness | AC-005.2 | 6 |
| PC-050 | pending | cargo fmt --check | Build | 6 |
| PC-051 | pending | cargo test | Build | 6 |

## Post-TDD

| Phase | Status |
|-------|--------|
| E2E tests | pending |
| Code review | pending |
| Doc updates | pending |
| Supplemental docs | pending |
| Write implement-done.md | pending |
