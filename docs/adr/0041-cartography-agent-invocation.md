# ADR 0041: Cartography Agent Invocation via SessionStart Hook

## Status
Accepted

## Context
The cartography agent needs to process pending deltas and generate documentation. The two-phase architecture (ADR-0037) separates capture from processing. The question is how the processing agent gets invoked.

## Decision
The `start:cartography` hook handler runs at session start. It reads pending delta files, acquires the `cartography-merge` file lock, then invokes the cartographer agent as a background Task subagent. If no deltas exist, it exits in <10ms. The handler is non-blocking — if the agent fails, the session continues normally.

## Consequences
- Documentation is generated at the start of each session, before the developer begins new work
- The file lock prevents concurrent sessions from double-processing the same deltas
- Agent failures are logged but never block session start
- The 30-second timeout on the SessionStart hook is sufficient for delta reading and agent dispatch
