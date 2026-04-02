---
id: BL-045
title: Remove alias commands (plan, solution) and audit for further duplicates
status: "implemented"
created: 2026-03-21
promoted_to: ""
tags: [commands, cleanup, aliases, refactor, dx]
scope: MEDIUM
target_command: /spec-refactor
---

## Optimized Prompt

The ECC `commands/` directory contains thin alias wrappers that forward to canonical commands:
- `commands/plan.md` → delegates to `/spec`
- `commands/solution.md` → delegates to `/design`

These aliases add maintenance surface, confuse new contributors, and contradict the CLAUDE.md guidance ("Old names (`/plan`, `/solution`) still work as aliases"). Now that the spec-driven pipeline is stable and users are expected to use canonical names, the aliases should be removed.

**Scope of change:**

1. **Delete alias files:**
   - `commands/plan.md`
   - `commands/solution.md`

2. **Audit remaining commands for further aliases or redundancy:**
   - `commands/plan-dev.md` — full command (not an alias), but check if it is superseded by `commands/spec-dev.md`
   - `commands/plan-fix.md` — full command (not an alias), but check if it is superseded by `commands/spec-fix.md`
   - `commands/plan-refactor.md` — full command (not an alias), but check if it is superseded by `commands/spec-refactor.md`
   - Any other commands that may forward or duplicate existing ones

3. **Update all references:**
   - `CLAUDE.md`: remove "Old names (`/plan`, `/solution`) still work as aliases" note, update any `/plan` or `/solution` references to canonical names
   - `docs/commands-reference.md`: remove alias entries
   - Any agent, skill, or hook files that reference `/plan` or `/solution` as command names
   - `skills/backlog-management/skill.md`: `target_command` field examples

4. **Grep-verify no dangling references:**
   - `rg "/plan" commands/ agents/ skills/ hooks/ docs/ CLAUDE.md` — confirm no remaining references to deleted aliases
   - `rg "/solution" commands/ agents/ skills/ hooks/ docs/ CLAUDE.md`

**Acceptance criteria:**
- AC-1: `commands/plan.md` and `commands/solution.md` are deleted
- AC-2: `plan-dev`, `plan-fix`, `plan-refactor` are either deleted (if superseded by spec-* equivalents) or retained with a clear rationale documented in a comment
- AC-3: All inbound references to `/plan` and `/solution` in docs, agents, skills, hooks, and CLAUDE.md are updated to canonical equivalents
- AC-4: `grep -r "/plan\b" commands/ agents/ skills/ hooks/` returns zero matches (except within `/plan-*` file names if those are retained)
- AC-5: `CLAUDE.md` no longer mentions the old alias names

**Out of scope:**
- Do NOT rename or delete `/plan-dev`, `/plan-fix`, `/plan-refactor` without first diffing them against their `/spec-*` counterparts to confirm they are truly superseded
- Do NOT change command logic — this is a structural cleanup only
- Do NOT add new commands

**Verification:**
1. Run `grep -r "plan.md\|/plan\b\|/solution\b" commands/ agents/ skills/ hooks/ docs/ CLAUDE.md` and confirm zero matches (except intentional retention)
2. Run `/spec add a login page` and confirm it still routes correctly after alias removal
3. Run `/design` and confirm it works standalone

## Original Input

Remove duplicate/alias commands. Currently `/plan` is an alias for `/spec`, `/solution` is an alias for `/design`. Audit all commands for duplicates/aliases and consolidate — remove the old aliases and keep only the canonical names. Check if other duplicates persist beyond spec/plan and solution/design.

## Challenge Log

No questions asked — intent confirmed directly by user. Reasonable default applied: `plan-dev`, `plan-fix`, `plan-refactor` are flagged for audit (may be superseded) but not pre-judged as deletable without a diff against their `/spec-*` equivalents.

## Related Backlog Items

- BL-019: Create /spec command (implemented — the canonical command this alias points to)
- BL-020: Create /design command (implemented — the canonical command solution.md points to)
