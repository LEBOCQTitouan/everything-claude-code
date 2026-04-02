# ADR 0044: INDEX.md Full Regeneration

## Status
Accepted

## Context
INDEX.md is a cross-reference matrix (elements × journeys × flows). It could be delta-merged like other cartography files, or fully regenerated each time.

## Decision
INDEX.md is fully regenerated (not delta-merged) on each cartography run. It is a computed view, not an authored document. The entire file is replaced with the current matrix state.

## Consequences
- Deterministic — always reflects the current element/journey/flow state
- No stale entries can accumulate (the whole file is rewritten)
- Contradicts the general delta-merge principle but is correct for computed views
- Manual additions to INDEX.md will be lost (documented as expected behavior)
