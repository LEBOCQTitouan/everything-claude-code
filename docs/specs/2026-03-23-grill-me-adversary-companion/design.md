# Solution: Grill-Me-Adversary Companion Skill with Adaptive Loop (BL-057)

## Spec Reference
Concern: dev, Feature: Create grill-me-adversary companion skill with adaptive loop (BL-057)

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `skills/grill-me-adversary/SKILL.md` | create | New companion skill with adversarial questioning, rubric, loop exit conditions | US-001–006 |
| 2 | `skills/grill-me/SKILL.md` | modify | Add 5-line "Adversary Mode" opt-in section | US-007 |
| 3 | `docs/domain/glossary.md` | modify | Add "Adversary Mode" entry | US-009, AC-009.1 |
| 4 | `CHANGELOG.md` | modify | Add BL-057 entry | US-009, AC-009.2 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | integration | Skill file exists | AC-001.1 | `test -f skills/grill-me-adversary/SKILL.md && echo PASS \|\| echo FAIL` | PASS |
| PC-002 | integration | name: grill-me-adversary | AC-001.2, AC-001.4 | `grep -c '^name: grill-me-adversary$' skills/grill-me-adversary/SKILL.md` | 1 |
| PC-003 | integration | Non-empty description | AC-001.2 | `grep -qP '^description: .+' skills/grill-me-adversary/SKILL.md && echo PASS \|\| echo FAIL` | PASS |
| PC-004 | integration | origin: ECC | AC-001.2 | `grep -c '^origin: ECC$' skills/grill-me-adversary/SKILL.md` | 1 |
| PC-005 | integration | No model field | AC-001.3 | `grep -c '^model:' skills/grill-me-adversary/SKILL.md \|\| echo 0` | 0 |
| PC-006 | integration | No tools field | AC-001.3 | `grep -c '^tools:' skills/grill-me-adversary/SKILL.md \|\| echo 0` | 0 |
| PC-007 | integration | Weakness heuristics documented | AC-002.1, AC-002.2 | `grep -q 'lowest.scored' skills/grill-me-adversary/SKILL.md && grep -q 'hedging' skills/grill-me-adversary/SKILL.md && grep -q 'viability' skills/grill-me-adversary/SKILL.md && echo PASS \|\| echo FAIL` | PASS |
| PC-008 | integration | Avoid repeated angles | AC-002.3 | `grep -qi 'already.*pushed\|avoid.*repeated\|already.*covered' skills/grill-me-adversary/SKILL.md && echo PASS \|\| echo FAIL` | PASS |
| PC-009 | integration | Question challenge mechanism | AC-003.1, AC-003.2 | `grep -qi 'hardest.*question\|harder.*question\|challenge.*planned' skills/grill-me-adversary/SKILL.md && echo PASS \|\| echo FAIL` | PASS |
| PC-010 | integration | Challenge result always shown | AC-003.3 | `grep -qi 'kept.*replaced\|show.*challenge.*result\|always show' skills/grill-me-adversary/SKILL.md && echo PASS \|\| echo FAIL` | PASS |
| PC-011 | integration | Five-stage preservation | AC-003.4 | `grep -qi 'five.stage\|5.stage' skills/grill-me-adversary/SKILL.md && echo PASS \|\| echo FAIL` | PASS |
| PC-012 | integration | Two scoring axes | AC-004.1 | `grep -qi 'completeness' skills/grill-me-adversary/SKILL.md && grep -qi 'specificity' skills/grill-me-adversary/SKILL.md && echo PASS \|\| echo FAIL` | PASS |
| PC-013 | integration | Completeness anchors (4 levels) | AC-004.2 | `grep -c 'No relevant aspects addressed\|One aspect addressed\|Most aspects addressed\|All aspects addressed' skills/grill-me-adversary/SKILL.md` | 4 |
| PC-014 | integration | Specificity anchors (4 levels) | AC-004.2 | `grep -c 'Entirely vague\|relies on hand-waving\|Concrete examples\|falsifiable throughout' skills/grill-me-adversary/SKILL.md` | 4 |
| PC-015 | integration | Follow-up threshold < 2 | AC-004.3, AC-005.1 | `grep -qi 'either.*below 2\|either.*< 2\|axis.*< 2\|below.*2' skills/grill-me-adversary/SKILL.md && echo PASS \|\| echo FAIL` | PASS |
| PC-016 | integration | Inline score display | AC-004.4 | `grep -qi 'inline\|show.*score' skills/grill-me-adversary/SKILL.md && echo PASS \|\| echo FAIL` | PASS |
| PC-017 | integration | Deflection handling | AC-004.5 | `grep -qi 'deflect' skills/grill-me-adversary/SKILL.md && echo PASS \|\| echo FAIL` | PASS |
| PC-018 | integration | Three-attempt cap | AC-005.2, AC-005.5 | `grep -qi 'three.*attempt\|3.*attempt' skills/grill-me-adversary/SKILL.md && echo PASS \|\| echo FAIL` | PASS |
| PC-019 | integration | "Stress-tested but unresolved" label | AC-005.3, AC-005.4 | `grep -q 'stress-tested but unresolved' skills/grill-me-adversary/SKILL.md && echo PASS \|\| echo FAIL` | PASS |
| PC-020 | integration | "Skipped" label | AC-005.6 | `grep -qi 'skipped' skills/grill-me-adversary/SKILL.md && echo PASS \|\| echo FAIL` | PASS |
| PC-021 | integration | Tone guidance documented | Decision 6 | `grep -qi 'firm.*curious' skills/grill-me-adversary/SKILL.md && echo PASS \|\| echo FAIL` | PASS |
| PC-022 | integration | Body word count <= 500 | AC-006.1, AC-006.2 | `awk '/^---$/{c++; if(c==2) start=1; next} start' skills/grill-me-adversary/SKILL.md \| wc -w \| awk '{if ($1 <= 500) print "PASS ("$1" words)"; else print "FAIL ("$1" words)"}'` | PASS |
| PC-023 | integration | Adversary Mode section exists | AC-007.1 | `grep -c '## Adversary Mode' skills/grill-me/SKILL.md` | 1 |
| PC-024 | integration | Section <= 5 lines | AC-007.2 | `awk '/^## Adversary Mode/{found=1; next} found && /^## /{exit} found && NF>0{count++} END{if(count<=5) print "PASS ("count" lines)"; else print "FAIL ("count" lines)"}' skills/grill-me/SKILL.md` | PASS |
| PC-025 | integration | Activation triggers | AC-007.3 | `grep -q 'adversary mode' skills/grill-me/SKILL.md && grep -q 'hard mode' skills/grill-me/SKILL.md && echo PASS \|\| echo FAIL` | PASS |
| PC-026 | integration | References grill-me-adversary | AC-007.4 | `grep -q 'grill-me-adversary' skills/grill-me/SKILL.md && echo PASS \|\| echo FAIL` | PASS |
| PC-027 | integration | Placement order | AC-007.5, AC-007.7 | `awk '/^## Negative Examples/{ne=NR} /^## Adversary Mode/{am=NR} /^## Output/{out=NR} END{if(ne<am && am<out) print "PASS"; else print "FAIL"}' skills/grill-me/SKILL.md` | PASS |
| PC-028 | integration | No scoring UI in base | AC-007.6 | `grep -c 'completeness\|specificity\|0-3' skills/grill-me/SKILL.md \|\| echo 0` | 0 |
| PC-029 | integration | Glossary entry | AC-009.1 | `grep -q 'Adversary Mode' docs/domain/glossary.md && echo PASS \|\| echo FAIL` | PASS |
| PC-030 | integration | CHANGELOG entry | AC-009.2 | `grep -q 'BL-057' CHANGELOG.md && echo PASS \|\| echo FAIL` | PASS |
| PC-031 | integration | ecc validate skills | AC-008.1 | `cargo run -- validate skills 2>&1 \| grep -c 'error' \|\| echo 0` | 0 |
| PC-032 | lint | Clippy clean | AC-008.2 | `cargo clippy -- -D warnings` | exit 0 |
| PC-033 | build | Build passes | AC-008.2 | `cargo build` | exit 0 |
| PC-034 | test | Test regression | AC-008.3 | `cargo test` | exit 0 |
| PC-035 | integration | Directory name matches frontmatter name | AC-001.4 | `grep -oP '^name: \K.*' skills/grill-me-adversary/SKILL.md \| grep -q '^grill-me-adversary$' && echo PASS \|\| echo FAIL` | PASS |

### Coverage Check

All 36 ACs covered by 35 PCs. Zero uncovered.

| AC | Covering PC(s) |
|----|---------------|
| AC-001.1 | PC-001 |
| AC-001.2 | PC-002, PC-003, PC-004 |
| AC-001.3 | PC-005, PC-006 |
| AC-001.4 | PC-002, PC-035 |
| AC-002.1 | PC-007 |
| AC-002.2 | PC-007 |
| AC-002.3 | PC-008 |
| AC-003.1 | PC-009 |
| AC-003.2 | PC-009 |
| AC-003.3 | PC-010 |
| AC-003.4 | PC-011 |
| AC-004.1 | PC-012 |
| AC-004.2 | PC-013, PC-014 |
| AC-004.3 | PC-015 |
| AC-004.4 | PC-016 |
| AC-004.5 | PC-017 |
| AC-005.1 | PC-015 |
| AC-005.2 | PC-018 |
| AC-005.3 | PC-019 |
| AC-005.4 | PC-019 |
| AC-005.5 | PC-018 |
| AC-005.6 | PC-020 |
| AC-006.1 | PC-022 |
| AC-006.2 | PC-022 |
| AC-007.1 | PC-023 |
| AC-007.2 | PC-024 |
| AC-007.3 | PC-025 |
| AC-007.4 | PC-026 |
| AC-007.5 | PC-027, PC-028 |
| AC-007.6 | PC-028 |
| AC-007.7 | PC-027 |
| AC-008.1 | PC-031 |
| AC-008.2 | PC-032, PC-033 |
| AC-008.3 | PC-034 |
| AC-009.1 | PC-029 |
| AC-009.2 | PC-030 |

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | Content validation | `ecc validate skills` | ContentValidator | New skill passes validation | ignored | Skill directory added |

### E2E Activation Rules

No E2E tests to un-ignore — validation is covered by PC-031.

## Test Strategy

TDD order (5 phases, 35 PCs):

1. **Phase 1 — Skeleton** (PC-001–006, PC-035): File exists, frontmatter correct, name matches directory
2. **Phase 2 — Content** (PC-007–022): All skill body sections and word count
3. **Phase 3 — Grill-me edit** (PC-023–028): Opt-in section placement, triggers, no scoring leakage
4. **Phase 4 — Docs** (PC-029–030): Glossary and CHANGELOG
5. **Phase 5 — Regression** (PC-031–034): Validate, clippy, build, test

Rationale: File must exist before content checks; content must be complete before word count; grill-me edit references the new skill; docs reference both; regression gates are last.

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/domain/glossary.md` | Domain | Add entry | "Adversary Mode" — opt-in grill-me enhancement with adaptive adversarial questioning and scoring. Placed alphabetically under Domain Terms. | US-009, AC-009.1 |
| 2 | `CHANGELOG.md` | Project | Add entry | BL-057: grill-me-adversary companion skill with adaptive loop. Under v4.2.0 Features. | US-009, AC-009.2 |

No ADRs needed (all 9 decisions marked "No").

## SOLID Assessment

**PASS** — Companion skill pattern correctly separates interview responsibility (grill-me) from evaluation responsibility (grill-me-adversary). SRP maintained. OCP respected. No circular dependencies.

## Robert's Oath Check

**CLEAN** — All 5 oath principles satisfied. 35 pass conditions provide comprehensive proof. 5 TDD phases enable frequent small releases.

## Security Notes

**CLEAR** — Pure Markdown changes. No executable code, no input handling, no auth, no secrets. No attack surface.

## Rollback Plan

Reverse dependency order:
1. Revert `CHANGELOG.md` entry
2. Revert `docs/domain/glossary.md` entry
3. Revert `skills/grill-me/SKILL.md` (remove Adversary Mode section)
4. Delete `skills/grill-me-adversary/` directory

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | PASS | 0 |
| Robert | CLEAN | 0 |
| Security | CLEAR | 0 |

### Adversary Findings

| Dimension | Verdict | Key Rationale |
|-----------|---------|---------------|
| Coverage | PASS (R2) | Full AC-to-PC traceability table, PC-035 added for name match |
| Order | PASS | Phases correctly sequenced |
| Fragility | PASS (R2) | PC-022 frontmatter stripping explained, PC-021 relabeled |
| Rollback | PASS | Clean reverse-order, all changes reversible |
| Architecture | PASS | Correct ECC companion skill pattern |
| Blast Radius | PASS | 4 files, additive only |
| Missing PCs | PASS (R2) | PC-035 added, negative constraints covered by design |
| Doc Plan | PASS (R2) | Glossary durability confirmed via BL-055 precedent |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `skills/grill-me-adversary/SKILL.md` | create | US-001–006 |
| 2 | `skills/grill-me/SKILL.md` | modify | US-007 |
| 3 | `docs/domain/glossary.md` | modify | US-009, AC-009.1 |
| 4 | `CHANGELOG.md` | modify | US-009, AC-009.2 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-23-grill-me-adversary-companion/design.md | Full design |
