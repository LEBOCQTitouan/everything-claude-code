---
id: BL-062
title: Display full spec/design/implement artifacts inline in terminal
status: open
created: 2026-03-26
promoted_to: ""
tags: [spec, design, implement, output, inline, terminal, ux]
scope: MEDIUM
target_command: /spec-refactor
---

## Optimized Prompt

The `/spec`, `/design`, and `/implement` pipeline commands persist their artifacts to `docs/specs/` but only partially display the content in the conversation. Users must open files to read the full spec or design. Change all three commands so that the **full document body is displayed inline in the terminal before the phase summary tables**, eliminating the need to open files.

**Affected files (detect exact paths from project):**
- `commands/spec-dev.md`, `commands/spec-fix.md`, `commands/spec-refactor.md`
- `commands/design.md`
- `commands/implement.md`

**Required behavior change:**

At the "Present and STOP" phase of each command:
1. **Display the full persisted artifact** (spec.md, design.md, or tasks.md) inline in the conversation as a fenced Markdown block
2. **Then** display the phase summary tables (as implemented by BL-048)
3. **Then** show the file path reference for future access

The artifact should be displayed in full — no truncation, no "key sections only" mode. For large specs (37 ACs) or designs (49 PCs), the full content is still shown.

**Acceptance criteria:**
- AC-1: After `/spec` completes, the full spec document body appears in conversation output before the summary tables
- AC-2: After `/design` completes, the full design document body appears in conversation output before the summary tables
- AC-3: After `/implement` completes, the full tasks.md body appears in conversation output before the summary tables
- AC-4: File path references are still shown (for later access) but are no longer the primary way users consume the output
- AC-5: No existing phase logic is altered — only the final presentation phase changes
- AC-6: Documents still persist to `docs/specs/` as before (display is additive, not a replacement for persistence)

**Out of scope:**
- Do NOT change artifact persistence logic
- Do NOT alter phase ordering or introduce new phases
- Do NOT add collapsible/expandable sections (plain Markdown only)
- Do NOT change summary table format (BL-048 owns that)

**Verification:**
1. Run `/spec` on a task and confirm the full spec.md content is visible in terminal without opening any file
2. Run `/design` and confirm the full design.md content is visible inline
3. Run `/implement` and confirm the full tasks.md content is visible inline
4. Confirm summary tables still appear after the document body
5. Confirm files are still persisted to `docs/specs/`

## Original Input

The spec/design content should be written out in terminal so that the user does not have to go look into files. Currently only partial content is shown and the user is told "See docs/specs/.../spec.md".

## Challenge Log

1. Was the document body never shown, shown then buried, or partially shown?
   - **Partially shown** — some parts appeared (e.g., summary tables) but not the full document body

2. Full doc before summary, after summary, key sections only, or replace summary?
   - **Full doc before summary** — display the complete document first, then the phase summary tables

3. Should this apply to /spec + /design only, or all pipeline commands?
   - **All pipeline commands** — /spec, /design, and /implement all display their artifacts inline

## Related Backlog Items

- BL-048: Comprehensive output summaries (implemented — adds summary tables; this entry adds the full document body display before those tables)
- BL-050: Deferred summary tables (open — complementary; summary tables appear after the inline document)
- BL-029: Persist specs as file artifacts (implemented — establishes persistence; this entry adds inline display)
