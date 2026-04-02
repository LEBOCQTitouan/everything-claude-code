---
id: BL-004
title: "robert: read-only + memory + negative examples"
tier: 2
scope: MEDIUM
target: direct edit
status: "implemented"
created: 2026-03-20
file: agents/robert.md
---

## Action

Four changes:

1. Remove `Write` from the tools list — robert becomes strictly `Read`, `Grep`, `Glob`. The calling command (`/review`, `/verify`, `/audit`) captures robert's structured output and writes `robert-notes.md` itself.
2. Add `memory: project` to frontmatter so rework ratio trends, recurring violations, and accepted exceptions persist across sessions.
3. Add `ubiquitous-language` to the skills array once BL-010 exists; until then add `architecture-review` if not already present.
4. Add an "Anti-Patterns" section with these four lines:
   - "DO NOT modify source code or tests"
   - "DO NOT approve work you haven't verified against each Oath point"
   - "DO NOT skip the self-audit section even if the main review is clean"
   - "DO NOT soften findings — a FAIL is a FAIL, explain why and move on"

## Dependencies

- BL-005 (commands must handle robert's output after Write removal)
- BL-010 (ubiquitous-language skill, optional)
