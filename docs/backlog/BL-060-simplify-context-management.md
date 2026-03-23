---
id: BL-060
title: Simplify context management — remove graceful exit, keep warnings only
status: open
scope: HIGH
target: /spec-refactor
created: 2026-03-23
---

## Problem

The graceful exit system (BL-054, BL-055, ADR-0014) over-engineers context management. Claude Code now has a built-in feature: when you accept a plan, it automatically clears context for a fresh window. This native behavior replaces most of the custom infrastructure.

## What to Do

### Remove (delete entirely)

1. **Graceful exit skill** — `skills/graceful-exit/SKILL.md` + `skills/graceful-exit/read-context-percentage.sh`
2. **Strategic compact skill** — `skills/strategic-compact/SKILL.md`
3. **ADR-0014** — `docs/adr/0014-context-aware-graceful-exit.md`
4. **Full spec directory** — `docs/specs/2026-03-23-graceful-mid-session-exit/` (spec.md, design.md, tasks.md)
5. **Graceful exit checkpoints** from `/implement` (Phase 3-7 context checks, 85% exit logic, Resumption Pointer saves)
6. **Re-entry logic** from `/audit-full` (Phase 0 resumption from campaign.md)
7. **Domain checkpoint + partial results write** from `agents/audit-orchestrator.md` (Phase 2 context check, partial dump to `docs/audits/partial-*/`)
8. **Context clear gate** from `/implement` Phase 0 step 7 (BL-054 — the "Clear context and start fresh?" prompt)
9. **Resumption Pointer** references from `skills/campaign-manifest/SKILL.md`
10. **Glossary entries** for "Graceful Exit" and "Context Checkpoint" from `docs/domain/glossary.md`

### Modify

1. **`/implement`** — After Plan Mode acceptance (ExitPlanMode), rely on Claude Code's built-in context clear on plan accept. The command should be structured so that plan acceptance naturally triggers the fresh context window. No custom compact gate needed.
2. **Keep context warnings** — Commands that currently display "context at X%" warnings should continue showing them (informational only, no exit/stop behavior).

### Keep (do not touch)

1. **Statusline** — `statusline/statusline-command.sh` and `statusline/context-persist.sh` (informational display)
2. **`docs/token-optimization.md`** — general guidance (review for stale graceful-exit references)
3. **Campaign manifest skill** — keep the skill itself, just remove Resumption Pointer section

### Archive backlog entries

- BL-054 → archived (context clear gate — replaced by native behavior)
- BL-055 → archived (graceful exit — reversed)

## Verification

- All commands still parse and execute without referencing deleted files
- No dangling skill/script references in commands, agents, or hooks
- `/implement` Plan Mode flow works with native context clear
- Context warning display still functions in commands
- `cargo test` passes (no Rust changes expected, but verify)
- `npm run lint` passes on modified Markdown

## Rationale

Claude Code's native plan-accept context clear is simpler, more reliable, and maintained upstream. Custom exit/resumption logic adds complexity without sufficient benefit — the exit behavior doesn't match user expectations, and state serialization is fragile.
