# ADR 0060: Declarative Tool Manifest

## Status

Accepted

## Context

ECC duplicated tool-permission lists across 93 locations (59 agents, 29
commands, 3 teams, 2 skills). Adding a new Claude Code tool required editing
the `VALID_TOOLS` constant in `ecc-domain/src/config/validate.rs` and then
touching every dependent file. Renaming a preset like `[Read, Grep, Glob]`
across all read-only analyzer agents was a 40+-file change.

This duplication caused drift and blocked cheap refactoring of the tool
vocabulary. The problem was identified during BL-140 (content drift audit) and
solved by BL-146 (Declarative Tool Manifest).

Inspiration: Claw Code's Tool Manifest Framework demonstrated that a single
YAML file declaring atomic tool identifiers and named preset bundles eliminates
maintenance burden. See `docs/research/competitor-claw-goose.md`.

## Decision

All 12 grill-me decisions from the BL-146 spec are captured here:

| # | Decision |
|---|---------|
| 1 | **Manifest path**: canonical repo-root path `manifest/tool-manifest.yaml`; git checkout propagates |
| 2 | **Scope**: all 93 surfaces — 59 agents + 29 commands + 3 teams + 2 skills |
| 3 | **Migration strategy**: dual-mode during migration; installer expands `tool-set:` to `tools:` at install time |
| 4 | **Retire VALID_TOOLS**: yes — removed from `ecc-domain::config::validate`; manifest is the new vocabulary |
| 5 | **Preset composition**: single preset reference + optional inline extension; no array syntax; no `extends:` |
| 6 | **Performance**: authoring-time only — no runtime impact |
| 7 | **Security**: zero runtime surface — manifest is authoring/install time data |
| 8 | **Breaking changes**: none during dual-mode; inline `tools:` continues to work |
| 9 | **Glossary additions**: `tool-set`, `preset`, `atomic tool` added to CLAUDE.md |
| 10 | **ADR**: this document |
| 11 | **Format and path**: YAML at `manifest/tool-manifest.yaml`; parsed via `serde-saphyr` |
| 12 | **Schema-version field**: deferred to v2 — not added in BL-146 |

### Concrete implementation

- `manifest/tool-manifest.yaml` — single source of truth for atomic tool names and named presets
- `ecc-domain::config::tool_manifest::ToolManifest` — typed value object; zero I/O
- `ecc-domain::config::tool_manifest::ToolManifestError` — typed errors for all parse/validation failures
- `ecc-domain::config::tool_manifest_resolver::resolve_effective_tools` — pure resolver; no I/O
- `ecc-app::install::global::expand_agents_tool_sets` — install-time expansion (atomic write per file)
- All validators updated to load manifest and cross-reference `tool-set:` against presets

### Install-time expansion

Agents, commands, teams, and skills may use `tool-set: <preset-name>` in
YAML frontmatter. During `ecc install`, every `tool-set:` reference is expanded
to an inline `tools: [A, B, C]` list so the Claude Code runtime sees standard
inline tool lists. Installed files never contain `tool-set:`.

## Consequences

### Positive

- Single source of truth — adding a new Claude Code tool is a one-line edit to `manifest/tool-manifest.yaml`
- Drift eliminated — validators enforce manifest references at authoring time
- Rename any preset → update `manifest/tool-manifest.yaml` + run `ecc validate`
- Claude Code runtime unaffected — always sees expanded inline `tools:` lists

### Negative

- Authors must know about `manifest/tool-manifest.yaml` (mitigated by authoring guide)
- Preset composition (`extends:`) not supported in v1 — deferred to v2
- Array `tool-set:` syntax not supported — single string only

### References

- BL-146: feature backlog item for declarative tool manifest
- BL-140: content drift audit that identified the 93-location duplication
- `docs/research/competitor-claw-goose.md`: Claw Code Tool Manifest Framework
- `docs/tool-manifest-authoring.md`: authoring guide for adding tools and presets
