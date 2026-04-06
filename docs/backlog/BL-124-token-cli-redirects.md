---
id: BL-124
title: "Token optimization wave 1 — zero-cost CLI redirects and one-liner fixes"
status: open
created: 2026-04-06
promoted_to: ""
tags: [token-optimization, agents, commands, cost]
scope: LOW
target_command: direct edit
dependencies: [BL-121]
---

## Optimized Prompt

Apply 7 zero-cost token optimization fixes from the BL-121 audit (Wave 1). Each is a direct edit to existing markdown files — no new Rust code:

1. `agents/doc-generator.md` changelog step — replace git log reimplementation with `ecc analyze changelog` call
2. `agents/evolution-analyst.md` hotspot/coupling/bus-factor steps — replace git log queries with `ecc analyze hotspots`, `ecc analyze coupling`, `ecc analyze bus-factor` calls; agent interprets output only
3. `agents/backlog-curator.md` duplicate check — replace in-context similarity scoring with `ecc backlog check-duplicates` call
4. 26 files across `commands/*.md` and `agents/*.md` — standardize verbose narrative-conventions references to single-line: `> **Narrative**: See narrative-conventions skill.`
5. All `audit-*.md` commands — make `audit-challenger` conditional: launch only if primary agent returns ≥3 findings or severity ≥ HIGH
6. `commands/spec-dev.md` — launch `requirements-analyst` and `architect` in parallel (pattern already used by spec-refactor Phase 1)

**Verification:** After each change, run `ecc validate agents commands` to confirm frontmatter/structure validity. Run the modified command once to verify behavior is preserved.

Reference: `docs/audits/token-optimization-2026-04-06.md` findings 1.3, 1.4, 1.5, 1.13, 3.2, 4.5, 4.6.

## Original Input

BL-121 audit Wave 1: redirect agents to existing CLI commands, standardize boilerplate one-liners, conditional audit-challenger, parallel spec-dev agents.

## Challenge Log

**Source:** BL-121 token optimization audit (2026-04-06). Pre-challenged during audit — all findings validated by 4-axis parallel analysis agents.
