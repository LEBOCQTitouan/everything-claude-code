---
name: graceful-exit
description: Detect high context-window usage and trigger a graceful mid-session exit with state preservation.
origin: ECC
---

# Graceful Exit

Use this skill when a Claude Code session approaches context-window limits and needs to preserve state before the window is exhausted.

## How It Works

1. The statusline hook writes context-window percentage to a runtime side-channel file via `context-persist.sh`.
2. `read-context-percentage.sh` reads the persisted percentage as a pure function (returns integer 0-100 or "unknown").
3. Downstream agents check the percentage and trigger exit procedures when thresholds are crossed.

## Components

- `statusline/context-persist.sh` -- side-channel writer (stdin JSON to runtime file, atomic write)
- `skills/graceful-exit/read-context-percentage.sh` -- pure-function reader (file to stdout)

## Usage

```bash
# Write (called by statusline hook automatically)
echo '{"context_window":{"used_percentage":85}}' | bash statusline/context-persist.sh

# Read
skills/graceful-exit/read-context-percentage.sh
# => 85
```

## Thresholds

| Level | Trigger | Action |
|-------|---------|--------|
| **75% (Warn)** | Context reaches 75% | Display "Context at XX%. Consider running /compact or finishing the current phase." and continue execution. |
| **85% (Exit)** | Context reaches 85% | Complete current logical unit, write state to campaign.md, display exit message, STOP. Never interrupt mid-wave, mid-PC, or mid-regression. |
| **95% (Hard Ceiling)** | Context reaches 95% | Immediate STOP with partial-state dump. Warning: "Context at 95% (hard ceiling). State may be incomplete. Check campaign.md and tasks.md before resuming." |

The 85% exit is independent of the 75% warning — it triggers regardless of whether a warning was previously displayed.

## State-Dump Contract

Each command must write the following before exiting at 85% or above:

### `/implement`

- Update `tasks.md` with current PC progress
- Update `campaign.md` Resumption Pointer:
  - `Current step: Phase 3 — Wave 2 complete (PC-001, PC-002 done)`
  - `Next action: Resume TDD at PC-003 (wave 3)`

### `/audit-full`

- Write completed domain results to `docs/audits/partial-<timestamp>/`
- Record in `campaign.md` Resumption Pointer:
  - `Current step: Phase 2 — domains complete: architecture, security, testing`
  - `Next action: Resume domain audits: conventions, errors, observability, documentation; then correlation + report`
- Include partial dir path in Resumption Pointer

## Exit Message

Template displayed to the user on graceful exit:

```
Context at XX%. State saved to `<campaign_path>`.
Start a new session and re-run `<command>` to continue.
After compacting, read campaign.md Resumption Pointer for re-entry context.
```

## Re-entry Guidance

- Commands read `campaign.md` Resumption Pointer on re-entry to determine where to resume
- Existing re-entry logic (`/implement` Phase 0, `/audit-full`) handles resume automatically
- Campaign re-entry orientation loads: toolchain, grill-me decisions, commit trail, resumption pointer

## Graceful Degradation

If the side-channel file is missing, unreadable, or contains invalid data, the reader returns "unknown" and callers should proceed without exit logic. The writer is best-effort and exits silently on any error.
