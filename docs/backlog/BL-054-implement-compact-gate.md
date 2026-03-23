---
id: BL-054
title: Full context clear + confirmation gate at /implement start
status: archived
created: 2026-03-22
scope: LOW
target_command: direct edit
tags: [implement, context-clear, session, gate, ux]
---

## Optimized Prompt

Modify the `/implement` command to trigger a full context clear (not compact) at the start, then resume from disk state only. This mirrors the old Claude Code behavior where transitioning from plan to implementation offered "clear context and auto-accept edits" — but made intentional and mandatory by ECC.

**Why clear, not compact:**
- Compact preserves compressed context — but after `/spec` + `/design`, that context is conversation noise (grill-me exchanges, adversary back-and-forth, draft iterations)
- The plan, spec, design, and tasks.md are already on disk in `docs/specs/`. Nothing valuable lives only in context
- A clean session starts with maximum headroom for the most context-intensive phase (TDD loops, subagent results, code diffs)
- Compact at 5% savings isn't worth the trust tradeoff — better to clear fully and rely on the plan file being complete

**Behavior:**
1. At the top of `/implement`, before entering Plan Mode or spawning TDD subagents:
   - Display current context usage percentage: "Context is at XX%."
   - Prompt via AskUserQuestion: "Implementation starts from the plan on disk. Clear context and start a fresh session for /implement? (The plan at docs/specs/<work-item>/plan.md will be loaded.)"
   - On confirmation: instruct the user to clear context (or trigger it programmatically if the API supports it), then `/implement` resumes by reading the plan file cold
   - On decline: proceed in current context (user's choice, no force)
2. The fresh session reads `docs/specs/<work-item>/design.md` and `docs/specs/<work-item>/tasks.md` to reconstruct full implementation context from disk
3. This is a one-time gate at the plan→implement boundary, not a recurring warning

**Reference:** This replicates the UX described in Claude Code issue "Used to always clear context — but now seeing 'clear context (5% used)'" where users valued the hard boundary between planning and execution sessions. ECC makes this structural rather than heuristic.

**Implementation location:** `commands/implement.md` — add as Phase 3 Step 0 before the existing workflow.

## Framework Source

- **User design philosophy**: "Agents are amnesiac by design. Everything important gets written to a campaign file on disk. Next session reads the file and picks up exactly where it left off."
- **Reference**: Claude Code "clear context and auto-accept edits" transition behavior

## Related Backlog Items

- BL-035 (state externalization) — ensures clear is always safe because nothing important lives only in context
- BL-053 (poweruser statusline) — displays context % continuously; this gate is a one-time checkpoint at phase boundary
