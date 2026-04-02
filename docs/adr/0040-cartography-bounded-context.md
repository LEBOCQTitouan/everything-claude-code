# ADR 0040: Cartography as Independent Bounded Context

## Status
Accepted

## Context
The cartography domain model (SessionDelta, CartographyManifest, merge algorithm, validation, staleness, coverage) needs a home in the domain layer. It could be part of the existing `session` or `workflow` bounded contexts, or independent.

## Decision
Cartography is an independent bounded context in `ecc-domain/src/cartography/` with zero imports from `workflow`, `session`, `hook_runtime`, or `detection` modules. The `session_id` field in `SessionDelta` is a plain `String`, not a type from the session context. The app layer bridges environment variables to domain types.

## Consequences
- Clean separation — cartography can be developed, tested, and evolved independently
- No coupling to workflow phase transitions or session lifecycle
- The hook handler in the app layer is responsible for translating between environment context and domain types
- If cartography later needs to reason about workflow state, an anti-corruption layer in the app layer handles the mapping
