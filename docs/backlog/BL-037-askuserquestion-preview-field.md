---
id: BL-037
title: AskUserQuestion preview field for architecture comparisons
status: implemented
created: 2026-03-21
scope: LOW
target_command: commands/spec-dev.md, commands/design.md
tags: [native, askuserquestion, preview, ux, architecture]
---

## Optimized Prompt

When presenting architecture alternatives during grill-me interview or design review, use AskUserQuestion with the preview field to show side-by-side comparisons. Preview content can include: Mermaid diagram source, code snippets showing different approaches, before/after state, or ASCII layout mockups. Currently zero commands use the preview field — all use label+description only. Add preview to: (1) /spec-dev grill-me when multiple architecture approaches exist, (2) /design when presenting file change alternatives, (3) any command that asks the user to choose between visual alternatives.

## Framework Source

- **Native Claude Code**: AskUserQuestion preview field renders markdown in a monospace box for visual comparison

## Related Backlog Items

- None
