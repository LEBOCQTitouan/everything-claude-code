---
id: BL-029
title: Persist specs and designs as versioned file artifacts
status: open
created: 2026-03-21
scope: HIGH
target_command: /spec-dev, /spec-fix, /spec-refactor, /design
tags: [kiro, bmad, specs, persistence, artifacts, cross-session]
---

## Optimized Prompt

After adversarial review PASS in /spec-* commands, persist the full spec to `docs/specs/YYYY-MM-DD-<slug>/spec.md`. After adversarial review PASS in /design, persist the full design to `docs/specs/YYYY-MM-DD-<slug>/design.md`. Update state.json with artifact file paths so /implement can re-read them on re-entry instead of requiring the user to paste specs from a prior session. The slug is derived from the feature description (lowercase, hyphenated, max 40 chars). Files are write-once with ## Revision blocks for updates. This eliminates the "spec not found in conversation context" problem when resuming across sessions.

## Framework Source

- **Kiro**: requirements.md, design.md, tasks.md as first-class versioned artifacts
- **BMAD**: Artifacts travel with the work — PRDs, architecture docs, stories are files in the repo

## Related Backlog Items

- Enables: BL-030 (tasks.md), BL-031 (fresh context per TDD task), BL-034 (grill-me decisions)
