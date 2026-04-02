# ADR 0037: Two-Phase Cartography Architecture

## Status
Accepted

## Context
The cartography system needs to generate documentation (user journeys, data flows) after each session. Stop hooks have 5-10 second async timeouts and cannot invoke AI agents. We need a mechanism that captures session changes without blocking completion, then processes them when time allows.

## Decision
Adopt a two-phase architecture: (1) Stop hook writes a lightweight `pending-delta-<session_id>.json` file containing changed file paths and metadata, (2) SessionStart handler in the next session processes pending deltas via the cartographer agent. Documentation updates lag by one session.

## Consequences
- Session completion is never delayed by cartography generation
- Documentation updates are one session behind (acceptable tradeoff)
- Pending delta files accumulate between sessions (bounded by auto-pruning at 30 days)
- If no subsequent session runs, deltas remain pending indefinitely (acceptable — manual `/cartography` command as fallback)
