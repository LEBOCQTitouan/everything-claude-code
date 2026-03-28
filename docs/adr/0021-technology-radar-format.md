# 0021. Technology Radar Format (4+1 Quadrants)

Date: 2026-03-28

## Status

Accepted

## Context

The `/audit-web` command needs a standardized output format for classifying and communicating upgrade findings. The format must be actionable, recognizable to practitioners, and extensible to cover ECC-specific concerns.

ThoughtWorks' Technology Radar is the industry standard for structured technology assessment. It uses:

- **4 quadrants**: Techniques, Tools, Platforms, Languages & Frameworks
- **4 rings**: Adopt, Trial, Assess, Hold

Existing open-source tooling (ThoughtWorks Build Your Own Radar, Zalando's tech-radar, Qiwi's automated tech-radar generator) accepts JSON/CSV input structured around these 4 quadrants and 4 rings.

However, the standard 4-quadrant schema has no natural home for findings discovered through competitor analysis, user-request mining, or blog-miner agents — findings that represent feature gaps or product opportunities rather than technology choices. These findings are a core deliverable of `/audit-web`'s Phase 2 agents (competitor-scout, user-request-miner, blog-miner, research-scout).

Two format options were considered:

1. **Pure 4-quadrant ThoughtWorks format**: Use only the standard quadrants. Map feature opportunity findings into the closest existing quadrant (e.g., Techniques for UX patterns, Languages & Frameworks for API design).
2. **4+1 hybrid format**: Retain the 4 standard ThoughtWorks quadrants unchanged, add 1 custom "Feature Opportunities" quadrant for competitor/user-request findings.

## Decision

Use the standard 4 ThoughtWorks quadrants plus 1 custom "Feature Opportunities" quadrant. Maintain the 4 standard rings (Adopt, Trial, Assess, Hold) across all 5 quadrants.

Option 1 was rejected because forcing feature opportunities into technology quadrants produces misleading classifications. A competitor's UX pattern does not belong in "Techniques" alongside CQRS or event sourcing — the conceptual mismatch would confuse readers and degrade report quality.

Option 2 was chosen because:

- The 4 standard quadrants remain structurally intact and compatible with existing radar generators
- The 5th quadrant is clearly labeled as a custom ECC extension, avoiding any ambiguity
- Feature Opportunities is the natural home for findings from competitor-scout, user-request-miner, blog-miner, and research-scout agents
- The 4 standard rings (Adopt, Trial, Assess, Hold) apply equally well to feature opportunities — users can distinguish between features worth adopting now vs. features worth watching

## Consequences

**Positive:**

- Reports are compatible with open-source radar generators (ThoughtWorks Build Your Own Radar) for the 4 standard quadrants
- The 5th quadrant surfaces product-level findings without polluting technology classification
- The full ring schema (Adopt/Trial/Assess/Hold) applies consistently across all 5 quadrants, preserving prioritization semantics
- Feature Opportunities findings from 4 agent categories (competitor-scout, user-request-miner, blog-miner, research-scout) have a dedicated, unambiguous home

**Negative:**

- The 5th quadrant deviates from the ThoughtWorks standard — radar generator tools will not render it natively; Feature Opportunities entries must be exported or rendered separately
- Teams unfamiliar with the ECC extension may be confused by the 5th quadrant on first encounter
