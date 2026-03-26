# 0014. Context-Aware Graceful Exit Convention

Date: 2026-03-23

## Status

Superseded by BL-060

## Context

Long-running ECC commands (/implement, /audit-full) can exhaust the context window, degrading output quality. With the 1M context window, quality degrades in the last 20%. The campaign manifest (BL-035) externalizes state to disk, making session interruption zero-cost — if the agent saves state before exiting.

PreToolUse/PostToolUse hooks do not receive context_window data. The statusline receives `context_window.used_percentage` via stdin JSON, making it the only continuous data source.

## Decision

1. **Statusline side-channel**: A separate `context-persist.sh` script writes `used_percentage` to a session-scoped temp file. Commands read it via `read-context-percentage.sh`.
2. **Two thresholds**: 75% warns (user decides), 85% mandates save-and-exit, 95% is a hard ceiling (immediate STOP).
3. **Session ID sanitization**: Strip non-alphanumeric except dash/underscore to prevent path traversal.
4. **User-private runtime directory**: `$ECC_RUNTIME_DIR` with fallback chain, chmod 700.
5. **Scope**: /implement and /audit-full only — spec/design commands are shorter-lived.
6. **Audit re-entry**: Partial domain results persist to `docs/audits/partial-<timestamp>/`. Re-entry skips completed domains.
7. **Fixed thresholds for v1**: No env var configurability. Deferred to avoid premature abstraction.

## Consequences

- Sessions exit cleanly before quality degrades
- Resume is zero-cost via campaign manifest Resumption Pointer
- Audit partial results enable domain-level resume (novel pattern)
- Pre-existing session ID vulnerability in suggest-compact.sh should be fixed
- Statusline side-channel adds one temp file write per statusline render (negligible overhead)
