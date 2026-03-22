---
id: BL-057
title: Create grill-me-adversary companion skill with adaptive loop
status: open
created: 2026-03-22
promoted_to: ""
tags: [skills, grill-me, adversarial, adaptive, interview]
scope: MEDIUM
target_command: /spec
---

## Optimized Prompt

Create a new companion skill `skills/grill-me-adversary/SKILL.md` that augments the existing `grill-me` skill with a fully adaptive adversarial loop. The skill is optional — grill-me loads it when the user requests "adversary mode" or "hard mode".

**Project context:** Rust workspace + Markdown-based ECC skill system (`skills/`, `agents/`, `commands/`). Skill format: Markdown with YAML frontmatter (`name`, `description`, `origin: ECC`). Skill directory name must match `name` field. Skills must be under 500 words for v1.

**What grill-me-adversary adds (beyond grill-me's core):**

1. **Adversarial question generation** — At each stage, `grill-me-adversary` synthesises a devil's-advocate angle before picking the next question. It deliberately seeks the most uncomfortable angle the user has not yet been pushed on, rather than following the fixed question bank.

2. **Answer evaluation** — After every user answer, the adversary scores the answer on two axes: completeness (0–3) and specificity (0–3). If either score is below 2, it probes deeper with a follow-up before advancing. It shows the score inline so the user knows why they are being re-questioned.

3. **Question generation challenge** — The adversary also challenges grill-me's own initial question selection: at the start of each stage it asks "is the planned question the hardest possible question for this stage?" and substitutes a harder one if not.

4. **Adaptive loop exit** — The loop terminates for a given branch only when both axes score ≥ 2, OR after three follow-up attempts (whichever comes first). On three-attempt exhaustion, the branch is flagged as "stress-tested but unresolved".

**Acceptance criteria:**

- `skills/grill-me-adversary/SKILL.md` exists with valid YAML frontmatter (`name: grill-me-adversary`, `origin: ECC`)
- Skill file is under 500 words
- Skill documents: adversarial question-generation logic, answer evaluation rubric (completeness + specificity 0–3), follow-up trigger threshold (score < 2), three-attempt cap, branch status labels ("stress-tested but unresolved")
- `skills/grill-me/SKILL.md` is updated with a single "Adversary Mode" section (≤ 5 lines) explaining how to opt in and referencing `grill-me-adversary`
- No other changes to grill-me's core five-stage flow
- `ecc validate skills` passes for both skill files

**Scope boundaries (do NOT do):**

- Do not rewrite or restructure grill-me's five stages
- Do not add adversary mode as the default — it must remain opt-in
- Do not add numeric scoring UI to the base grill-me skill
- Do not create a new command or agent for this feature
- Do not exceed 500 words in the new skill file

**Verification steps:**

1. `ecc validate skills` — both skill files pass validation
2. Confirm `skills/grill-me-adversary/SKILL.md` directory name matches `name` frontmatter field
3. Word count check: `wc -w skills/grill-me-adversary/SKILL.md` — must be ≤ 500 words (body only, excluding frontmatter)
4. Manual review: grill-me's opt-in section references `grill-me-adversary` correctly and does not alter any existing behaviour description
5. `cargo clippy -- -D warnings` and `cargo test` still pass (no Rust changes expected, but verify)

**Run with:** `/spec` — new skill development

## Original Input

Add an adversarial layer to grill-me: a companion that both generates questions AND evaluates user answers, runs a fully adaptive adversarial loop, and challenges grill-me's own initial question generation. Implemented as a new companion skill (`grill-me-adversary`) that grill-me optionally loads as a mode, keeping grill-me's core intact.

## Challenge Log

**Q1:** Should the adversary only generate harder questions, or should it also evaluate user answers and decide whether to probe deeper?

**A1:** Both — it generates questions AND evaluates answers, forming a fully adaptive adversarial loop. It also challenges grill-me's initial question generation.

**Q2:** Should this extend grill-me in-place, live as a separate skill, or be a new command?

**A2:** New companion skill (`grill-me-adversary`) that grill-me optionally loads as a mode, keeping grill-me's core intact.

## Related Backlog Items

- [BL-011](BL-011-create-grill-me-skill.md) — grill-me skill (implemented) — this entry extends it via companion
- [BL-034](BL-034-capture-grill-me-decisions-in-work-item-files.md) — capture grill-me decisions (open) — orthogonal but both touch grill-me output
