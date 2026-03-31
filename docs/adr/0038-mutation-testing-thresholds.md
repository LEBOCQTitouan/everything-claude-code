# ADR 0038: Mutation Testing Threshold Strategy

## Status

Accepted (aspirational — thresholds TBD after baseline measurement)

## Context

With cargo-mutants integrated (ADR 0037), the team needs threshold targets to guide test improvement. Well-tested Rust projects achieve 80-95% mutation scores. Setting thresholds before baseline measurement risks either unachievable targets or insufficient ambition.

## Decision

1. **Aspirational thresholds (to be revised after US-002/003 baseline runs):**
   - ecc-domain validation logic: 100% mutation kill rate (pure business rules where surviving mutants indicate real gaps)
   - ecc-domain remaining modules: 85%
   - ecc-app: 85%
2. **Thresholds are informational, not blocking** — CI mutation job uses `continue-on-error: true`
3. **Thresholds will be revised** after the first baseline measurement provides real data

## Consequences

- Initial baselines may show scores significantly below targets — this is expected and not a failure
- Focus test improvement on modules with lowest kill rates first
- Mutation score dashboard (`docs/audits/mutation-scores.md`) tracks progress over time
- Threshold ADR will be updated with concrete values after baseline measurement
