---
id: BL-035
title: Campaign file manifest for amnesiac agents
status: implemented
created: 2026-03-21
scope: MEDIUM
target_command: /spec refactor
tags: [context, state, resilience, campaign-file, manifest, amnesiac]
---

## Optimized Prompt

Refactor the ECC pipeline so that no important state lives only in context. The design principle: if losing context would lose progress, the system is fragile. Externalize all state to disk so clearing context is always free.

**Core change — campaign manifest per work item:**
Each work item in `docs/specs/<work-item>/` should have a manifest file (e.g., `campaign.md`) that maps all state files with locations and short descriptions. A fresh agent reads this single file to orient instantly without exploring. Contents:
- What was planned (pointer to spec + design files)
- What was built (pointer to tasks.md with completion status)
- What decisions were made (pointer to decisions log from grill-me, adversary verdicts, user confirmations)
- What's left to do (remaining tasks, blockers)

This is the "capability manifest" pattern: agents have the manifest in memory and can search for relevant information without burning context exploring the directory.

**What to audit and fix:**
1. All pipeline phases already persist specs, designs, and tasks.md — audit that no phase relies on in-context-only state
2. Decision trail from grill-me interviews, adversary verdicts, and user confirmations must be written to the campaign file before proceeding
3. Agent subagent results (TDD outcomes, review findings) must be captured in tasks.md or the campaign file, not just returned to the parent context
4. The strategic-compact skill should reflect this philosophy: clearing is always safe because state is externalized

**Success criteria:**
- Any session can be interrupted at any point and resumed from disk state alone
- A fresh agent can orient from the campaign manifest in one file read
- No pipeline command relies on prior conversation turns for correctness

## Framework Source

- **User design philosophy**: "Agents are amnesiac by design. Everything important gets written to a campaign file on disk. Next session reads the file and picks up exactly where it left off."
- **Community pattern (DevMoses)**: "Capability manifests — a map of what exists, where it lives, and what it does, so the agent can orient without burning context exploring."

## Related Backlog Items

- BL-029 (persist specs as file artifacts) — already implemented, foundation
- BL-030 (persist tasks.md) — already implemented, foundation
- BL-034 (capture grill-me decisions) — open, part of this externalization
- BL-054 (implement context clear gate) — planned phase boundary clear
- BL-055 (graceful mid-session exit) — unplanned mid-work context save + exit
