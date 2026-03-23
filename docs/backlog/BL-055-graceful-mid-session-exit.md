---
id: BL-055
title: Graceful mid-session exit when context gets heavy
status: archived
created: 2026-03-22
scope: MEDIUM
target_command: /spec dev
tags: [context, resilience, exit, state-dump, session]
---

## Optimized Prompt

Add a graceful exit pattern to ECC pipeline commands: when context gets heavy mid-phase (not just at phase boundaries), the agent writes current state to disk and tells the user to start a new session.

**Behavior:**
1. When context usage crosses a threshold (e.g., 80%) during any long-running ECC command (`/implement`, `/audit-full`, `/spec`):
   - Write current progress to the campaign manifest (`docs/specs/<work-item>/campaign.md`): what's done, what's in progress, what's next
   - Inform the user: "Context is at XX%. State has been saved to `docs/specs/<work-item>/campaign.md`. Start a new session and re-run the command to continue."
   - Exit cleanly — don't try to squeeze more work into degraded context
2. The next session reads the campaign file cold and picks up exactly where the previous one stopped
3. This is distinct from BL-054 (planned boundary clear between phases) — this covers the unplanned case where context fills up mid-work

**Why this matters:**
- The 1M window enables bigger chunks per session, but should still be treated as if it could end at any time
- Output quality degrades in the last 20% of context — better to exit cleanly than produce worse code
- With the campaign manifest (BL-035) in place, resuming is zero-cost

**Implementation:**
- Add a context-check utility that commands can call between logical units of work
- The check reads context % (from the session data available to hooks/statusline)
- Commands integrate the check at natural breakpoints (between TDD tasks, between audit domains, etc.)
- Not a hook — this is command-level logic that knows how to write meaningful state

**What NOT to do:**
- Don't interrupt mid-function or mid-test — wait for the current logical unit to complete
- Don't just warn — actually save state and exit
- Don't estimate tokens from char counts — use the actual context data

## Framework Source

- **User design philosophy**: "When context gets heavy, the agent writes state to the file and exits. I still treat it like it could end at any time. Because it can."

## Related Backlog Items

- BL-035 (campaign manifest) — provides the file format for state dumps
- BL-054 (implement context clear gate) — planned phase boundary; this covers unplanned mid-phase
- BL-053 (poweruser statusline) — displays context % continuously; this acts on it
