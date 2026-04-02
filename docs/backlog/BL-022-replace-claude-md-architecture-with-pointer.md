---
id: BL-022
title: Replace CLAUDE.md architecture block with pointer
tier: 5
scope: LOW
target: direct edit
status: "implemented"
created: 2026-03-20
file: CLAUDE.md
---

## Action

The ASCII architecture diagram (~10 lines) duplicates `docs/ARCHITECTURE.md`. Replace with: "Architecture: see `docs/ARCHITECTURE.md` (domain -> ports -> app -> infra -> cli)". Saves ~10 lines. Combined with BL-021, CLAUDE.md drops from 116 to ~80 lines.
