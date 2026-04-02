---
id: BL-083
title: Adversarial challenge phase for all /audit-* commands
status: "implemented"
created: 2026-03-27
promoted_to: ""
tags: [audit, adversarial, quality, pipeline]
scope: HIGH
target_command: /spec-dev
---

## Optimized Prompt

Add an interleaved adversarial challenge phase to every `/audit-*` command
(`/audit-full`, `/audit-archi`, `/audit-code`, `/audit-convention`, `/audit-doc`,
`/audit-errors`, `/audit-evolution`, `/audit-observability`, `/audit-security`,
`/audit-test`).

**Tech stack:** Rust CLI + Claude Code slash commands (detected from project)

**Workflow:** `/spec-dev` → `/design` → `/implement`

### What to build

Each domain audit (e.g., `/audit-code`) gains an adversary that runs immediately
after that audit completes, before results are surfaced to the user.

The adversary is a **separate agent** (clean context — no pollution from the
audit's own reasoning). It:

1. Re-interrogates the codebase independently to verify or refute the audit's
   findings (it does NOT re-examine the audit agent's reasoning chain).
2. Searches the web for current best practices relevant to the flagged issues.
3. Produces "challenged findings": confirmed, refuted, or amended findings with
   rationale for every dimension evaluated.
4. When it finds nothing to challenge, emits an explicit "clean bill of health"
   statement.
5. On low-quality adversary output (vague, no rationale): retries once with a
   stricter prompt; if still low quality, surfaces a warning alongside the raw
   content.

### Disagreement handling

When the audit and adversary disagree, the user sees both perspectives and
makes the final call. No silent suppression. Recommendations are made with
justification.

### Coverage and cost policy

- Always-on, no opt-in flag.
- Runs full passes every time — no sampling or skipping to cut cost.
- Applies to all `/audit-*` commands, not just `/audit-full`.

### Acceptance criteria

- [ ] Every `/audit-*` command definition includes an adversary agent invocation
  after each domain audit stage completes.
- [ ] Adversary agent has `skills: ["clean-craft"]` and `memory: project`
  frontmatter per ECC adversary conventions.
- [ ] Adversary agent does NOT include `Write` or `Edit` in its tools.
- [ ] Clean bill of health message appears when adversary finds no issues.
- [ ] Disagreement display shows audit finding, adversary challenge, and
  prompts user for final decision.
- [ ] Retry logic triggers on low-quality output; warning surfaces content to
  user if second attempt is also low quality.
- [ ] All existing audit tests pass; new tests cover adversary path (happy,
  refuted, disagreement, low-quality retry).
- [ ] `cargo clippy -- -D warnings` passes with zero warnings.

### Scope boundaries — do NOT

- Do not add an opt-in flag or `--no-adversary` bypass.
- Do not change the domain-specific logic of any existing audit command.
- Do not reuse the audit agent's context for the adversary — separate agent,
  clean spawn.
- Do not modify the `/review` or `/verify` commands.
- Do not implement sampling or partial-pass logic.

### Verification steps

1. Run `/audit-code` on a file with a known issue — confirm adversary output
   appears after audit findings.
2. Inject a deliberately weak adversary response — confirm retry and warning
   flow triggers.
3. Inject a finding the adversary refutes — confirm disagreement display and
   user prompt appear.
4. Run full test suite: `cargo test` — all tests green.
5. `cargo clippy -- -D warnings` — zero warnings.

## Original Input

Add an adversarial phase to audit commands — a separate agent that re-interrogates
the codebase, challenges the audit's findings, and searches the web for best
practices. Interleaved (runs after each domain audit, not once at the end).
Disagreements shown to the user who makes the final call. Clean bill of health
when adversary finds nothing. Retry with stricter prompt on low-quality output.
Always-on, no opt-in flag, full passes every time.

## Challenge Log

Mode: backlog-mode (escalated to full 5 stages, HIGH scope)

### Stage 1: Clarity

**Q1**: Which audit commands are in scope?
**A**: ALL `/audit-*` commands, not just `/audit-full`.
**Status**: answered

**Q2**: What does the adversary actually re-examine?
**A**: The adversary re-interrogates codebase status independently (not the audit
agent's reasoning chain). It also searches the web for best practices related
to the flagged issues.
**Status**: answered

### Stage 2: Assumptions

**Q3**: Interleaved or batch?
**A**: Interleaved — adversary runs after each domain audit completes, before
moving to the next.
**Status**: answered

**Q4**: How are disagreements resolved?
**A**: Displayed to the user, who makes the final choice. No silent suppression.
**Status**: answered

### Stage 3: Edge Cases

**Q5**: What happens when the adversary finds nothing?
**A**: Explicit "clean bill of health" message emitted.
**Status**: answered

**Q6**: What happens when adversary output is low quality?
**A**: Retry with stricter prompt. If still low quality, surface warning alongside
the content to the user.
**Status**: answered

### Stage 4: Alternatives

**Q7**: Why a separate agent rather than a second pass by the same agent?
**A**: Separate agents justified by context pollution research — clean context
produces better critical evaluation.
**Status**: answered

**Q8**: Should this be opt-in?
**A**: Always-on. No opt-in flag.
**Status**: answered

### Stage 5: Stress Test

**Q9**: Who decides when audit and adversary genuinely disagree?
**A**: Always ask the user, but make justified recommendations.
**Status**: answered

**Q10**: What about cost — skip or sample on large codebases?
**A**: Always run full passes. Accept the cost, never sample or skip.
**Status**: answered

## Related Backlog Items

- BL-036: Numeric quality scores for adversary agents — adversary output quality
  scoring connects to retry logic here.
- BL-038: TaskCreate in audit-full and doc-orchestrator — parallel execution
  context overlaps.
- BL-042: Background mode for /audit-full — background execution model may
  interact with interleaved adversary timing.
- BL-043: QA strategist agent — a QA strategist could consume adversary-challenged
  findings as input.
