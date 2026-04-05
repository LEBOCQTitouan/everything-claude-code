# ADR 0052: Move Cartography Delta Processing from Hook to Doc-Orchestrator

## Status

Accepted (2026-04-06)

## Context

The `start:cartography` SessionStart hook was designed to process pending delta files by spawning a `claude --agent cartographer` subprocess. This failed silently because:

1. `CLAUDE_PROJECT_DIR` environment variable is not available during hook execution
2. The 30-second async timeout is insufficient for AI agent invocation
3. Spawning a Claude subprocess from within a hook handler violates the hook contract (hooks should be lightweight)

Delta files accumulated in `.claude/cartography/` indefinitely with no user visibility.

## Decision

Move cartography delta processing from the SessionStart hook to the doc-orchestrator pipeline (`/doc-suite` command):

- **`stop:cartography` hook** (unchanged): Writes delta JSON files at session end
- **`start:cartography` hook** (thinned): Counts pending deltas, prints a reminder pointing to `/doc-suite --phase=cartography`
- **Doc-orchestrator Phase 1.5**: Processes deltas by dispatching the cartographer agent natively within a Claude session, then commits and archives

This leverages the fact that the cartographer agent requires a Claude session to run, and the doc-orchestrator already runs inside one.

## Consequences

- Cartography docs are generated alongside all other documentation via `/doc-suite`
- Session start is faster (<100ms stat-walk vs. silent 30s timeout)
- Users have explicit visibility and control over when deltas are processed
- The cartographer agent definition was updated to return JSON envelopes instead of performing git operations directly
