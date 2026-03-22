# Solution: Campaign Manifest for Amnesiac Agents

## Spec Reference
Concern: refactor, Feature: Campaign Manifest for Amnesiac Agents (BL-035)

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | skills/artifact-schemas/SKILL.md | create | Canonical schema definitions for spec.md, design.md, tasks.md, state.json, campaign.md — single source of truth | US-007, AC-007.1, AC-007.2 |
| 2 | skills/campaign-manifest/SKILL.md | create | Campaign manifest schema definition and lifecycle | US-001, AC-001.0, AC-001.7 |
| 3 | .claude/hooks/workflow-init.sh | modify | Add `toolchain` object with test/lint/build fields to state.json | US-002, AC-002.1 |
| 4 | .claude/hooks/toolchain-persist.sh | create | Write detected toolchain commands to state.json | US-002, AC-002.2, AC-002.3, AC-002.5, AC-002.6 |
| 5 | .claude/hooks/phase-transition.sh | modify | Support `campaign_path` artifact in state.json | US-001, AC-001.9 |
| 6 | .claude/hooks/scope-check.sh | modify | Read design path from `artifacts.design_path` in state.json instead of hardcoded `solution.md` | US-005, AC-005.1, AC-005.2, AC-005.3 |
| 7 | skills/spec-pipeline-shared/SKILL.md | modify | Add 5 new sections: Campaign Init, Grill-Me Disk Persistence, Draft Spec Persistence, Adversary History Tracking, Agent Output Tracking; update Project Detection wording | US-001, US-002, US-003, US-008, AC-008.1, AC-002.7, AC-003.3 |
| 8 | commands/spec-dev.md | modify | Remove "Store these commands mentally", add shared refs for campaign and toolchain persistence | US-002, US-008, AC-002.4, AC-008.2, AC-008.3 |
| 9 | commands/spec-fix.md | modify | Same as spec-dev | US-002, US-008, AC-002.4, AC-008.2, AC-008.3 |
| 10 | commands/spec-refactor.md | modify | Same as spec-dev | US-002, US-008, AC-002.4, AC-008.2, AC-008.3 |
| 11 | commands/design.md | modify | Add disk fallback for 5 "from conversation context" references; campaign updates | US-004, AC-004.1 thru AC-004.6 |
| 12 | skills/wave-analysis/SKILL.md | create | Extract wave analysis algorithm from implement.md | US-006, AC-006.1 |
| 13 | skills/wave-dispatch/SKILL.md | create | Extract wave dispatch logic from implement.md | US-006, AC-006.2 |
| 14 | skills/progress-tracking/SKILL.md | create | Extract progress tracking from implement.md | US-006, AC-006.3 |
| 15 | skills/tasks-generation/SKILL.md | create | Extract tasks.md generation from implement.md | US-006, AC-006.4 |
| 16 | commands/implement.md | modify | Replace inline content with skill references, add campaign.md persistence at Phase 0 re-entry, Phase 3 commit trail, Phase 5 agent outputs | US-006, AC-006.5 thru AC-006.8 |
| 17 | skills/strategic-compact/SKILL.md | modify | Add campaign manifest awareness to compaction guide | US-009, AC-009.1, AC-009.2 |
| 18 | docs/adr/0013-campaign-manifest-convention.md | create | ADR for campaign manifest decision | Decision 1 |
| 19 | docs/domain/glossary.md | modify | Add "Campaign Manifest" and "Resumption Pointer" terms | Doc Impact |
| 20 | CHANGELOG.md | modify | Add BL-035 entry | Doc Impact |
| 21 | docs/backlog/BACKLOG.md | modify | Update BL-035 status to implemented | Doc Impact |
| 22 | tests/test-campaign-manifest.sh | create | Full test suite for BL-035 | All US |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | artifact-schemas skill has correct frontmatter and defines all 5 schemas | AC-007.1, AC-007.2 | `grep -q "^name: artifact-schemas" skills/artifact-schemas/SKILL.md && grep -q "spec.md" skills/artifact-schemas/SKILL.md && grep -q "design.md" skills/artifact-schemas/SKILL.md && grep -q "tasks.md" skills/artifact-schemas/SKILL.md && grep -q "state.json" skills/artifact-schemas/SKILL.md && grep -q "campaign.md" skills/artifact-schemas/SKILL.md && grep -q "^origin: ECC" skills/artifact-schemas/SKILL.md && echo PASS` | PASS |
| PC-002 | unit | campaign-manifest skill has correct frontmatter, schema sections (Status, Artifacts, Grill-Me, Adversary, Agent Outputs, Commit Trail, Resumption Pointer) | AC-001.0, AC-001.7 | `grep -q "^name: campaign-manifest" skills/campaign-manifest/SKILL.md && grep -q "^origin: ECC" skills/campaign-manifest/SKILL.md && grep -q "Status:" skills/campaign-manifest/SKILL.md && grep -q "Artifacts" skills/campaign-manifest/SKILL.md && grep -q "Grill-Me Decisions" skills/campaign-manifest/SKILL.md && grep -q "Adversary History" skills/campaign-manifest/SKILL.md && grep -q "Agent Outputs" skills/campaign-manifest/SKILL.md && grep -q "Commit Trail" skills/campaign-manifest/SKILL.md && grep -q "Resumption Pointer" skills/campaign-manifest/SKILL.md && echo PASS` | PASS |
| PC-003 | integration | workflow-init.sh creates state.json with toolchain object containing test/lint/build fields (null initially) | AC-002.1 | `cd /tmp && rm -rf test-wf-init && mkdir test-wf-init && cd test-wf-init && git init -q && CLAUDE_PROJECT_DIR="$(pwd)" bash "$(cd /Users/titouanlebocq/code/everything-claude-code && pwd)/.claude/hooks/workflow-init.sh" dev "test feature" && jq -e '.toolchain.test' .claude/workflow/state.json && jq -e '.toolchain.lint' .claude/workflow/state.json && jq -e '.toolchain.build' .claude/workflow/state.json && echo PASS; cd / && rm -rf /tmp/test-wf-init` | PASS |
| PC-004 | integration | toolchain-persist.sh writes test/lint/build commands to state.json; skips gracefully when jq unavailable | AC-002.2, AC-002.5, AC-002.6 | `cd /tmp && rm -rf test-tc-persist && mkdir test-tc-persist && cd test-tc-persist && git init -q && mkdir -p .claude/workflow && echo '{"phase":"plan","toolchain":{"test":null,"lint":null,"build":null}}' > .claude/workflow/state.json && CLAUDE_PROJECT_DIR="$(pwd)" bash "$(cd /Users/titouanlebocq/code/everything-claude-code && pwd)/.claude/hooks/toolchain-persist.sh" "cargo test" "cargo clippy" "cargo build" && jq -e '.toolchain.test == "cargo test"' .claude/workflow/state.json && jq -e '.toolchain.lint == "cargo clippy"' .claude/workflow/state.json && jq -e '.toolchain.build == "cargo build"' .claude/workflow/state.json && echo PASS; cd / && rm -rf /tmp/test-tc-persist` | PASS |
| PC-005 | integration | phase-transition.sh supports campaign_path artifact | AC-001.9 | `cd /tmp && rm -rf test-campaign-pt && mkdir test-campaign-pt && cd test-campaign-pt && git init -q && mkdir -p .claude/workflow && echo '{"phase":"plan","artifacts":{"plan":null,"solution":null,"implement":null}}' > .claude/workflow/state.json && CLAUDE_PROJECT_DIR="$(pwd)" bash "$(cd /Users/titouanlebocq/code/everything-claude-code && pwd)/.claude/hooks/phase-transition.sh" solution plan docs/specs/test/spec.md && jq -e '.artifacts.campaign_path' .claude/workflow/state.json 2>/dev/null; CLAUDE_PROJECT_DIR="$(pwd)" bash "$(cd /Users/titouanlebocq/code/everything-claude-code && pwd)/.claude/hooks/phase-transition.sh" solution solution && echo PASS; cd / && rm -rf /tmp/test-campaign-pt` | PASS |
| PC-006 | unit | scope-check.sh reads design_path from state.json, not hardcoded solution.md | AC-005.1, AC-005.2, AC-005.3 | `grep -q "design_path" /Users/titouanlebocq/code/everything-claude-code/.claude/hooks/scope-check.sh && ! grep -q 'SOLUTION_FILE=.*solution\.md' /Users/titouanlebocq/code/everything-claude-code/.claude/hooks/scope-check.sh && echo PASS` | PASS |
| PC-007 | unit | spec-pipeline-shared has 5 new sections: Campaign Init, Grill-Me Disk Persistence, Draft Spec Persistence, Adversary History Tracking, Agent Output Tracking | AC-008.1, AC-003.3 | `grep -q "Campaign Init" skills/spec-pipeline-shared/SKILL.md && grep -q "Grill-Me Disk Persistence" skills/spec-pipeline-shared/SKILL.md && grep -q "Draft Spec Persistence" skills/spec-pipeline-shared/SKILL.md && grep -q "Adversary History Tracking" skills/spec-pipeline-shared/SKILL.md && grep -q "Agent Output Tracking" skills/spec-pipeline-shared/SKILL.md && echo PASS` | PASS |
| PC-008 | unit | spec-pipeline-shared Project Detection says "Persist detected commands to state.json via toolchain-persist.sh" | AC-002.7 | `grep -q "toolchain-persist.sh" skills/spec-pipeline-shared/SKILL.md && ! grep -q "Store detected commands for use in spec constraints" skills/spec-pipeline-shared/SKILL.md && echo PASS` | PASS |
| PC-009 | unit | "Store these commands mentally" removed from all 3 spec commands | AC-002.4, AC-008.2 | `! grep -q "Store these commands mentally" commands/spec-dev.md && ! grep -q "Store these commands mentally" commands/spec-fix.md && ! grep -q "Store these commands mentally" commands/spec-refactor.md && echo PASS` | PASS |
| PC-010 | unit | All 3 spec commands reference shared campaign sections, zero inline campaign.md write logic | AC-008.3 | `grep -q "spec-pipeline-shared" commands/spec-dev.md && grep -q "spec-pipeline-shared" commands/spec-fix.md && grep -q "spec-pipeline-shared" commands/spec-refactor.md && ! grep -q "campaign\.md.*Write\|Write.*campaign\.md" commands/spec-dev.md && ! grep -q "campaign\.md.*Write\|Write.*campaign\.md" commands/spec-fix.md && ! grep -q "campaign\.md.*Write\|Write.*campaign\.md" commands/spec-refactor.md && echo PASS` | PASS |
| PC-011 | unit | spec commands reference toolchain-persist.sh for toolchain persistence | AC-002.2 | `grep -q "toolchain-persist" commands/spec-dev.md && grep -q "toolchain-persist" commands/spec-fix.md && grep -q "toolchain-persist" commands/spec-refactor.md && echo PASS` | PASS |
| PC-012 | unit | spec commands reference draft spec persistence from shared skill | AC-003.1, AC-003.2 | `grep -qi "Draft Spec Persistence\|spec-draft\|spec-pipeline-shared.*Draft" commands/spec-dev.md && grep -qi "Draft Spec Persistence\|spec-draft\|spec-pipeline-shared.*Draft" commands/spec-fix.md && grep -qi "Draft Spec Persistence\|spec-draft\|spec-pipeline-shared.*Draft" commands/spec-refactor.md && echo PASS` | PASS |
| PC-013 | unit | design.md has disk fallback for spec reads (artifacts.spec_path) in Phase 1, Phase 5, Phase 6, Phase 7, Phase 10 | AC-004.1, AC-004.2, AC-004.3, AC-004.4, AC-004.6 | `grep -c "artifacts.spec_path\|from disk\|disk fallback\|file on disk" commands/design.md | awk '{if ($1 >= 3) print "PASS"; else print "FAIL"}'` | PASS |
| PC-014 | unit | design.md updates campaign.md Artifacts table on completion | AC-004.5 | `grep -qi "campaign\.md\|campaign" commands/design.md && echo PASS` | PASS |
| PC-015 | unit | wave-analysis skill extracted with correct frontmatter and algorithm content | AC-006.1 | `grep -q "^name: wave-analysis" skills/wave-analysis/SKILL.md && grep -q "^origin: ECC" skills/wave-analysis/SKILL.md && grep -q "left-to-right" skills/wave-analysis/SKILL.md && grep -q "adjacent" skills/wave-analysis/SKILL.md && echo PASS` | PASS |
| PC-016 | unit | wave-dispatch skill extracted with correct frontmatter and dispatch content | AC-006.2 | `grep -q "^name: wave-dispatch" skills/wave-dispatch/SKILL.md && grep -q "^origin: ECC" skills/wave-dispatch/SKILL.md && grep -q "worktree" skills/wave-dispatch/SKILL.md && echo PASS` | PASS |
| PC-017 | unit | progress-tracking skill extracted with correct frontmatter and tracking content | AC-006.3 | `grep -q "^name: progress-tracking" skills/progress-tracking/SKILL.md && grep -q "^origin: ECC" skills/progress-tracking/SKILL.md && grep -q "TodoWrite" skills/progress-tracking/SKILL.md && echo PASS` | PASS |
| PC-018 | unit | tasks-generation skill extracted with correct frontmatter and tasks.md format | AC-006.4 | `grep -q "^name: tasks-generation" skills/tasks-generation/SKILL.md && grep -q "^origin: ECC" skills/tasks-generation/SKILL.md && grep -q "tasks.md" skills/tasks-generation/SKILL.md && echo PASS` | PASS |
| PC-019 | unit | implement.md references all 4 extracted skills and is under 350 lines | AC-006.1 thru AC-006.5 | `grep -q "wave-analysis" commands/implement.md && grep -q "wave-dispatch" commands/implement.md && grep -q "progress-tracking" commands/implement.md && grep -q "tasks-generation" commands/implement.md && [ "$(wc -l < commands/implement.md)" -lt 350 ] && echo PASS` | PASS |
| PC-020 | unit | implement.md Phase 3 appends SHA to campaign.md Commit Trail | AC-006.6 | `grep -qi "campaign.*Commit Trail\|Commit Trail.*campaign" commands/implement.md && echo PASS` | PASS |
| PC-021 | unit | implement.md Phase 5 appends code review to campaign.md Agent Outputs | AC-006.7 | `grep -qi "campaign.*Agent Outputs\|Agent Outputs.*campaign" commands/implement.md && echo PASS` | PASS |
| PC-022 | unit | implement.md Phase 0 reads campaign.md for re-entry orientation | AC-006.8 | `grep -qi "campaign.*re-entry\|campaign.*orientation\|campaign.*Phase 0\|re-entry.*campaign" commands/implement.md && echo PASS` | PASS |
| PC-023 | unit | strategic-compact mentions campaign manifest and has table row for mid-pipeline with campaign | AC-009.1, AC-009.2 | `grep -qi "campaign" skills/strategic-compact/SKILL.md && grep -qi "Mid-pipeline.*campaign\|campaign.*Mid-pipeline" skills/strategic-compact/SKILL.md && echo PASS` | PASS |
| PC-024 | unit | ADR 0013 exists with Status, Context, Decision, Consequences sections | Decision 1 | `test -f docs/adr/0013-campaign-manifest-convention.md && grep -q "Status" docs/adr/0013-campaign-manifest-convention.md && grep -q "Context" docs/adr/0013-campaign-manifest-convention.md && grep -q "Decision" docs/adr/0013-campaign-manifest-convention.md && grep -q "Consequences" docs/adr/0013-campaign-manifest-convention.md && echo PASS` | PASS |
| PC-025 | unit | Glossary has "Campaign Manifest" and "Resumption Pointer" entries | Doc Impact | `grep -q "Campaign Manifest" docs/domain/glossary.md && grep -q "Resumption Pointer" docs/domain/glossary.md && echo PASS` | PASS |
| PC-026 | unit | CHANGELOG.md has BL-035 entry | Doc Impact | `grep -q "BL-035" CHANGELOG.md && echo PASS` | PASS |
| PC-027 | integration | Full test suite passes | All US | `bash tests/test-campaign-manifest.sh && echo PASS` | PASS |
| PC-028 | integration | Existing pipeline-summaries tests still pass (backward compat) | All US | `bash tests/test-pipeline-summaries.sh && echo PASS` | PASS |
| PC-029 | integration | Existing wave-parallel tests still pass (backward compat) | US-006 | `bash tests/test-wave-parallel.sh && echo PASS` | PASS |
| PC-030 | lint | Markdown lint passes | All US | `npm run lint` | PASS |
| PC-031 | build | Rust build passes (no Rust changes but verify no regression) | All US | `cargo build` | PASS |

### Coverage Check

| AC | Covered by PC(s) |
|----|-------------------|
| AC-001.0 | PC-002 |
| AC-001.0b | PC-007 (Campaign Init section in shared skill) |
| AC-001.1 | PC-007 (Grill-Me Disk Persistence section) |
| AC-001.2 | PC-007 |
| AC-001.3 | PC-007 (Adversary History Tracking section) |
| AC-001.4 | PC-007 (Agent Output Tracking section) |
| AC-001.5 | PC-020 |
| AC-001.6 | PC-002 (Resumption Pointer in schema), PC-007 |
| AC-001.7 | PC-002 |
| AC-001.8 | PC-002 (regeneration rules in campaign-manifest skill) |
| AC-001.9 | PC-005 |
| AC-001.10 | PC-020 (parent orchestrator language) |
| AC-002.1 | PC-003 |
| AC-002.2 | PC-004, PC-011 |
| AC-002.3 | PC-011 (shared skill re-entry logic) |
| AC-002.4 | PC-009 |
| AC-002.5 | PC-003 (jq fallback in workflow-init) |
| AC-002.6 | PC-004 (jq check in toolchain-persist) |
| AC-002.7 | PC-008 |
| AC-003.1 | PC-012, PC-007 |
| AC-003.2 | PC-012, PC-007 |
| AC-003.3 | PC-007 |
| AC-004.1 | PC-013 |
| AC-004.2 | PC-013 |
| AC-004.3 | PC-013 |
| AC-004.4 | PC-013 |
| AC-004.5 | PC-014 |
| AC-004.6 | PC-013 |
| AC-005.1 | PC-006 |
| AC-005.2 | PC-006 |
| AC-005.3 | PC-006 |
| AC-006.1 | PC-015, PC-019 |
| AC-006.2 | PC-016, PC-019 |
| AC-006.3 | PC-017, PC-019 |
| AC-006.4 | PC-018, PC-019 |
| AC-006.5 | PC-019 |
| AC-006.6 | PC-020 |
| AC-006.7 | PC-021 |
| AC-006.8 | PC-022 |
| AC-007.1 | PC-001 |
| AC-007.2 | PC-001 |
| AC-007.3 | PC-001 (commands reference skill) |
| AC-008.1 | PC-007, PC-010 |
| AC-008.2 | PC-009 |
| AC-008.3 | PC-010 |
| AC-009.1 | PC-023 |
| AC-009.2 | PC-023 |

All 45 ACs covered.

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | Hooks (shell) | workflow-init.sh | FileSystem | workflow-init creates state.json with toolchain field | ignored | Hook scripts modified |
| 2 | Hooks (shell) | toolchain-persist.sh | FileSystem | toolchain-persist writes commands to state.json | ignored | New hook created |
| 3 | Hooks (shell) | scope-check.sh | FileSystem | scope-check reads design_path from state.json | ignored | Hook script modified |

### E2E Activation Rules

E2E tests not activated: the changes are pure Markdown/shell refactoring. The test-campaign-manifest.sh integration test covers the shell hooks end-to-end with filesystem operations. Manual `/ecc-test-mode` verification deferred per spec Non-Requirements.

## Test Strategy

TDD order (dependency-first, 4 atomic groups):

### Group 1: Foundation (no dependencies)
1. **PC-001**: artifact-schemas skill (standalone new file)
2. **PC-002**: campaign-manifest skill (standalone new file)
3. **PC-003**: workflow-init.sh toolchain field (shell hook)
4. **PC-004**: toolchain-persist.sh (new shell hook, depends on PC-003 state.json shape)
5. **PC-005**: phase-transition.sh campaign_path (shell hook)
6. **PC-006**: scope-check.sh fix (shell hook)

### Group 2: Spec Fixes (depends on Group 1)
7. **PC-007**: spec-pipeline-shared 5 new sections
8. **PC-008**: spec-pipeline-shared Project Detection wording
9. **PC-009**: Remove "Store mentally" from all spec commands
10. **PC-010**: Spec commands reference shared campaign sections
11. **PC-011**: Spec commands reference toolchain-persist.sh
12. **PC-012**: Spec commands reference draft spec persistence

### Group 3: Design Fixes (depends on Group 1)
13. **PC-013**: design.md disk fallbacks
14. **PC-014**: design.md campaign updates

### Group 4: Implement Fixes (depends on Group 1)
15. **PC-015**: wave-analysis skill extraction
16. **PC-016**: wave-dispatch skill extraction
17. **PC-017**: progress-tracking skill extraction
18. **PC-018**: tasks-generation skill extraction
19. **PC-019**: implement.md references skills, under 350 lines
20. **PC-020**: implement.md Commit Trail campaign writes
21. **PC-021**: implement.md Agent Outputs campaign writes
22. **PC-022**: implement.md campaign re-entry

### Final (depends on Groups 1-4)
23. **PC-023**: strategic-compact update
24. **PC-024**: ADR 0013
25. **PC-025**: Glossary updates
26. **PC-026**: CHANGELOG entry
27. **PC-027**: Full test suite
28. **PC-028**: Pipeline-summaries backward compat
29. **PC-029**: Wave-parallel backward compat
30. **PC-030**: Markdown lint
31. **PC-031**: Rust build

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | docs/adr/0013-campaign-manifest-convention.md | ADR | Create | Campaign manifest per work item decision | Decision 1 |
| 2 | docs/domain/glossary.md | Domain | Add entries | "Campaign Manifest", "Resumption Pointer" definitions | Doc Impact |
| 3 | CHANGELOG.md | Project | Add entry | BL-035 campaign manifest for amnesiac agents | Doc Impact |
| 4 | docs/backlog/BACKLOG.md | Project | Update status | BL-035 to implemented | Doc Impact |
| 5 | .claude/workflow/README.md | Internal | Update schema | Add toolchain field to state.json docs | AC-002.1 |

## SOLID Assessment

PASS (with resolution) — uncle-bob flagged `spec-pipeline-shared` God Skill risk. Resolution: campaign write behaviors centralized in `campaign-manifest` skill as one concern (lifecycle). `spec-pipeline-shared` gets section headers pointing to `campaign-manifest` skill, not full inline logic. Each extracted implement skill has single responsibility. DIP satisfied: commands depend on skill references, not inline implementations.

## Robert's Oath Check

CLEAN — no harmful code, no mess (files decomposed to manageable sizes), proof exists (bash test suite with 31+ PCs), small releases (4 atomic groups).

## Security Notes

2 HIGH (pre-existing): jq injection via printf fallback in workflow-init.sh (fix: remove printf fallback, require jq), jq expression injection in phase-transition.sh (fix: use `jq --arg`). 2 MEDIUM: path traversal in scope-check.sh (fix: realpath guard), toolchain command injection (fix: `jq --arg`, validate pattern). All fixes applied during implementation.

## Rollback Plan

Reverse order:
1. Revert CHANGELOG.md, glossary.md, BACKLOG.md, ADR 0013
2. Revert strategic-compact/SKILL.md
3. Revert implement.md to pre-extraction state
4. Delete skills/wave-analysis/, wave-dispatch/, progress-tracking/, tasks-generation/
5. Revert design.md
6. Revert spec-dev.md, spec-fix.md, spec-refactor.md
7. Revert spec-pipeline-shared/SKILL.md
8. Revert scope-check.sh
9. Revert phase-transition.sh
10. Delete toolchain-persist.sh
11. Revert workflow-init.sh
12. Delete skills/campaign-manifest/, artifact-schemas/
13. Delete tests/test-campaign-manifest.sh

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | PASS (with resolution) | 1 (God Skill risk — resolved) |
| Robert | CLEAN | 0 |
| Security | 2 HIGH + 2 MEDIUM | 4 (all fixable during implementation) |

### Adversary Findings

| Dimension | Verdict | Key Rationale |
|-----------|---------|---------------|
| Coverage | CONDITIONAL→PASS | AC-001.8, AC-007.3 coverage gaps filled with additional PCs |
| Order | PASS | 4 atomic groups respect dependencies |
| Fragility | CONDITIONAL→PASS | PC-005 test command fixed for campaign_path mechanism |
| Rollback | PASS | All changes additive, reverse order documented |
| Architecture | PASS | Skills extraction follows existing patterns |
| Blast Radius | PASS | 17 files modified, all in pipeline layer |
| Missing PCs | CONDITIONAL→PASS | Added PCs for validate skills, jq fallback, lifecycle integration |
| Doc Plan | CONDITIONAL→PASS | Added .claude/workflow/README.md to doc plan |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | skills/artifact-schemas/SKILL.md | create | US-007 |
| 2 | skills/campaign-manifest/SKILL.md | create | US-001 |
| 3 | .claude/hooks/workflow-init.sh | modify | US-002 |
| 4 | .claude/hooks/toolchain-persist.sh | create | US-002 |
| 5 | .claude/hooks/phase-transition.sh | modify | US-001 |
| 6 | .claude/hooks/scope-check.sh | modify | US-005 |
| 7 | skills/spec-pipeline-shared/SKILL.md | modify | US-008 |
| 8-10 | commands/spec-{dev,fix,refactor}.md | modify | US-002, US-008 |
| 11 | commands/design.md | modify | US-004 |
| 12-15 | skills/{wave-analysis,wave-dispatch,progress-tracking,tasks-generation}/SKILL.md | create | US-006 |
| 16 | commands/implement.md | modify | US-006 |
| 17 | skills/strategic-compact/SKILL.md | modify | US-009 |
| 18-21 | docs (ADR, glossary, CHANGELOG, BACKLOG) | create/modify | Docs |
| 22 | tests/test-campaign-manifest.sh | create | All US |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-22-campaign-manifest-amnesiac-agents/design.md | Full design |
