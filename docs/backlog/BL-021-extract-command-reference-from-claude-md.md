---
id: BL-021
title: Extract command reference tables from CLAUDE.md
tier: 5
scope: LOW
target: direct edit
status: open
created: 2026-03-20
files: docs/commands-reference.md (create), CLAUDE.md (modify)
---

## Action

Move the full Audit Commands table (12 rows, ~15 lines) and Side Commands table (7 rows, ~10 lines) from CLAUDE.md to `docs/commands-reference.md`. Replace in CLAUDE.md with a single line: "Full command reference: see `docs/commands-reference.md`". This saves ~25 lines of context loaded on every session that most conversations don't need.
