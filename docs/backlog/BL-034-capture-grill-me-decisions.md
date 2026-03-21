---
id: BL-034
title: Capture grill-me decisions in work-item files
status: open
created: 2026-03-21
scope: MEDIUM
target_command: /spec-dev, /spec-fix, /spec-refactor
tags: [gsd, context, decisions, memory, gray-area]
---

## Optimized Prompt

After the grill-me interview completes in each /spec-* command, write the Q&A pairs to the work-item plan.md file (via memory-writer.sh or direct write). Each decision record includes: question asked, recommended answer, user's actual answer, and rationale for divergence (if any). This preserves gray-area decisions across sessions — currently these are lost when conversation ends. The format should be a Markdown table or structured list under a `## Grill-Me Decisions` section in the work-item file.

## Framework Source

- **GSD**: Externalizes all context to .planning/CONTEXT.md — a living document of decisions, assumptions, and open questions

## Related Backlog Items

- Depends on: BL-029 (work-item file paths)
