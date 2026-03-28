# 0020. /audit-web Independent from /audit-full

Date: 2026-03-28

## Status

Accepted

## Context

ECC has `/audit-full` which orchestrates parallel domain-specific audits of internal code health (architecture, code quality, conventions, documentation, error handling, evolution, observability, security, tests). A new `/audit-web` command scans the external web for upgrade opportunities using a 4-phase pipeline (INVENTORY, LANDSCAPE SCAN, EVALUATE, SYNTHESIZE) and outputs findings in Technology Radar format.

The question is whether to add web scanning as a new domain within `/audit-full` or keep it as an independent command.

Two integration strategies were considered:

1. **Domain inside /audit-full**: Add web scanning as one of the parallel agents launched by `/audit-full`. A `--include-web` flag or a new domain slot would trigger Phase 2 parallel web searches alongside code audits.
2. **Independent command /audit-web**: Keep web scanning entirely separate — users invoke it explicitly, results are persisted independently, and no coupling to the internal code audit pipeline exists.

## Decision

Keep `/audit-web` independent from `/audit-full`.

Option 1 was rejected because:

- **Fundamentally different concerns**: `/audit-full` diagnoses internal code health using static analysis, git history, and AST inspection. `/audit-web` discovers external opportunities by querying the live web. These are opposite directions of inquiry — inward vs outward.
- **Latency and cost**: Phase 2 of `/audit-web` launches 8+ parallel web search agents. Embedding this inside `/audit-full` would add significant latency and token cost to every full audit run, even when web scanning is not desired.
- **Cross-correlation model breaks**: `/audit-full` correlates findings across code domains (e.g., a hotspot in evolution audit amplifies a security finding). Web findings don't correlate with code hotspots — a new crate's release does not become more relevant because a file has high churn.
- **Cost consent gate**: `/audit-web` requires a user-facing cost consent gate after Phase 1 before launching parallel web searches. This gate has no equivalent in `/audit-full` and would create a disruptive pause in the middle of a full internal audit.

Option 2 preserves clean separation of concerns, allows each command to evolve independently, and lets users run web scanning on its own schedule (e.g., monthly) without coupling it to code health checks (e.g., pre-merge).

## Consequences

**Positive:**

- Clear mental model: `/audit-full` = internal health, `/audit-web` = external opportunities
- `/audit-full` remains fast and deterministic — no web latency or rate-limit variance
- `/audit-web` can be scheduled independently (e.g., monthly) without disrupting code health gate workflows
- Each command can be enhanced without risk of breaking the other
- Future `/audit-full --include-web` flag remains possible as a user-opt-in composition layer

**Negative:**

- Users who want complete coverage must run two commands (`/audit-full` then `/audit-web`)
- There is no automatic cross-referencing between code hotspots and web-discovered upgrade opportunities — a user must mentally correlate findings across reports
