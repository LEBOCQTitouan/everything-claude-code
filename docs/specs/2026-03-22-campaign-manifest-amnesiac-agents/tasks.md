# Tasks: Campaign Manifest for Amnesiac Agents (BL-035)

## Pass Conditions

- [x] PC-001: artifact-schemas skill frontmatter + 5 schemas | done@2026-03-23T00:00:00Z
- [x] PC-002: campaign-manifest skill frontmatter + schema sections | done@2026-03-23T00:00:00Z
- [x] PC-003: workflow-init.sh toolchain field | done@2026-03-23T00:02:00Z
- [x] PC-004: toolchain-persist.sh writes commands + jq fallback | done@2026-03-23T00:02:00Z
- [x] PC-005: phase-transition.sh campaign_path support | done@2026-03-23T00:02:00Z
- [x] PC-006: scope-check.sh reads design_path | done@2026-03-23T00:02:00Z
- [x] PC-007: spec-pipeline-shared 5 campaign sections | done@2026-03-23T00:05:00Z
- [x] PC-008: spec-pipeline-shared Project Detection wording | done@2026-03-23T00:05:00Z
- [x] PC-009: Remove "Store mentally" from all spec commands | done@2026-03-23T00:05:00Z
- [x] PC-010: Spec commands reference shared campaign | done@2026-03-23T00:05:00Z
- [x] PC-011: Spec commands reference toolchain-persist | done@2026-03-23T00:05:00Z
- [x] PC-012: Spec commands reference draft spec persistence | done@2026-03-23T00:05:00Z
- [x] PC-013: design.md disk fallbacks | done@2026-03-23T00:07:00Z
- [x] PC-014: design.md campaign updates | done@2026-03-23T00:07:00Z
- [x] PC-015: wave-analysis skill extraction | done@2026-03-23T00:10:00Z
- [x] PC-016: wave-dispatch skill extraction | done@2026-03-23T00:10:00Z
- [x] PC-017: progress-tracking skill extraction | done@2026-03-23T00:10:00Z
- [x] PC-018: tasks-generation skill extraction | done@2026-03-23T00:10:00Z
- [x] PC-019: implement.md references skills + under 350 lines | done@2026-03-23T00:10:00Z
- [x] PC-020: implement.md Commit Trail campaign writes | done@2026-03-23T00:10:00Z
- [x] PC-021: implement.md Agent Outputs campaign writes | done@2026-03-23T00:10:00Z
- [x] PC-022: implement.md campaign re-entry orientation | done@2026-03-23T00:10:00Z
- [x] PC-023: strategic-compact campaign awareness | done@2026-03-23T00:12:00Z
- [x] PC-024: ADR 0013 | done@2026-03-23T00:12:00Z
- [x] PC-025: Glossary entries | done@2026-03-23T00:12:00Z
- [x] PC-026: CHANGELOG entry | done@2026-03-23T00:12:00Z
- [x] PC-027: Full test suite (37/37) | done@2026-03-23T00:15:00Z
- [x] PC-028: Pipeline-summaries backward compat (57/57) | done@2026-03-23T00:15:00Z
- [x] PC-029: Wave-parallel backward compat (40/40) | done@2026-03-23T00:15:00Z
- [x] PC-031: Rust build | done@2026-03-23T00:15:00Z

## Post-TDD

- [x] E2E tests | done@2026-03-23T00:16:00Z (none required)
- [x] Code review | done@2026-03-23T00:18:00Z (3 HIGH fixed: jq error handling, realpath portability, jq requirement)
- [x] Doc updates | done@2026-03-23T00:12:00Z (ADR 0013, glossary, CHANGELOG, backlog)
- [x] Write implement-done.md | done@2026-03-23T00:20:00Z

## Phase Summary

### Tasks Executed

| PC ID | Description | RED-GREEN Status | Commit Count |
|-------|-------------|------------------|--------------|
| PC-001 | artifact-schemas skill | GREEN | 1 |
| PC-002 | campaign-manifest skill | GREEN | 1 |
| PC-003–006 | Hook modifications | GREEN | 1 |
| PC-007–012 | Spec pipeline fixes | GREEN | 1 |
| PC-013–014 | Design disk fallbacks | GREEN | 1 |
| PC-015–022 | Implement decomposition | GREEN | 1 |
| PC-023–026 | Docs | GREEN | 1 |
| PC-027–031 | Test suite + backward compat | GREEN | 1 |

### Commits Made

| Hash (short) | Message |
|--------------|---------|
| b059de9 | feat: create artifact-schemas and campaign-manifest skills |
| f07d11f | feat: add toolchain persistence and campaign_path to hooks |
| edb9303 | feat: add campaign persistence to spec pipeline |
| 0e80327 | feat: add disk fallbacks and campaign updates to design command |
| f9f9258 | refactor: extract 4 sub-skills from implement.md, add campaign persistence |
| 418d4d4 | docs: add ADR 0013, glossary, CHANGELOG, backlog for BL-035 |
| d7ab6ed | test: add campaign manifest test suite and fix backward compat |
| fc42d3f | fix: address code review findings — jq error handling and realpath portability |

### Docs Updated

| Doc File | Level | What Changed |
|----------|-------|--------------|
| docs/adr/0013-campaign-manifest-convention.md | ADR | Campaign manifest convention |
| docs/domain/glossary.md | Domain | Campaign Manifest, Resumption Pointer |
| CHANGELOG.md | Project | BL-035 entry |
| docs/backlog/BACKLOG.md | Project | BL-035 → implemented |
| docs/adr/README.md | Index | Added ADR 0013 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| .claude/workflow/implement-done.md | Full implementation summary |
| docs/specs/2026-03-22-campaign-manifest-amnesiac-agents/tasks.md | Tasks with completion status |
