# Campaign Manifest

## Artifacts

| Artifact | Path | Status |
|----------|------|--------|

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|

| 1 | Inline vs subagent evaluation | Lightweight read-only subagent per PC. Consistent with agent-per-concern pattern. | recommended |
| 2 | Evaluation trigger frequency | Conditional: evaluate when fix_round_count > 0, integration/e2e type, or wave boundary. Skip clean unit PCs. | recommended |
| 3 | Spec achievability trigger | Every triggered evaluation — run achievability whenever evaluation triggers (fix rounds > 0, integration/e2e, wave boundary) | user |
| 4 | Escalation fatigue: FAIL-only + 3-WARN auto-escalate | Accepted as recommended | recommended |
| 5 | Fix-round ordering: evaluation runs AFTER fix-round budget resolves (PC passes or skipped) | Accepted as recommended | recommended |
| 6 | Team manifest: add pc-evaluator agent entry to implement-team.md | Accepted as recommended | recommended |
| 7 | Tasks.md status trail: add eval@timestamp between green and done | Accepted as recommended | recommended |
| 8 | ADR: evaluation architecture warrants ADR (subagent + conditional triggers) | Accepted as recommended | recommended |
## Agent Outputs

| Agent | Phase | Summary |
|-------|-------|---------|

## Commit Trail

| SHA | Message |
|-----|---------|
