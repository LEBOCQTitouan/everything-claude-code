---
id: BL-049
title: Offload web research phase to Task subagents in /spec-* commands
status: open
created: 2026-03-22
promoted_to: ""
tags: [spec, context-management, subagent, web-research, performance]
scope: MEDIUM
target_command: /spec-refactor
---

## Optimized Prompt

Refactor Phase 3 (web research) in all three `/spec-*` commands — `/spec-dev`,
`/spec-fix`, and `/spec-refactor` — to delegate WebSearch calls to a Task
subagent instead of running them inline in the main context.

**Context.**
Currently each `/spec-*` command runs WebSearch directly in the main session
during Phase 3. These calls accumulate throwaway tokens that are never re-used
after the Research Summary is produced. Over a full spec session this inflates
context meaningfully and can trigger `/compact` interruptions.

The grill-me phase (Phase 4) must NOT be offloaded: it needs full prior-phase
context to ask sharp questions, and AskUserQuestion must feel native to the
session. Only Phase 3 is the target.

**What to build.**
In each of the three command files, replace the inline Phase 3 WebSearch block
with a Task subagent invocation:

1. The subagent receives: the user's raw idea and the Requirements Summary
   produced in the prior phase.
2. The subagent performs all WebSearch calls in its own isolated context.
3. The subagent returns a condensed Research Summary (bullet list, ≤20 items)
   to the main context.
4. The main command resumes Phase 4 using only that summary — no raw search
   results leak into main context.

**Files affected.**
- `commands/spec-dev.md` — Phase 3 block
- `commands/spec-fix.md` — Phase 3 block
- `commands/spec-refactor.md` — Phase 3 block

**Acceptance criteria.**
- All three command files delegate Phase 3 to a Task subagent.
- The subagent receives only: raw idea + Requirements Summary (no full prior
  conversation).
- The subagent returns only a Research Summary bullet list to main context.
- No raw WebSearch result tokens appear in the main session after Phase 3.
- Phase 4 (grill-me) is unchanged and still runs inline.
- Existing Phase 3 behavior (topic coverage, quality of summary) is preserved
  or improved.

**Scope boundaries — do NOT:**
- Touch Phase 4 (grill-me) or any other phase.
- Modify `/design` or `/implement`.
- Change the Research Summary format consumed by later phases.
- Add new dependencies or new agent files.

**Verification steps.**
1. Run a sample `/spec-dev` session and confirm no raw search results appear in
   main-context tool output.
2. Confirm the Research Summary quality matches the previous inline approach.
3. Confirm `/spec-fix` and `/spec-refactor` behave identically.
4. Run `/verify` to confirm no regressions in related tests.

## Original Input

Push heavy exploration work into subagents in pipeline commands to keep main
context lean, avoiding slow /compact calls. Specifically: offload Phase 3
(web research) in /spec-dev, /spec-fix, and /spec-refactor to Task subagents
that return only a distilled Research Summary bullet list. Grill-me should NOT
be offloaded because it needs full prior-phase context and must feel native.

## Challenge Log

**Q1: Which parts of the /spec-* commands should be offloaded — web research,
grill-me, or both?**
A: Initially both; refined during curation to web research only (Phase 3).
Grill-me was ruled out because it requires full prior-phase context and
AskUserQuestion must feel native to the main session.

**Q2: Which commands are in scope?**
A: All three /spec-* commands: /spec-dev, /spec-fix, /spec-refactor. /design
and /implement already delegate heavy work and are out of scope.

**Q3: What should the subagent return?**
A: A condensed Research Summary bullet list only — no raw search results.
This is what the main context would have produced inline, minus all the
throwaway WebSearch tokens.

## Related Backlog Items

- BL-028 — Add web search to /spec commands (the feature being refactored here)
- BL-035 — Context window usage monitoring (same motivation: keep main context lean)
