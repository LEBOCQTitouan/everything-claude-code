# Campaign Manifest

## Artifacts

| Artifact | Path | Status |
|----------|------|--------|

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|

| 1 | Scope: include CLAUDE.md for new repos? | Yes — full bootstrapping: PRD + arch + ADR + initial CLAUDE.md for new repos | user |
| 2 | Output path for foundation docs | Merge approach: use docs/foundation/ for generated drafts, then merge into existing docs/ARCHITECTURE.md via LLM + user review. Foundation docs serve as input, not replacement. | user |
| 3 | ADR numbering | Auto-number: detect next available ADR number. ADR-0001 only for new repos. | recommended |
| 4 | grill-me and interview-me composition | Add foundation-mode to grill-me skill (upgrade on top of current). Use full interview-me (all 8 stages, user skips irrelevant). Compose both. | user |
| 5 | Adversarial review approach | Reuse spec-adversary with modified Task prompt. Dimensions: completeness, consistency, feasibility, ambiguity, scope. | recommended |
| 6 | Workflow state machine integration | Full state machine: use ecc-workflow init/transition. Provides re-entry support and phase gates. | user |
| 7 | Domain concepts glossary | Add foundation document + codebase-analysis phase to CLAUDE.md glossary | recommended |
| 8 | ADR for grill-me foundation-mode | Yes — ADR documenting why grill-me was extended with foundation-mode rather than creating a new skill | recommended |
## Agent Outputs

| Agent | Phase | Summary |
|-------|-------|---------|

## Commit Trail

| SHA | Message |
|-----|---------|
