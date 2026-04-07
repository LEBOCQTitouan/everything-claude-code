# Campaign Manifest

## Artifacts

| Artifact | Path | Status |
|----------|------|--------|

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|

| 1 | Scope | Full sweep, all crates. Implement as a reusable skill for any repo, not just ECC. | user |
| 2 | Mechanism | Convention enforced during implementation (code-reviewer) and flagged during audit (code-audit). Not a one-time sweep. | user |
| 3 | Delivery strategy | Skill + convention first, ECC sweep as separate follow-up | recommended |
| 4 | Pattern naming convention | # Pattern section header with PatternName [Source] format | recommended |
| 5 | Diagram eligibility | Heuristic thresholds (3+ states, 3+ branches, 3+ composed types) PLUS any code key to project comprehension, even if below thresholds | user |
| 6 | Enforcement level | HIGH finding in code-reviewer (blocking). MEDIUM in audit-code (changed files only). Backlog item for full sweep. | user |
| 7 | Breaking changes | No breaking changes — doc-comments are metadata only | recommended |
| 8 | ADR decisions | No ADR needed — convention is defined in skill file, not an architecture decision | recommended |
## Agent Outputs

| Agent | Phase | Summary |
|-------|-------|---------|

## Commit Trail

| SHA | Message |
|-----|---------|
