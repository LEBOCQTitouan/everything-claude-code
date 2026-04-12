---
id: BL-145
title: Wire party-mode into /spec phase as augmentation layer before adversarial review
status: open
created: 2026-04-12
promoted_to: ""
tags: [spec, party-mode, adversary, grill-me, augmentation, spec-dev, spec-fix, spec-refactor]
scope: MEDIUM
target_command: /spec-dev
depends_on: [BL-144]
---

## Optimized Prompt

Wire party-mode (BL-144) as an optional augmentation step inside `/spec-dev`, `/spec-fix`, and `/spec-refactor` — exclusively at those three commands, nowhere else.

**Context:** ECC's spec pipeline ends each spec phase with an adversarial review (spec-adversary). Party-mode is a BMAD-style multi-agent round-table that challenges the user's thinking. The goal is to insert party-mode between the spec Plan Mode review and the spec-adversary verdict, so agents upgrade the spec before the adversary judges it. Party-mode does NOT replace spec-adversary and does NOT affect the PASS/FAIL/CONDITIONAL verdict mechanics.

**Trigger:** Opt-in. During the spec Plan Mode review (after the spec draft is shown, before the user approves for adversarial review), surface a new prompt option: "Run party-mode on this spec?" The user must explicitly choose it — it is never auto-triggered.

**Flow when triggered:**
1. User reviews spec draft in Plan Mode.
2. User opts in to party-mode.
3. Party-mode round-table runs against the spec draft — agents challenge assumptions, surface gaps, and propose upgrades.
4. Party-mode findings are presented to the user as a structured report.
5. User revises the spec draft based on findings (or accepts as-is).
6. Existing spec-adversary runs on the (possibly revised) spec.
7. PASS/FAIL/CONDITIONAL verdict fires as normal.

**Acceptance criteria:**
- [ ] Party-mode opt-in prompt appears in the Plan Mode review step of `/spec-dev`, `/spec-fix`, and `/spec-refactor` only.
- [ ] Party-mode invocation passes the current spec draft as context to BL-144's `/party` command (or equivalent agent entry point).
- [ ] Party-mode output is rendered as a structured findings report before the user is asked to revise.
- [ ] The spec-adversary always runs after party-mode completes, unchanged.
- [ ] PASS/FAIL/CONDITIONAL verdict logic is untouched.
- [ ] No changes to `/design`, `/implement`, `/verify`, or any `/audit-*` commands.
- [ ] The opt-in is skippable — user can bypass party-mode and go straight to spec-adversary.

**Scope boundaries (NOT in scope):**
- Do not modify `/design`, `/implement`, `/verify`, `/audit-*`, or any other command.
- Do not change spec-adversary verdict mechanics.
- Do not make party-mode the default (always opt-in).
- Do not implement BL-144 itself — BL-145 depends on BL-144 being available.

**Verification steps:**
1. Run `/spec-dev` on a new feature — confirm party-mode opt-in prompt appears at Plan Mode review.
2. Choose party-mode — confirm findings report is shown before adversarial review.
3. Skip party-mode — confirm spec-adversary runs immediately with no party-mode invocation.
4. Run `/design` and `/implement` — confirm no party-mode prompt appears.
5. Run `/verify` and `/audit-*` — confirm no party-mode prompt appears.
6. Trigger a CONDITIONAL verdict after party-mode — confirm verdict logic is unchanged.

## Original Input

"party-mode integration at ECC workflow gates"

Scoped during grill-me to: augment, not replace. Party-mode challenges the user's thinking on spec elements to upgrade them. Wired into /spec phase only (/spec-dev, /spec-fix, /spec-refactor). Runs after spec Plan Mode review, before spec-adversary. Opt-in. Output: party-mode findings presented to user, user revises spec, then spec-adversary runs. Does not affect PASS/FAIL/CONDITIONAL verdict mechanics.

## Challenge Log

Mode: backlog-mode
Depth Profile: standard
Stages completed: 1/3 (sufficient signal reached after Stage 1)
Questions answered: 1/1
Questions skipped: 0

### Stage 1: Clarity

**Q1**: [Clarification] Is party-mode meant to replace the existing spec-adversary, augment it, or run independently?
**A**: Augment, not replace. Party-mode's role is to challenge the user's thinking on elements presented, to upgrade them. It should ONLY be wired into the /spec phase — NOT /design, /implement, /verify, or /audit-*. Runs after spec Plan Mode review, before spec-adversary. Opt-in. Output presented to user for revision, then spec-adversary runs unchanged.
**Status**: answered — sufficient signal to optimize, remaining stages skipped by curator.

## Related Backlog Items

- **BL-144** (depends-on): `/party` command — BMAD-style multi-agent round-table. BL-145 is blocked until BL-144 is available.
- **BL-143**: `/project-foundation` command — overlapping grill-me/interview-me integration patterns (independent).
