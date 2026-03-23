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

## Graceful Degradation

If the side-channel file is missing, unreadable, or contains invalid data, the reader returns "unknown" and callers should proceed without exit logic. The writer is best-effort and exits silently on any error.
