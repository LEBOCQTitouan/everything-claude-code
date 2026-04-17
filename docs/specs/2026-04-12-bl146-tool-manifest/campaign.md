# Campaign Manifest

## Artifacts

| Artifact | Path | Status |
|----------|------|--------|

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|

| 1 | Manifest path discovery: canonical vs overrides? | Canonical only — single file at repo root, git checkout propagates to worktrees, no override layering, trivial discovery | recommended |
| 2 | Scope: which surfaces? | Everything — agents (59) + commands (29) + teams (3) + skills (2) = 93 files, full coverage | user |
| 3 | Migration strategy? | Dual-mode during migration — validator accepts tool-set + inline, installer expands, incremental migration, reversible | recommended |
| 4 | Retire VALID_TOOLS? | Retire in US-001 as explicit AC — manifest atomic tools section replaces VALID_TOOLS, single source from day one | recommended |
| 5 | Preset composition? | Single preset + inline extension — tool-set: NAME plus optional inline tools: [...] for extras | recommended |
| 6 | Performance constraints? | None — authoring-time operation, install-time one-shot expansion, validator is already O(files*tools) linear | recommended |
| 7 | Security implications? | Manifest is authoring-time data, no runtime permission impact. Claude Code's own allow/deny rules in settings.json remain authoritative at runtime. No new attack surface. | recommended |
| 8 | Breaking changes? | None during dual-mode migration. Post-migration, inline tools remains valid forever. No public API or CLI contract changes. | recommended |
| 9 | Glossary additions? | Add: tool-set (frontmatter reference key), preset (named tool bundle), atomic tool (Claude Code tool primitive). Update CLAUDE.md gotchas. | recommended |
| 10 | ADR needed? | Yes — ADR for declarative tool manifest. Documents: YAML over TOML, VO not aggregate, VALID_TOOLS retirement, install-time expansion boundary, dual-mode migration. | recommended |
| 11 | Manifest path and format? | YAML at manifest/tool-manifest.yaml at repo root. Uses serde-saphyr (ECC precedent). Mirrors team.rs pattern. | recommended |
## Agent Outputs

| Agent | Phase | Summary |
|-------|-------|---------|

## Commit Trail

| SHA | Message |
|-----|---------|
