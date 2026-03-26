# Solution: Context-Aware Doc Generation at End of /implement (BL-056)

## Spec Reference
Concern: dev, Feature: Context-aware doc generation at end of implement (BL-056)

## File Changes (dependency order)
| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `agents/module-summary-updater.md` | create | Agent must exist before implement.md references it. Dedicated file for ECC convention compliance. | US-002, AC-002.1-010 |
| 2 | `agents/diagram-updater.md` | create | Agent must exist before implement.md references it. Separate from diagram-generator to avoid ownership conflicts. | US-003, AC-003.1-014 |
| 3 | `commands/implement.md` | modify | Core change: Phase 7.5, Phase 0 re-entry, Phase 2 TodoWrite, Phase 7 schema, Phase 8 checks, Related Agents. | US-001, US-004, US-005 |
| 4 | `.claude/hooks/doc-enforcement.sh` | modify | Extend to check `## Supplemental Docs` heading. WARNING-only, consistent with existing pattern. | US-006, AC-006.1-003 |
| 5 | `docs/diagrams/INDEX.md` | modify | New "Feature Implementation Diagrams" category protected from doc-suite regeneration. | US-007, AC-007.1-003 |
| 6 | `docs/adr/0015-feature-implementation-diagrams.md` | create | ADR for category separation rationale (Decision #4). | Decision #4 |
| 7 | `docs/domain/glossary.md` | modify | "Supplemental Docs" entry — new domain concept. | US-008, AC-008.1 |
| 8 | `CHANGELOG.md` | modify | BL-056 feature entry. | US-008, AC-008.2 |

## Pass Conditions
| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | module-summary-updater name field | AC-002.1 | `head -10 agents/module-summary-updater.md \| grep -c 'name: module-summary-updater'` | 1 |
| PC-002 | unit | module-summary-updater model haiku | AC-002.1 | `head -10 agents/module-summary-updater.md \| grep -c 'model: haiku'` | 1 |
| PC-003 | unit | module-summary-updater tools list | AC-002.1 | `head -10 agents/module-summary-updater.md \| grep -cE 'tools:.*Read.*Write.*Edit.*Grep.*Glob'` | 1 |
| PC-004 | unit | IMPLEMENT-GENERATED markers in agent | AC-002.8 | `grep -c 'IMPLEMENT-GENERATED' agents/module-summary-updater.md` | >= 1 |
| PC-005 | unit | Module defined as Rust crate | AC-002.9 | `grep -ci 'rust crate\|cargo workspace' agents/module-summary-updater.md` | >= 1 |
| PC-006 | unit | diagram-updater name field | AC-003.1 | `head -10 agents/diagram-updater.md \| grep -c 'name: diagram-updater'` | 1 |
| PC-007 | unit | diagram-updater model haiku | AC-003.1 | `head -10 agents/diagram-updater.md \| grep -c 'model: haiku'` | 1 |
| PC-008 | unit | diagram-updater tools list | AC-003.1 | `head -10 agents/diagram-updater.md \| grep -cE 'tools:.*Read.*Write.*Edit.*Grep.*Glob'` | 1 |
| PC-009 | unit | Header uses diagram-updater name | AC-003.10 | `grep -c 'diagram-updater' agents/diagram-updater.md` | >= 2 |
| PC-010 | unit | MUST NOT modify module-dependency-graph | AC-003.11 | `grep -ci 'must not modify.*module-dependency-graph\|never.*module-dependency-graph' agents/diagram-updater.md` | >= 1 |
| PC-011 | unit | MUST NOT register in CUSTOM.md | AC-003.13 | `grep -ci 'must not.*CUSTOM.md\|never.*CUSTOM.md' agents/diagram-updater.md` | >= 1 |
| PC-012 | unit | Concrete trigger heuristics | AC-003.12 | `grep -c '2+ crates' agents/diagram-updater.md` | >= 1 |
| PC-013 | unit | implement.md has Phase 7.5 section | AC-001.1 | `grep -c 'Phase 7.5' commands/implement.md` | >= 1 |
| PC-014 | unit | Supplemental docs mentioned 3+ times | AC-001.7, AC-001.4, AC-001.11 | `grep -ci 'supplemental doc' commands/implement.md` | >= 3 |
| PC-015 | unit | TodoWrite has Supplemental docs | AC-001.4 | `grep -c 'Supplemental docs' commands/implement.md` | >= 1 |
| PC-016 | unit | Schema has ## Supplemental Docs | AC-005.1, AC-005.3 | `grep -c '## Supplemental Docs' commands/implement.md` | >= 1 |
| PC-017 | unit | Table schema columns correct | AC-001.12 | `grep -c 'Subagent.*Status.*Output File.*Commit SHA.*Notes' commands/implement.md` | >= 1 |
| PC-018 | unit | Phase 8 checks Supplemental Docs | AC-001.11 | `sed -n '/Phase 8/,$ p' commands/implement.md \| grep -c 'Supplemental Docs'` | >= 1 |
| PC-019 | unit | Related Agents: module-summary-updater | AC-001.11 | `grep -c 'module-summary-updater' commands/implement.md` | >= 1 |
| PC-020 | unit | Related Agents: diagram-updater | AC-001.11 | `grep -c 'diagram-updater' commands/implement.md` | >= 1 |
| PC-021 | unit | implement.md under 800 lines | Constraint | `test $(wc -l < commands/implement.md) -lt 800 && echo PASS` | PASS |
| PC-022 | unit | Parallel dispatch with allowedTools | AC-001.2, AC-001.9 | `grep -c 'allowedTools' commands/implement.md` | >= 1 |
| PC-023 | unit | Cross-link fixup mentioned | AC-001.3, AC-001.10 | `grep -ci 'cross-link fixup\|fixup pass' commands/implement.md` | >= 1 |
| PC-024 | unit | Non-blocking failure handling | AC-001.8 | `grep -ci 'non-blocking\|partial failure' commands/implement.md` | >= 1 |
| PC-025 | unit | Context checkpoint in Phase 7.5 | AC-001.6 | `grep -A5 'Phase 7.5' commands/implement.md \| grep -ci 'context checkpoint\|graceful-exit'` | >= 1 |
| PC-026 | unit | Hook checks Supplemental Docs | AC-006.1 | `grep -c 'Supplemental Docs' .claude/hooks/doc-enforcement.sh` | >= 1 |
| PC-027 | unit | Hook emits WARNING | AC-006.2 | `grep -A2 'Supplemental Docs' .claude/hooks/doc-enforcement.sh \| grep -c 'WARNING'` | >= 1 |
| PC-028 | unit | Hook exits 0 on warning | AC-006.2 | `grep -A4 'Supplemental Docs' .claude/hooks/doc-enforcement.sh \| grep -c 'exit 0'` | >= 1 |
| PC-029 | unit | Hook has ECC_WORKFLOW_BYPASS | Constraint | `head -5 .claude/hooks/doc-enforcement.sh \| grep -c 'ECC_WORKFLOW_BYPASS'` | 1 |
| PC-030 | unit | Hook has set -uo pipefail | Constraint | `head -5 .claude/hooks/doc-enforcement.sh \| grep -c 'set -uo pipefail'` | 1 |
| PC-031 | unit | INDEX.md has Feature Implementation Diagrams | AC-007.1 | `grep -c '## Feature Implementation Diagrams' docs/diagrams/INDEX.md` | 1 |
| PC-032 | unit | INDEX.md explains Phase 7.5 origin | AC-007.3 | `grep -ci 'Phase 7.5\|/implement' docs/diagrams/INDEX.md` | >= 1 |
| PC-033 | unit | Section order correct | AC-003.14 | `awk '/Command Workflow/{a=1} /Feature Implementation/{if(a) b=1} /Coverage/{if(b) print "PASS"}' docs/diagrams/INDEX.md` | PASS |
| PC-034 | unit | Coverage table includes new category | AC-003.14 | `grep -c 'Feature implementation' docs/diagrams/INDEX.md` | >= 1 |
| PC-035 | unit | ADR 0015 exists | Decision #4 | `test -f docs/adr/0015-feature-implementation-diagrams.md && echo PASS` | PASS |
| PC-036 | unit | Glossary has Supplemental Docs | AC-008.1 | `grep -c '### Supplemental Docs' docs/domain/glossary.md` | 1 |
| PC-037 | unit | CHANGELOG has BL-056 | AC-008.2 | `grep -c 'BL-056' CHANGELOG.md` | >= 1 |
| PC-038 | lint | Markdown lint passes | Constraint | `npm run lint` | exit 0 |
| PC-039 | lint | Cargo clippy passes | Constraint | `cargo clippy -- -D warnings` | exit 0 |
| PC-040 | build | Cargo build passes | Constraint | `cargo build` | exit 0 |
| PC-041 | unit | CUSTOM.md unchanged | AC-003.13 | `git diff --name-only docs/diagrams/CUSTOM.md \| wc -l` | 0 |
| PC-042 | unit | All 4 diagram types in agent | AC-003.2-005 | `grep -c 'sequence\|flowchart\|dependency\|C4\|component' agents/diagram-updater.md` | >= 4 |
| PC-043 | unit | Format template in agent | AC-003.8, AC-003.10 | `grep -c 'Generated by diagram-updater\|mermaid' agents/diagram-updater.md` | >= 2 |
| PC-044 | unit | All 3 trigger heuristics | AC-003.12 | `grep -c '2+ crates\|3+ variants\|new crate' agents/diagram-updater.md` | >= 3 |
| PC-045 | unit | Cross-link fixup with no-op and commit | AC-004.1-003 | `grep -c 'cross-link\|fixup\|no diagram' commands/implement.md` | >= 3 |
| PC-046 | unit | Supplemental Docs placement in schema | AC-005.1-003 | `awk '/ADRs Created/{a=1} /Supplemental Docs/{if(a) b=1} /Subagent Execution/{if(b) print "PASS"}' commands/implement.md` | PASS |
| PC-047 | integration | Hook detects missing Supplemental Docs | AC-006.1-002 | `bash -c 'IMPL=$(mktemp); echo "## Docs Updated" > "$IMPL"; echo "- CHANGELOG" >> "$IMPL"; IMPL_FILE="$IMPL" CLAUDE_PROJECT_DIR=$(pwd) ECC_WORKFLOW_BYPASS=0 bash -c "export IMPL_FILE; sed \"s|\$PROJECT_DIR/.claude/workflow/implement-done.md|\$IMPL_FILE|\" .claude/hooks/doc-enforcement.sh \| bash" 2>&1; rm "$IMPL"' \| grep -c WARNING` | >= 1 |

### Coverage Check
All 50 ACs covered by 47 PCs.

### E2E Test Plan
No E2E boundaries affected — pure orchestration and documentation changes.

### E2E Activation Rules
No E2E tests to activate.

## Test Strategy
TDD order:
1. **Phase 1 (PC-001–012)**: Create both agent files. Independent, can be parallel.
2. **Phase 2 (PC-013–025, PC-045–046)**: Modify implement.md — Phase 7.5, re-entry, TodoWrite, schema, Phase 8, Related Agents.
3. **Phase 3 (PC-026–044, PC-047)**: Hook extension, INDEX.md, ADR, glossary, CHANGELOG, regression gates.

## Doc Update Plan
| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/domain/glossary.md` | Domain | Add "Supplemental Docs" entry | Context-aware docs from /implement Phase 7.5 | US-008 AC-008.1 |
| 2 | `CHANGELOG.md` | Project | Add BL-056 entry | Phase 7.5, two agents, non-blocking, ADR 0015 | US-008 AC-008.2 |
| 3 | `docs/adr/0015-feature-implementation-diagrams.md` | Architecture | Create ADR | Category separation: one-time snapshots not subject to regeneration | Decision #4 |

## SOLID Assessment
PASS with notes. SRP concern about Phase 7.5 inline in implement.md — mitigated by consistency with existing dispatch patterns (tdd-executor, code-reviewer both dispatched inline). No actionable violations.

## Robert's Oath Check
CLEAN. 8/9 oath promises clean. Rework ratio 0.10 (healthy). Non-blocking failures protect the pipeline. 47 PCs cover all 50 ACs.

## Security Notes
CLEAR. No CRITICAL or HIGH findings. Two LOW items: add explicit write scope constraints to both agents, confirm hook exit-0 is intentional.

## Rollback Plan
Reverse dependency order:
1. Revert `CHANGELOG.md` changes
2. Revert `docs/domain/glossary.md` changes
3. Delete `docs/adr/0015-feature-implementation-diagrams.md`
4. Revert `docs/diagrams/INDEX.md` changes
5. Revert `.claude/hooks/doc-enforcement.sh` changes
6. Revert `commands/implement.md` changes
7. Delete `agents/diagram-updater.md`
8. Delete `agents/module-summary-updater.md`

Immediate mitigation for hook regression: `ECC_WORKFLOW_BYPASS=1`

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | PASS with notes | 1 (SRP — mitigated by existing pattern) |
| Robert | CLEAN | 0 |
| Security | CLEAR | 2 LOW/INFO |

### Adversary Findings

| Dimension | Verdict | Key Rationale |
|-----------|---------|---------------|
| AC Coverage | PASS (R2) | 47 PCs cover all 50 ACs after adding PC-042–047 |
| Execution Order | PASS (R1) | Dependency chain correct: agents → implement.md → hook/docs |
| Fragility | PASS (R2) | Grep-based verification acceptable for markdown-only changes |
| Rollback Adequacy | PASS (R2) | Reverse dependency order + ECC_WORKFLOW_BYPASS mitigation |
| Architecture Compliance | PASS (R1) | No Rust code, no domain changes, pure orchestration layer |
| Blast Radius | PASS (R1) | 8 files, no crate boundaries crossed |
| Missing PCs | PASS (R2) | Hook integration test (PC-047), diagram type coverage (PC-042–044) added |
| Doc Plan | PASS (R2) | CHANGELOG, ADR 0015, glossary all covered |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `agents/module-summary-updater.md` | create | US-002 |
| 2 | `agents/diagram-updater.md` | create | US-003 |
| 3 | `commands/implement.md` | modify | US-001, US-004, US-005 |
| 4 | `.claude/hooks/doc-enforcement.sh` | modify | US-006 |
| 5 | `docs/diagrams/INDEX.md` | modify | US-007 |
| 6 | `docs/adr/0015-feature-implementation-diagrams.md` | create | Decision #4 |
| 7 | `docs/domain/glossary.md` | modify | US-008 |
| 8 | `CHANGELOG.md` | modify | US-008 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-23-context-aware-doc-generation/design.md | Full design |
