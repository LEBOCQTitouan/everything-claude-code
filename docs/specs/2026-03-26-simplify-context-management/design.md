# Solution: Simplify Context Management — Remove Graceful Exit Infrastructure (BL-060)

## Spec Reference
Concern: refactor, Feature: simplify-context-management

## File Changes (dependency order)

### Deletions (Phase 1)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `skills/graceful-exit/SKILL.md` | delete | Core graceful exit skill replaced by native plan-accept context clear | US-001, AC-001.1 |
| 2 | `skills/graceful-exit/read-context-percentage.sh` | delete | Reader script for deleted graceful exit system | US-001, AC-001.2 |
| 3 | `skills/strategic-compact/` (entire directory) | delete | SKILL.md coupled to graceful exit; suggest-compact.sh superseded by Rust handler in dev_hooks.rs. Delete entire directory. | US-001, AC-001.3 |
| 4 | `docs/specs/2026-03-23-graceful-mid-session-exit/spec.md` | delete | Historical spec for removed feature | US-001, AC-001.4 |
| 5 | `docs/specs/2026-03-23-graceful-mid-session-exit/design.md` | delete | Historical design for removed feature | US-001, AC-001.4 |
| 6 | `docs/specs/2026-03-23-graceful-mid-session-exit/tasks.md` | delete | Historical tasks for removed feature | US-001, AC-001.4 |

### Modifications (Phase 2-6)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 7 | `commands/implement.md` | modify | Remove 5 Context Checkpoint blocks (Phases 3-7.5), Phase 0 step 7 (Context Clear Gate), Phase 0 step 6 Resumption Pointer ref, all `read-context-percentage` refs | US-002, AC-002.1-002.6 |
| 8 | `commands/audit-full.md` | modify | Remove Phase 0 "Graceful Exit Recovery" section (lines 18-29), Phase 4 "Cleanup Partial Results" section (lines 241-249) | US-003, AC-003.1-003.2 |
| 9 | `agents/audit-orchestrator.md` | modify | Remove `graceful-exit` from skills frontmatter, remove graceful-exit skill ref from Reference Skills, remove "Context Checkpoint After Each Domain Agent" section, remove "Skip Completed Domains on Re-entry" section | US-003, AC-003.3-003.5 |
| 10 | `skills/campaign-manifest/SKILL.md` | modify | Remove `## Resumption Pointer` from schema, remove "Context checkpoint" from Incremental Updates, remove "Resumption Pointer" from Malformed Recovery required headers | US-004, AC-004.1-004.4 |
| 11 | `commands/design.md` | modify | Remove Resumption Pointer update from Phase 10 line 255 | US-004, AC-004.3 |
| 12 | `docs/adr/0014-context-aware-graceful-exit.md` | modify | Change Status from "Accepted" to "Superseded by BL-060" | US-005, AC-005.1 |
| 13 | `docs/adr/README.md` | modify | Change ADR 0014 row status to "Superseded" | US-005, AC-005.5 |
| 14 | `docs/domain/glossary.md` | modify | Remove entries: "### Graceful Exit", "### Context Checkpoint", "### Resumption Pointer" | US-005, AC-005.2-005.4 |
| 15 | `tests/test-campaign-manifest.sh` | modify | Remove `test_strategic_compact_campaign` function and call, update `test_campaign_manifest_skill` to not assert Resumption Pointer, update `test_glossary_entries` to not assert Resumption Pointer | US-006, AC-006.6 |
| 16 | `skills/prompt-optimizer/SKILL.md` | modify | Remove `strategic-compact` row from table | US-006, AC-006.7 |
| 17 | `skills/configure-ecc/SKILL.md` | modify | Remove `strategic-compact` from Core bundle description and skill table | US-006, AC-006.7 |
| 18 | `hooks/README.md` | modify | Remove strategic-compact link from Related section | US-006, AC-006.7 |
| 19 | `docs/longform-guide.md` | modify | Update strategic compact mention to generic compaction guidance | US-006, AC-006.7 |
| 20 | `docs/token-optimization.md` | modify | Remove `skills/strategic-compact/` references, update strategic compaction section | US-006, AC-006.7 |
| 21 | `docs/DEPENDENCY-GRAPH.md` | modify | No change needed — `suggest-compact` is a hook script (kept), not the deleted skill | US-006, AC-006.7 |
| 22 | `CHANGELOG.md` | modify | Add BL-060 entry | US-007, AC-007.1 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | Graceful exit SKILL.md deleted | AC-001.1 | `! test -f skills/graceful-exit/SKILL.md` | exit 0 |
| PC-002 | unit | read-context-percentage.sh deleted | AC-001.2 | `! test -f skills/graceful-exit/read-context-percentage.sh` | exit 0 |
| PC-003 | unit | Strategic compact directory deleted | AC-001.3 | `! test -d skills/strategic-compact` | exit 0 |
| PC-004 | unit | Graceful exit spec directory deleted | AC-001.4 | `! test -d docs/specs/2026-03-23-graceful-mid-session-exit` | exit 0 |
| PC-005 | unit | No dangling graceful-exit/SKILL.md refs | AC-001.5 | `! grep -rl 'graceful-exit/SKILL.md' commands/ skills/ agents/ docs/adr/ docs/domain/ hooks/ --include='*.md'` | exit 0 |
| PC-006 | unit | No dangling read-context-percentage refs | AC-001.6 | `! grep -rl 'read-context-percentage' commands/ skills/ agents/ docs/adr/ docs/domain/ hooks/ --include='*.md'` | exit 0 |
| PC-007 | unit | No dangling strategic-compact/SKILL.md refs | AC-001.7 | `! grep -rl 'strategic-compact/SKILL.md' commands/ skills/ agents/ docs/adr/ docs/domain/ hooks/ --include='*.md'` | exit 0 |
| PC-008 | unit | No "Context Checkpoint" blockquotes in implement.md | AC-002.1 | `! grep -c 'Context Checkpoint' commands/implement.md` | exit 0 |
| PC-009 | unit | No read-context-percentage in implement.md | AC-002.2 | `! grep -q 'read-context-percentage' commands/implement.md` | exit 0 |
| PC-010 | unit | No 85% exit logic in implement.md | AC-002.3 | `! grep -q '85%' commands/implement.md` | exit 0 |
| PC-011 | unit | Phase 0 step 7 (Context Clear Gate) removed | AC-002.4 | `! grep -q 'Context Clear Gate' commands/implement.md` | exit 0 |
| PC-012 | unit | No Resumption Pointer in implement.md | AC-002.5 | `! grep -q 'Resumption Pointer' commands/implement.md` | exit 0 |
| PC-013 | unit | Campaign re-entry orientation still exists without Resumption Pointer | AC-002.6 | `grep -q 'Campaign re-entry orientation\|campaign.*re-entry\|campaign.*orientation' commands/implement.md` | exit 0 |
| PC-014 | unit | No "Graceful Exit Recovery" in audit-full.md | AC-003.1 | `! grep -q 'Graceful Exit Recovery' commands/audit-full.md` | exit 0 |
| PC-015 | unit | No "Cleanup Partial Results" in audit-full.md | AC-003.2 | `! grep -q 'Cleanup Partial Results' commands/audit-full.md` | exit 0 |
| PC-016 | unit | No graceful-exit in audit-orchestrator skills frontmatter | AC-003.3 | `! grep -q 'graceful-exit' agents/audit-orchestrator.md` | exit 0 |
| PC-017 | unit | No "Context Checkpoint After Each Domain Agent" in audit-orchestrator | AC-003.4 | `! grep -q 'Context Checkpoint After Each Domain Agent' agents/audit-orchestrator.md` | exit 0 |
| PC-018 | unit | No "Skip Completed Domains" in audit-orchestrator | AC-003.5 | `! grep -q 'Skip Completed Domains' agents/audit-orchestrator.md` | exit 0 |
| PC-019 | unit | No "## Resumption Pointer" in campaign-manifest skill | AC-004.1 | `! grep -q '## Resumption Pointer' skills/campaign-manifest/SKILL.md` | exit 0 |
| PC-020 | unit | No "Context checkpoint" in campaign-manifest Incremental Updates | AC-004.2 | `! grep -qi 'context checkpoint' skills/campaign-manifest/SKILL.md` | exit 0 |
| PC-021 | unit | No Resumption Pointer in design.md | AC-004.3 | `! grep -q 'Resumption Pointer' commands/design.md` | exit 0 |
| PC-022 | unit | No Resumption Pointer in campaign-manifest Malformed Recovery | AC-004.4 | `! grep -q 'Resumption Pointer' skills/campaign-manifest/SKILL.md` | exit 0 |
| PC-023 | unit | ADR-0014 status is Superseded | AC-005.1 | `grep -q 'Superseded by BL-060' docs/adr/0014-context-aware-graceful-exit.md` | exit 0 |
| PC-024 | unit | No "### Graceful Exit" in glossary | AC-005.2 | `! grep -q '### Graceful Exit' docs/domain/glossary.md` | exit 0 |
| PC-025 | unit | No "### Context Checkpoint" in glossary | AC-005.3 | `! grep -q '### Context Checkpoint' docs/domain/glossary.md` | exit 0 |
| PC-026 | unit | No "### Resumption Pointer" in glossary | AC-005.4 | `! grep -q '### Resumption Pointer' docs/domain/glossary.md` | exit 0 |
| PC-027 | unit | ADR index shows 0014 as Superseded | AC-005.5 | `grep '0014' docs/adr/README.md \| grep -q 'Superseded'` | exit 0 |
| PC-028 | unit | BL-054 status is archived | AC-006.1 | `grep -q 'status: archived' docs/backlog/BL-054-implement-compact-gate.md` | exit 0 |
| PC-029 | unit | BL-055 status is archived | AC-006.2 | `grep -q 'status: archived' docs/backlog/BL-055-graceful-mid-session-exit.md` | exit 0 |
| PC-030 | unit | No graceful-exit refs in commands/skills/agents | AC-006.3 | `! grep -rl 'graceful-exit' commands/ skills/ agents/` | exit 0 |
| PC-031 | unit | No Resumption.*Pointer refs in commands/skills/agents | AC-006.4 | `! grep -rEl 'Resumption.*Pointer' commands/ skills/ agents/` | exit 0 |
| PC-032 | unit | No read-context-percentage in active dirs | AC-006.5 | `! grep -rl 'read-context-percentage' commands/ skills/ agents/ hooks/ --include='*.md'` | exit 0 |
| PC-033 | unit | test-campaign-manifest.sh passes | AC-006.6 | `bash tests/test-campaign-manifest.sh` | exit 0 |
| PC-034 | unit | No strategic-compact refs in additional files | AC-006.7 | `! grep -rl 'strategic-compact' skills/prompt-optimizer/ skills/configure-ecc/ hooks/README.md docs/token-optimization.md` | exit 0 |
| PC-035 | unit | CHANGELOG has BL-060 entry | AC-007.1 | `grep -q 'BL-060' CHANGELOG.md` | exit 0 |
| PC-036 | build | cargo test passes | AC-007.2 | `cargo test` | PASS |
| PC-037 | lint | cargo clippy passes | AC-007.3 | `cargo clippy -- -D warnings` | PASS |
| PC-038 | build | cargo build passes | AC-007.4 | `cargo build` | PASS |

### Coverage Check

All 38 ACs covered:

| AC | Covering PC(s) |
|----|---------------|
| AC-001.1 | PC-001 |
| AC-001.2 | PC-002 |
| AC-001.3 | PC-003 |
| AC-001.4 | PC-004 |
| AC-001.5 | PC-005 |
| AC-001.6 | PC-006 |
| AC-001.7 | PC-007 |
| AC-002.1 | PC-008 |
| AC-002.2 | PC-009 |
| AC-002.3 | PC-010 |
| AC-002.4 | PC-011 |
| AC-002.5 | PC-012 |
| AC-002.6 | PC-013 |
| AC-003.1 | PC-014 |
| AC-003.2 | PC-015 |
| AC-003.3 | PC-016 |
| AC-003.4 | PC-017 |
| AC-003.5 | PC-018 |
| AC-004.1 | PC-019 |
| AC-004.2 | PC-020 |
| AC-004.3 | PC-021 |
| AC-004.4 | PC-022 |
| AC-005.1 | PC-023 |
| AC-005.2 | PC-024 |
| AC-005.3 | PC-025 |
| AC-005.4 | PC-026 |
| AC-005.5 | PC-027 |
| AC-006.1 | PC-028 |
| AC-006.2 | PC-029 |
| AC-006.3 | PC-030 |
| AC-006.4 | PC-031 |
| AC-006.5 | PC-032 |
| AC-006.6 | PC-033 |
| AC-006.7 | PC-034 |
| AC-007.1 | PC-035 |
| AC-007.2 | PC-036 |
| AC-007.3 | PC-037 |
| AC-007.4 | PC-038 |

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|

No E2E boundaries affected — pure markdown/shell deletion refactoring.

### Design Clarifications (from adversarial review)

**`suggest-compact.sh` deleted**: The Rust handler in `dev_hooks.rs` fully supersedes the shell script. The entire `skills/strategic-compact/` directory is deleted (SKILL.md + suggest-compact.sh). The Rust hook handler (`ecc-hook "pre:edit-write:suggest-compact"`) continues working independently.

**`docs/adr/0013-campaign-manifest-convention.md`**: Contains historical Resumption Pointer references in Consequences section. These are acceptable as historical context in an ADR — they document what was decided at the time. No modification needed.

**`docs/ARCHITECTURE.md`**: References `suggest-compact` in session lifecycle. This refers to the Rust hook handler (kept), not the deleted skill. No modification needed.

**Grep exclusions**: Absence checks exclude `docs/specs/`, `docs/audits/`, `docs/adr/0013-*`, `docs/backlog/BL-035-*`, and `docs/pre-rebase-inventory.md` as historical/inventory documents.

### E2E Activation Rules

No E2E tests activated. No port/adapter changes.

## Test Strategy

TDD order (deletion-first, then modifications by dependency depth):

1. **PC-001 through PC-004**: Delete files/directories (US-001 deletions)
2. **PC-005 through PC-007**: Verify no dangling refs from deletions (US-001 absence checks)
3. **PC-008 through PC-013**: Modify `commands/implement.md` (US-002)
4. **PC-014 through PC-018**: Modify `commands/audit-full.md` and `agents/audit-orchestrator.md` (US-003)
5. **PC-019 through PC-022**: Modify `skills/campaign-manifest/SKILL.md` and `commands/design.md` (US-004)
6. **PC-023 through PC-027**: Supersede ADR-0014 and update glossary (US-005)
7. **PC-028 through PC-034**: Archive backlog items, update test file, clean additional refs (US-006)
8. **PC-035**: Add CHANGELOG entry (US-007)
9. **PC-036 through PC-038**: Final quality gates — cargo test, clippy, build (US-007)

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/domain/glossary.md` | Domain | Remove 3 entries | Remove Graceful Exit, Context Checkpoint, Resumption Pointer | US-005 |
| 2 | `docs/adr/0014-context-aware-graceful-exit.md` | Architecture | Supersede | Change status to "Superseded by BL-060" | US-005 |
| 3 | `docs/adr/README.md` | Architecture | Update index | Mark ADR 0014 as Superseded | US-005 |
| 4 | `CHANGELOG.md` | Project | Add entry | Add BL-060 removal entry under Refactoring | US-007 |

## SOLID Assessment

PASS — Pure deletion refactoring. No new abstractions, no dependency changes, no Rust code changes. Reduces coupling by removing dead cross-references between skills, commands, and agents.

## Robert's Oath Check

CLEAN — Removing dead infrastructure reduces mess. No harmful code introduced. No tests removed (test file updated to reflect deleted files). Build integrity preserved.

## Security Notes

CLEAR — No security surfaces changed. No secrets, no auth, no input validation boundaries affected. Pure markdown/shell deletion.

## Rollback Plan

Reverse dependency order:
1. Revert CHANGELOG entry
2. Restore glossary entries, ADR status, backlog statuses
3. Restore campaign-manifest Resumption Pointer, design.md ref
4. Restore audit-orchestrator sections, audit-full sections
5. Restore implement.md context checkpoints and Phase 0 gate
6. Restore additional file references (prompt-optimizer, configure-ecc, hooks README, longform-guide, token-optimization)
7. Restore test-campaign-manifest.sh assertions
8. Restore deleted files: `skills/graceful-exit/`, `skills/strategic-compact/`, `docs/specs/2026-03-23-graceful-mid-session-exit/`

Or: `git revert` the merge commit.
