# Design: Explanatory Narrative Audit (BL-051)

## Overview

Add "narrate before acting" instructions to all 22 ECC command files and create a shared narrative-conventions skill. Pure Markdown content additions — no logic, no Rust, no agent frontmatter changes. Test via bash grep assertions following the `test-pipeline-summaries.sh` pattern.

## File Changes Table (Dependency Order)

| # | File | Action | Group | Dependencies |
|---|------|--------|-------|--------------|
| 1 | `skills/narrative-conventions/SKILL.md` | Create | 1 | None |
| 2 | `commands/spec-dev.md` | Modify | 2 | #1 |
| 3 | `commands/spec-fix.md` | Modify | 2 | #1 |
| 4 | `commands/spec-refactor.md` | Modify | 2 | #1 |
| 5 | `commands/design.md` | Modify | 3 | #1 |
| 6 | `commands/implement.md` | Modify | 3 | #1 |
| 7 | `commands/audit-full.md` | Modify | 4 | #1 |
| 8 | `commands/audit-archi.md` | Modify | 4 | #1 |
| 9 | `commands/audit-code.md` | Modify | 4 | #1 |
| 10 | `commands/audit-security.md` | Modify | 4 | #1 |
| 11 | `commands/audit-test.md` | Modify | 4 | #1 |
| 12 | `commands/audit-convention.md` | Modify | 4 | #1 |
| 13 | `commands/audit-errors.md` | Modify | 4 | #1 |
| 14 | `commands/audit-observability.md` | Modify | 4 | #1 |
| 15 | `commands/audit-doc.md` | Modify | 4 | #1 |
| 16 | `commands/audit-evolution.md` | Modify | 4 | #1 |
| 17 | `commands/verify.md` | Modify | 4 | #1 |
| 18 | `commands/build-fix.md` | Modify | 4 | #1 |
| 19 | `commands/review.md` | Modify | 4 | #1 |
| 20 | `commands/catchup.md` | Modify | 4 | #1 |
| 21 | `commands/backlog.md` | Modify | 4 | #1 |
| 22 | `commands/spec.md` | Modify | 4 | #1 |
| 23 | `commands/ecc-test-mode.md` | Modify | 4 | #1 |
| 24 | `tests/test-narrative-audit.sh` | Create | 5 | #1-#23 |
| 25 | `docs/adr/0011-command-narrative-convention.md` | Create | 5 | #1-#23 |
| 26 | `CHANGELOG.md` | Modify | 5 | #1-#23 |
| 27 | `docs/narrative-audit.md` | Create | 5 | #1-#23 |

## Narrative Insertion Patterns

Each command gets narrative instructions at specific insertion points. The patterns are:

### Pattern A: Agent Delegation Narration
Insert before every `Launch a Task` or agent invocation line:
> Before dispatching, tell the user which agent is being launched, what it will analyze, and what to expect from its output.

### Pattern B: Gate/Block Narration
Insert at every gate failure path (error messages, STOP points):
> If this gate blocks, explain to the user what failed, why it matters, and provide specific remediation steps.

### Pattern C: Phase Transition Narration
Insert at major phase boundaries:
> Before starting this phase, tell the user what phase is beginning, what it will accomplish, and how it connects to the previous phase.

### Pattern D: Result Narration
Insert after agent results are collected:
> After receiving the agent's output, summarize the key findings conversationally before incorporating them into the structured output.

### Skill Reference
Each command references `skills/narrative-conventions/SKILL.md` near its top (after the MANDATORY WORKFLOW block) with:
> **Narrative**: See `skills/narrative-conventions/SKILL.md` for narration conventions. Before each agent delegation, gate check, and phase transition, narrate what is happening and why.

## Per-Command Narrative Points

### Group 1: Shared Skill

**`skills/narrative-conventions/SKILL.md`** — New file with frontmatter (`name: narrative-conventions`, `description`, `origin: ECC`). Content sections:
1. **Agent Delegation** — before dispatching any agent, tell the user: which agent, what it analyzes, what output to expect
2. **Gate Failure** — if a gate blocks, explain: what blocked, why it matters, specific remediation steps
3. **Progress** — at phase transitions, tell the user: what phase begins, what it accomplishes, what comes next
4. **Tone** — neutral technical, active voice, present tense; instruct what to communicate, never how to word it
5. **Placement** — narrative appears before the action it describes

Must be under 500 words.

### Group 2: Spec Commands (spec-dev, spec-fix, spec-refactor)

Each gets identical narrative additions at these points:
- **Phase 1** (Requirements Analysis): Pattern A before `requirements-analyst` agent dispatch
- **Phase 2** (Architecture Review): Pattern A before `architect` agent dispatch
- **Phase 3** (Web Research): Pattern C at phase start — tell user what queries will run
- **Phase 7/8/9** (Adversarial Review): Pattern D after adversary verdict + Pattern B at FAIL/CONDITIONAL handling
- **Gate failures** (state validation, adversarial FAIL): Pattern B
- **Skill reference** near top of file

Grep markers per file: `narrative-conventions`, `tell the user which agent`, `explain.*what.*blocked`, `remediation`

### Group 3: Design + Implement

**`commands/design.md`**:
- Phase 0 (State Validation): Pattern B at gate failure
- Phase 2 (SOLID Validation): Pattern A before SOLID agent
- Phase 3 (Professional Conscience): Pattern A before Robert agent
- Phase 4 (Security Quick-Check): Pattern A before security agent
- Phase 7 (AC Coverage): Pattern D after coverage result
- Adversarial review: Pattern D + Pattern B
- Skill reference near top

**`commands/implement.md`**:
- Phase 0 (State Validation): Pattern B at gate failure
- TDD Loop (PC dispatch): Pattern A + tell user which PC, what AC it covers, what to expect
- Regression verification: Pattern D — report how many prior PCs re-verified and result
- Code review findings: Pattern D — tell user what was found and fixed
- Gate failures (regression, state): Pattern B
- Skill reference near top

### Group 4: Audits + Utilities

**`commands/audit-full.md`**: Pattern A before orchestrator dispatch, Pattern D for per-domain completion reporting, Pattern C at cross-domain correlation phase

**`commands/audit-*.md`** (9 domain audits): Pattern A before each parallel agent dispatch, Pattern C at report generation phase

**`commands/verify.md`**: Pattern A before code-reviewer and arch-reviewer agents, narrate why both are needed

**`commands/build-fix.md`**: Pattern C at error classification step — explain Structural/Contractual/Incidental before acting

**`commands/review.md`**: Pattern A before robert agent — explain what the Programmer's Oath evaluation means

**`commands/catchup.md`**: Pattern B when stale workflow detected — explain consequences of resetting before offering option

**`commands/backlog.md`**: Pattern C at idea challenge phase

**`commands/spec.md`**: Pattern C at classification — narrate the classification result before delegating

**`commands/ecc-test-mode.md`**: Pattern C — explain what "hooks active" means and what to expect

## Pass Conditions Table

| # | PC | Bash Command | Validates |
|---|-----|-------------|-----------|
| 1 | skill-frontmatter | `bash tests/test-narrative-audit.sh test_skill_frontmatter` | AC-001.1: frontmatter fields |
| 2 | skill-content | `bash tests/test-narrative-audit.sh test_skill_content` | AC-001.2, AC-001.3, AC-001.4: patterns, word count, tone |
| 3 | spec-dev-narrative | `bash tests/test-narrative-audit.sh test_specdev_narrative` | AC-002.1-002.4: agent narration, web research narration, adversarial translation, gate narration |
| 4 | spec-fix-narrative | `bash tests/test-narrative-audit.sh test_specfix_narrative` | AC-002.1-002.5: consistency + web research + adversarial translation |
| 5 | spec-refactor-narrative | `bash tests/test-narrative-audit.sh test_specrefactor_narrative` | AC-002.1-002.5: consistency + web research + adversarial translation |
| 6 | design-narrative | `bash tests/test-narrative-audit.sh test_design_narrative` | AC-003.1, AC-003.2, AC-003.6 |
| 7 | implement-narrative | `bash tests/test-narrative-audit.sh test_implement_narrative` | AC-003.3-003.6 |
| 8 | audit-full-narrative | `bash tests/test-narrative-audit.sh test_audit_full_narrative` | AC-004.1, AC-004.2 |
| 9 | audit-domain-narrative | `bash tests/test-narrative-audit.sh test_audit_domain_narrative` | AC-004.1, AC-004.3: all 9 domain audits + act-on-findings guidance |
| 10 | verify-narrative | `bash tests/test-narrative-audit.sh test_verify_narrative` | AC-004.4 |
| 11 | build-fix-narrative | `bash tests/test-narrative-audit.sh test_buildfix_narrative` | AC-004.5 |
| 12 | review-narrative | `bash tests/test-narrative-audit.sh test_review_narrative` | AC-004.6 |
| 13 | catchup-narrative | `bash tests/test-narrative-audit.sh test_catchup_narrative` | AC-004.7 |
| 14 | utility-narrative | `bash tests/test-narrative-audit.sh test_utility_narrative` | backlog, spec, ecc-test-mode |
| 15 | adr-0011 | `bash tests/test-narrative-audit.sh test_adr_0011` | AC-005.1 |
| 16 | changelog | `bash tests/test-narrative-audit.sh test_changelog_bl051` | AC-005.2 |
| 17 | audit-doc | `bash tests/test-narrative-audit.sh test_audit_doc` | AC-005.3 |
| 18 | line-counts | `bash tests/test-narrative-audit.sh test_line_counts` | All files under 800 lines |
| 19 | skill-ref-consistency | `bash tests/test-narrative-audit.sh test_skill_ref_consistency` | All 22 commands reference the skill |
| 20 | full-suite | `bash tests/test-narrative-audit.sh` | All tests pass end-to-end |
| 21 | cargo-clippy | `cargo clippy -- -D warnings` | No Rust regressions |
| 22 | cargo-build | `cargo build` | Build passes |
| 23 | cargo-test | `cargo test` | All 1185 tests pass |
| 24 | markdown-lint | `npm run lint` | Markdown lint passes |

**Total: 24 PCs**

## TDD Implementation Order

### Phase 1: Test Scaffold + Shared Skill (PCs 1-2)
**Layers**: Content (skill)

1. **RED**: Create `tests/test-narrative-audit.sh` with `test_skill_frontmatter` and `test_skill_content` functions. Run — both fail (file does not exist).
2. **GREEN**: Create `skills/narrative-conventions/SKILL.md` with frontmatter and all pattern sections. Run PCs 1-2 — pass.
3. **REFACTOR**: Review skill for clarity and word count.
4. **Commit**: `test: add narrative audit test scaffold and skill tests` then `feat: create narrative-conventions skill`

### Phase 2: Spec Trio Narrative (PCs 3-5)
**Layers**: Content (command)

1. **RED**: Add `test_specdev_narrative`, `test_specfix_narrative`, `test_specrefactor_narrative` to test file. Run — all fail.
2. **GREEN**: Add narrative instructions + skill reference to `commands/spec-dev.md`, `commands/spec-fix.md`, `commands/spec-refactor.md`. Run PCs 3-5 — pass.
3. **REFACTOR**: Verify consistency across all three files.
4. **Commit**: `test: add spec command narrative tests` then `feat: add narrative to spec-dev, spec-fix, spec-refactor`

### Phase 3: Design + Implement Narrative (PCs 6-7)
**Layers**: Content (command)

1. **RED**: Add `test_design_narrative`, `test_implement_narrative` to test file. Run — both fail.
2. **GREEN**: Add narrative instructions + skill reference to `commands/design.md`, `commands/implement.md`. Run PCs 6-7 — pass.
3. **REFACTOR**: Verify consistency with Phase 2 patterns.
4. **Commit**: `test: add design and implement narrative tests` then `feat: add narrative to design and implement commands`

### Phase 4: Audit + Utility Narrative (PCs 8-14, 18-19)
**Layers**: Content (command)

1. **RED**: Add `test_audit_full_narrative`, `test_audit_domain_narrative`, `test_verify_narrative`, `test_buildfix_narrative`, `test_review_narrative`, `test_catchup_narrative`, `test_utility_narrative`, `test_line_counts`, `test_skill_ref_consistency` to test file. Run — all fail.
2. **GREEN**: Add narrative instructions + skill reference to all 16 remaining command files. Run PCs 8-14, 18-19 — pass.
3. **REFACTOR**: Spot check tone/voice consistency across all commands.
4. **Commit**: `test: add audit and utility narrative tests` then `feat: add narrative to audit and utility commands`

### Phase 5: Documentation (PCs 15-17)
**Layers**: Content (documentation)

1. **RED**: Add `test_adr_0011`, `test_changelog_bl051`, `test_audit_doc` to test file. Run — all fail.
2. **GREEN**: Create `docs/adr/0011-command-narrative-convention.md`, add CHANGELOG entry, create `docs/narrative-audit.md`. Run PCs 15-17 — pass.
3. **REFACTOR**: Review ADR for completeness.
4. **Commit**: `test: add documentation tests for BL-051` then `docs: add ADR 0011, changelog, and narrative audit summary`

### Phase 6: Full Suite Validation (PCs 20-24)
**Layers**: None (validation only)

1. Run `bash tests/test-narrative-audit.sh` — full suite must pass (PC 20)
2. Run `cargo clippy -- -D warnings` (PC 21)
3. Run `cargo build` (PC 22)
4. Run `cargo test` (PC 23)
5. Run `npm run lint` (PC 24)
6. No commits — validation gate only.

## Test Design: `tests/test-narrative-audit.sh`

### Structure
Follows `tests/test-pipeline-summaries.sh` exactly: same helpers (`assert_file_contains`, `assert_file_not_contains`), same runner pattern (single test name as `$1` or all tests), same PASS/FAIL counters.

### Key Grep Patterns per Test Function

**`test_skill_frontmatter`**: Check `name: narrative-conventions`, `description:`, `origin: ECC`. Assert NOT contains `model:` or `tools:`.

**`test_skill_content`**: Check contains "agent delegation" (case-insensitive), "gate failure" (case-insensitive), "progress" (case-insensitive), "active voice", "before the action". Word count check: `wc -w < "$SKILL_FILE"` is under 500.

**`test_specdev_narrative`**: Check `spec-dev.md` contains `narrative-conventions`, `tell the user which agent`, `remediation`. Same for spec-fix and spec-refactor.

**`test_design_narrative`**: Check `design.md` contains `narrative-conventions`, `tell the user which validation`, `remediation`, `coverage result`.

**`test_implement_narrative`**: Check `implement.md` contains `narrative-conventions`, `tell the user.*PC`, `re-verified`, `remediation`, `what was found`.

**`test_audit_full_narrative`**: Check `audit-full.md` contains `narrative-conventions`, `tell the user which domain`, `completion status`.

**`test_audit_domain_narrative`**: Loop over all 9 `audit-*.md` (excluding audit-full), assert each contains `narrative-conventions` and `tell the user`.

**`test_verify_narrative`**: Check `verify.md` contains `narrative-conventions`, `tell the user.*reviewer`, `why both`.

**`test_buildfix_narrative`**: Check `build-fix.md` contains `narrative-conventions`, `explain the classification`.

**`test_review_narrative`**: Check `review.md` contains `narrative-conventions`, `Programmer.*Oath`.

**`test_catchup_narrative`**: Check `catchup.md` contains `narrative-conventions`, `consequences.*resetting`.

**`test_utility_narrative`**: Check `backlog.md`, `spec.md`, `ecc-test-mode.md` each contain `narrative-conventions`.

**`test_line_counts`**: Loop over all 22 command files + skill file, assert each under 800 lines.

**`test_skill_ref_consistency`**: Loop over all 22 command files, assert each contains `narrative-conventions`.

**`test_adr_0011`**: Assert file exists, contains `Status`, `Context`, `Decision`, `Consequences`.

**`test_changelog_bl051`**: Assert `CHANGELOG.md` contains `BL-051` or `narrative`.

**`test_audit_doc`**: Assert `docs/narrative-audit.md` exists, contains at least 5 command names.

## Risks and Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| Command files exceed 800 lines after additions | Medium | Narrative adds ~5-15 lines per file; current longest is ~310 lines; well within limit |
| Grep patterns too brittle | Low | Use case-insensitive matching and partial phrases rather than exact sentences |
| Inconsistent narration tone across commands | Medium | Shared skill defines patterns; test_skill_ref_consistency enforces all commands reference it |
| Existing narration in some commands creates duplication | Low | Spec says "augment existing, not rewrite"; only add where missing |

## Boy Scout Delta Candidates

- `commands/build-fix.md` line 3: missing `allowed-tools` in frontmatter (unlike all other commands)
- `commands/review.md` line 3: missing `allowed-tools` in frontmatter
- Any dead TODOs found during editing

## Success Criteria

- [ ] `skills/narrative-conventions/SKILL.md` exists with correct frontmatter and under 500 words
- [ ] All 22 command files reference `narrative-conventions` skill
- [ ] All 22 command files have narrative before agent delegations and gate failures
- [ ] ADR 0011 documents the convention
- [ ] CHANGELOG has BL-051 entry
- [ ] `docs/narrative-audit.md` lists all commands and narrative points
- [ ] All 24 PCs pass
- [ ] All command files remain under 800 lines
- [ ] No Rust regressions (clippy, build, test all pass)
