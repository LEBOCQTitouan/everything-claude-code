# Campaign Manifest

## Artifacts

| Artifact | Path | Status |
|----------|------|--------|

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|

| 1 | Compass scope? | Everything — all component types (50+ compass files) AND should transpire into every file in projects using ECC. Compass context must be available per-file, not just per-crate. | user |
| 2 | Scope boundaries? | Out of scope: GUI/TUI for compass viewing, external doc hosting, real-time collaborative editing of compass files. In scope: per-component compass files for all ECC component types + per-file tribal knowledge annotations in downstream projects. | recommended |
| 3 | Test strategy? | Validation via ecc validate + cargo doc. Coverage: compass file structure assertions (25-35 lines, required sections). Auto-repair tested via drift-then-repair roundtrip. | recommended |
| 4 | Breaking changes? | None — purely additive. New skills/agents/hooks/compass files. No existing behavior changes. | recommended |
| 5 | ADR needed? | Yes — ADR for Meta tribal knowledge integration: five-question framework, compass-not-encyclopedia principle, auto-repair tier model, periodic validation hook. | recommended |
| 6 | Glossary? | Add: compass context file (25-35 line module orientation doc), tribal knowledge extraction (five-question framework per module), auto-repair (tiered automated doc drift fix). | recommended |
## Agent Outputs

| Agent | Phase | Summary |
|-------|-------|---------|

## Commit Trail

| SHA | Message |
|-----|---------|
