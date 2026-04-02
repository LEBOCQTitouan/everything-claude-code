# ADR 0042: Post-Loop Element Generation

## Status
Accepted

## Context
The element registry cross-references all journeys and flows. Delta processing is per-session. The cross-reference matrix needs the complete post-delta state of journeys and flows to be accurate.

## Decision
Element generator is invoked once per `start_cartography` run, after the delta processing loop completes (journey + flow generators), before the git commit. INDEX.md is regenerated after element generation.

## Consequences
- Elements always see the complete, current journey/flow state
- Element generation failure rolls back all three layers atomically (same git reset path)
- The lock covers the entire sequence including element generation
- Elements lag journeys/flows by zero sessions (same commit)
