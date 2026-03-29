# Design: Socratic Grill-Me Upgrade (BL-098)

Spec: `docs/specs/2026-03-29-socratic-grill-me-upgrade/spec.md`

## Overview

Rewrite the grill-me skill to embed four Socratic techniques (OARS, Laddering, MECE Decomposition, 6-Type Rotation) with depth profiles. Update the adversary companion skill. Create ADR-0032. Markdown-only change — no Rust code.

---

## File Changes

| # | File | Change | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `skills/grill-me/SKILL.md` | Add `## OARS Protocol` section with Open/Acknowledge/Reflect/Summarize sequence. Acknowledge uses factual recognition, never praise. Reflect is mandatory after every answer. Summarize emitted at stage transitions. | Core conversational framework replacing raw challenge loop | AC-001.1 through AC-001.5 |
| 2 | `skills/grill-me/SKILL.md` | Add `## Laddering` section. Progressive "why?" drilling when answers are abstract/vague. Depth-7 safety valve. Terminates on concrete/falsifiable answer. | Depth-first probing replaces flat question lists | AC-002.1 through AC-002.5 |
| 3 | `skills/grill-me/SKILL.md` | Add `## MECE Decomposition` section. Universal decomposition of requirement spaces into ME/CE sub-questions. Atomic-topic exemption with rationale note. | Exhaustive coverage guarantee | AC-003.1 through AC-003.4 |
| 4 | `skills/grill-me/SKILL.md` | Add `## Socratic Type Rotation` section. 6 types: Clarification, Assumption, Evidence, Viewpoint, Implication, Meta. `[Type]` annotation visible on every question. No type exceeds 2x fair share. | Multi-angle reasoning challenge | AC-004.1 through AC-004.4 |
| 5 | `skills/grill-me/SKILL.md` | Add `## Depth Profiles` section. Three profiles: shallow (OARS Reflect only, MECE max 2 branches), standard (full OARS + 1-2 ladder levels + MECE), deep (all techniques full intensity). Mode defaults: backlog=standard, spec=deep, standalone=deep. Consumer override documented. Profile controls intensity within mode limits. | Context-appropriate technique intensity | AC-005.1 through AC-005.7 |
| 6 | `skills/grill-me/SKILL.md` | Remove `## Question Cap` section (25-question cap). Remove all references to "25-question cap" and "cap" counting throughout the file. | Cap conflicts with depth-first philosophy | AC-006.1 |
| 7 | `skills/grill-me/SKILL.md` | Update `## Modes` — all three modes reference new techniques. Standalone and spec-mode use `deep` profile default. Backlog-mode uses `standard` profile default. Add note that consuming commands can override profile. | All modes must use new techniques | AC-006.4, AC-005.4 |
| 8 | `skills/grill-me/SKILL.md` | Update `## Challenge Loop` to integrate OARS: Reflect fires before challenge evaluation. Laddering replaces flat follow-up when answer is abstract. Socratic type tag on each follow-up. | OARS and challenge loop must compose | AC-001.5 |
| 9 | `skills/grill-me/SKILL.md` | Update `## Adversary Mode` reference to note OARS Reflect fires before adversary scoring. | Ordering contract | AC-001.5 |
| 10 | `skills/grill-me/SKILL.md` | Update frontmatter `description` to mention Socratic techniques and depth profiles. | Discoverability | AC-006.7 |
| 11 | `skills/grill-me-adversary/SKILL.md` | Update `## Question-Generation Challenge` to use Socratic type annotations when substituting questions. Substituted questions carry `[Type]` tag. | Adversary alignment with Socratic rotation | AC-006.2, AC-006.6 |
| 12 | `skills/grill-me-adversary/SKILL.md` | Update `## Adversarial Question Generation` to reference OARS, laddering, and Socratic type rotation as enhanced question types. | Adversary uses full technique palette | AC-006.2 |
| 13 | `docs/adr/0033-socratic-questioning-protocol.md` | New ADR covering decisions 1-5 from spec. Includes research-to-implementation type mapping (elenchus -> Assumption, maieutics -> Evidence, etc.). Supersedes relevant parts of ADR-0017. | Architectural decision record | AC-006.5 |
| 14 | `docs/adr/0017-grill-me-universal-protocol.md` | Add supersession note: "Partially superseded by ADR-0033 — question cap removed, depth profiles introduced." | Prevent ADR staleness | — |
| 15 | `CHANGELOG.md` | Add BL-098 entry under current version. | Standard changelog practice | — |

---

## Pass Conditions

| # | Type | Description | Verifies AC | Command | Expected |
|---|------|-------------|-------------|---------|----------|
| PC-01 | grep | OARS Protocol section exists | AC-001.* | `grep -c '## OARS Protocol' skills/grill-me/SKILL.md` | `1` |
| PC-02 | grep | Acknowledge uses factual recognition, not praise | AC-001.2 | `grep -c 'Acknowledge' skills/grill-me/SKILL.md` | `>= 2` (section + definition) |
| PC-03 | grep | No "Affirm" in OARS (must be Acknowledge) | AC-001.2 | `grep -ci 'Affirm' skills/grill-me/SKILL.md` | `0` |
| PC-04 | grep | Reflect is mandatory | AC-001.1 | `grep -c 'Reflect' skills/grill-me/SKILL.md` | `>= 3` |
| PC-05 | grep | Summarize at stage transition | AC-001.3 | `grep -q 'Summarize' skills/grill-me/SKILL.md && echo PASS` | `PASS` |
| PC-06 | grep | Laddering section exists | AC-002.* | `grep -c '## Laddering' skills/grill-me/SKILL.md` | `1` |
| PC-07 | grep | Depth-7 safety valve documented | AC-002.5 | `grep -q 'depth.*7\|7.*depth\|seven.*depth\|depth.*seven' skills/grill-me/SKILL.md && echo PASS` | `PASS` |
| PC-08 | grep | MECE Decomposition section exists | AC-003.* | `grep -c '## MECE Decomposition' skills/grill-me/SKILL.md` | `1` |
| PC-09 | grep | Atomic-topic exemption documented | AC-003.4 | `grep -qi 'atomic' skills/grill-me/SKILL.md && echo PASS` | `PASS` |
| PC-10 | grep | Socratic Type Rotation section exists | AC-004.* | `grep -c '## Socratic Type Rotation' skills/grill-me/SKILL.md` | `1` |
| PC-11 | grep | All 6 Socratic types present | AC-004.1 | `for t in Clarification Assumption Evidence Viewpoint Implication Meta; do grep -q "\[$t\]" skills/grill-me/SKILL.md || echo "MISSING: $t"; done` | (no output) |
| PC-12 | grep | Depth Profiles section exists | AC-005.* | `grep -c '## Depth Profiles' skills/grill-me/SKILL.md` | `1` |
| PC-13 | grep | All 3 profiles defined | AC-005.1-3 | `for p in shallow standard deep; do grep -qi "$p" skills/grill-me/SKILL.md || echo "MISSING: $p"; done` | (no output) |
| PC-14 | grep | Mode defaults documented | AC-005.4 | `grep -q 'backlog.*standard' skills/grill-me/SKILL.md && grep -q 'spec.*deep' skills/grill-me/SKILL.md && grep -q 'standalone.*deep' skills/grill-me/SKILL.md && echo PASS` | `PASS` |
| PC-15 | grep | 25-question cap removed | AC-006.1 | `grep -ci '25.question\|question cap\|25-question' skills/grill-me/SKILL.md` | `0` |
| PC-16 | grep | Question Cap section removed | AC-006.1 | `grep -c '## Question Cap' skills/grill-me/SKILL.md` | `0` |
| PC-17 | grep | Skip/exit behavior preserved | AC-006.3 | `grep -q '## Skip and Exit' skills/grill-me/SKILL.md && echo PASS` | `PASS` |
| PC-18 | grep | Adversary uses Socratic type annotations | AC-006.6 | `grep -q '\[Type\]\|Socratic type\|type annotation' skills/grill-me-adversary/SKILL.md && echo PASS` | `PASS` |
| PC-19 | grep | Adversary references enhanced question types | AC-006.2 | `grep -qi 'OARS\|laddering\|Socratic' skills/grill-me-adversary/SKILL.md && echo PASS` | `PASS` |
| PC-20 | grep | ADR-0032 exists | AC-006.5 | `test -f docs/adr/0033-socratic-questioning-protocol.md && echo PASS` | `PASS` |
| PC-21 | grep | ADR contains research-to-implementation mapping | AC-006.5 | `grep -q 'elenchus\|maieutics' docs/adr/0033-socratic-questioning-protocol.md && echo PASS` | `PASS` |
| PC-22 | grep | OARS Reflect fires before adversary scoring | AC-001.5 | `grep -qi 'Reflect.*before.*advers\|Reflect.*before.*scor' skills/grill-me/SKILL.md && echo PASS` | `PASS` |
| PC-23 | grep | All 5 required section headers present | AC-006.7 | `for h in "OARS Protocol" "Laddering" "MECE Decomposition" "Socratic Type Rotation" "Depth Profiles"; do grep -q "## $h" skills/grill-me/SKILL.md || echo "MISSING: $h"; done` | (no output) |
| PC-24 | grep | No "Affirm" in adversary either | AC-001.2 | `grep -ci 'Affirm' skills/grill-me-adversary/SKILL.md` | `0` |
| PC-25 | grep | 2x fair share rotation constraint documented | AC-004.4 | `grep -q '2x\|twice.*fair\|double.*fair' skills/grill-me/SKILL.md && echo PASS` | `PASS` |
| PC-26 | grep | Profile override documented | AC-005.5 | `grep -qi 'override' skills/grill-me/SKILL.md && echo PASS` | `PASS` |
| PC-27 | grep | Mode limits take precedence over profile | AC-005.6 | `grep -qi 'mode.*limits.*precedence\|limits.*take.*precedence\|mode.*limits.*override\|stage.*limits.*precedence' skills/grill-me/SKILL.md && echo PASS` | `PASS` |
| PC-28 | build | Cargo workspace still builds | — | `cargo build` | exit 0 |
| PC-29 | lint | Clippy passes | — | `cargo clippy -- -D warnings` | exit 0 |
| PC-30 | grep | CHANGELOG has BL-098 entry | — | `grep -q 'BL-098' CHANGELOG.md && echo PASS` | `PASS` |
| PC-31 | grep | Skill file under 800 lines | — | `test $(wc -l < skills/grill-me/SKILL.md) -lt 800 && echo PASS` | `PASS` |
| PC-32 | grep | ADR-0017 has supersession note | — | `grep -qi 'superseded.*0033\|superseded.*ADR-033' docs/adr/0017-grill-me-universal-protocol.md && echo PASS` | `PASS` |
| PC-33 | grep | Cross-stage mutation section has no cap reference | AC-006.1 | `grep -A5 'Cross-Stage' skills/grill-me/SKILL.md \| grep -ci 'cap' \| grep -q '^0$' && echo PASS` | `PASS` |

---

## TDD Order

Since this is a markdown-only change, "TDD" means: write the grep-based verification script first (RED — tests fail because content does not exist yet), then write the markdown content (GREEN — tests pass), then refactor for clarity.

### Phase 1: Grill-Me Core Techniques (File Changes 1-6, 8)

**Layers**: Entity (skill protocol definition)

**RED**: Create a verification script or run PCs 01-09, 15-16, 22, 23, 25 against the unchanged file. All must fail.

**GREEN**: Edit `skills/grill-me/SKILL.md`:
1. Remove `## Question Cap` section and all 25-question/cap references
2. Add `## OARS Protocol` section after `## Question Statuses`
3. Add `## Laddering` section after OARS Protocol
4. Add `## MECE Decomposition` section after Laddering
5. Add `## Socratic Type Rotation` section after MECE Decomposition
6. Update `## Challenge Loop` to integrate OARS Reflect before challenge evaluation, laddering for abstract answers, and Socratic type tags

**REFACTOR**: Review section ordering and internal cross-references for consistency.

**Commit cadence**:
- `test: add BL-098 phase 1 structural verification (PCs 01-09, 15-16, 22-23, 25)`
- `feat: add OARS, Laddering, MECE, Socratic rotation to grill-me skill`
- `refactor: improve grill-me section ordering` (if changes made)

**Boy Scout Delta**: Check `skills/grill-me/SKILL.md` for any stale TODOs or dead cross-references.

---

### Phase 2: Depth Profiles and Mode Updates (File Changes 5, 7, 9, 10)

**Layers**: Entity (skill protocol definition)

**RED**: Run PCs 12-14, 26-27 against the file. All must fail.

**GREEN**: Edit `skills/grill-me/SKILL.md`:
1. Add `## Depth Profiles` section after Socratic Type Rotation
2. Update `## Modes` — standalone/spec-mode default to `deep`, backlog-mode defaults to `standard`
3. Add consumer override documentation
4. Add "mode limits take precedence" rule
5. Update `## Adversary Mode` reference for OARS Reflect ordering
6. Update frontmatter description

**REFACTOR**: Ensure profile definitions and mode sections are internally consistent.

**Commit cadence**:
- `test: add BL-098 phase 2 structural verification (PCs 12-14, 26-27)`
- `feat: add depth profiles and update modes in grill-me skill`
- `refactor: align profile descriptions with mode sections` (if changes made)

**Boy Scout Delta**: Check `skills/grill-me-adversary/SKILL.md` for stale references to the 25-question cap.

---

### Phase 3: Adversary Companion Update (File Changes 11-12)

**Layers**: Entity (skill protocol definition)

**RED**: Run PCs 18-19, 24 against the adversary file. Must fail (18-19) or already pass (24).

**GREEN**: Edit `skills/grill-me-adversary/SKILL.md`:
1. Update `## Question-Generation Challenge` to include `[Type]` annotation on substituted questions
2. Update `## Adversarial Question Generation` to reference OARS, laddering, and Socratic type rotation
3. Ensure no "Affirm" language

**REFACTOR**: Verify adversary and grill-me files use consistent terminology.

**Commit cadence**:
- `test: add BL-098 phase 3 structural verification (PCs 18-19, 24)`
- `feat: update grill-me-adversary with Socratic techniques`
- `refactor: align adversary terminology with grill-me` (if changes made)

**Boy Scout Delta**: Remove any dead references in adversary skill.

---

### Phase 4: ADR and CHANGELOG (File Changes 13-14)

**Layers**: Adapter (documentation artifacts)

**RED**: Run PCs 20-21, 30. Must fail.

**GREEN**:
1. Create `docs/adr/0033-socratic-questioning-protocol.md` with:
   - Status: Accepted
   - Context: limitations of current flat questioning
   - Decision: embed OARS + Laddering + MECE + Socratic rotation with depth profiles
   - Research-to-implementation type mapping table (elenchus -> Assumption, maieutics -> Evidence, dialectic -> Viewpoint, generalization -> Implication, counterfactual -> Meta, clarification -> Clarification)
   - Consequences: uncapped depth, profile-based intensity, backward compatible
   - Supersedes relevant depth/cap decisions from ADR-0017
2. Add BL-098 entry to `CHANGELOG.md`

**REFACTOR**: Verify ADR cross-references are correct.

**Commit cadence**:
- `test: add BL-098 phase 4 structural verification (PCs 20-21, 30)`
- `docs: add ADR-0032 socratic questioning protocol`
- `docs: add BL-098 to CHANGELOG`

**Boy Scout Delta**: Check `docs/adr/README.md` for stale ADR index entries.

---

### Phase 5: Full Verification Gate

**Layers**: none (verification only)

Run all 30 PCs. Run `cargo build` and `cargo clippy -- -D warnings` to confirm no regressions (PCs 28-29). No commit from this phase unless a fix is needed.

---

## E2E Assessment

- **Touches user-facing flows?** Yes — grill-me skill is consumed by /spec-*, /backlog, and standalone
- **Crosses 3+ modules end-to-end?** No — changes are confined to 2 skill files + 1 ADR + CHANGELOG
- **New E2E tests needed?** No — structural verification via grep is sufficient; behavioral verification is manual (run grill-me with a test prompt)
- Existing Rust test suite and Bats tests run as regression gate

## Risks and Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| Skill file exceeds 800-line convention | Medium | Keep technique sections concise (protocol rules, not examples). Monitor line count after each phase. |
| ADR numbering conflict (duplicate 0030/0031 exist) | Low | Use 0032 which is unambiguously next. Note in ADR that spec references "ADR-033". |
| Consumers break if they grep for "Question Cap" | Low | Search codebase for references to "Question Cap" or "25-question" before removing. |
| Adversary and grill-me terminology drift | Low | Phase 3 refactor step explicitly checks cross-file consistency. |

## Success Criteria

- [ ] All 30 pass conditions pass
- [ ] `cargo build` succeeds (no Rust regression)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] Skill file under 800 lines
- [ ] Manual smoke test: run "grill me" on a test topic, observe OARS sequence, Socratic tags, and laddering behavior
