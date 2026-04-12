---
name: progress-tracking
description: Progress tracking logic for TDD loop — TodoWrite, TaskUpdate, tasks.md status updates
origin: ECC
---

# Progress Tracking (Parent-Owned)

After regression verification passes:

1. Update TodoWrite to mark PC-NNN as complete
2. Call TaskUpdate to mark PC-N's task as completed
3. Update `tasks.md` status for the completed PC:
   - Before dispatch: update the PC line from `pending` to append `| red@<ISO 8601 timestamp>`
   - On subagent success (green_result): append `| green@<ISO 8601 timestamp>`
   - After self-evaluation (if triggered): append `| eval@<ISO 8601 timestamp>` with verdict summary (e.g., `eval@2026-04-12T16:00:00Z PASS/PASS/PASS`)
   - After regression verification passes: append `| done@<ISO 8601 timestamp>` and change `[ ]` to `[x]`
   - On subagent failure: append `| failed@<ISO 8601 timestamp> ERROR: <error summary>` — do NOT mark `[x]`
4. If the subagent failed, do NOT mark the PC as complete — TodoWrite, Task, and tasks.md remain in-progress
5. On re-entry (implement phase re-entry), tasks.md is the authoritative resume source (see Phase 0 step 6)

## Loop Completion

After ALL PCs complete successfully:

1. Run every PC's Command one final time. Record results.
2. Run the lint PC (e.g., `cargo clippy -- -D warnings`). Must pass.
3. Run the build PC (e.g., `cargo build`). Must pass.
4. Update the TodoWrite checklist: mark all PC items complete.
