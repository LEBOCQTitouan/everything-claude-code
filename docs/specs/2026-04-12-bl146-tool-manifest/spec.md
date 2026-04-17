# Spec: Declarative Tool Manifest for ECC Agents, Commands, Teams, and Skills

Source: BL-146
Scope: HIGH
Worktree: ecc-bl146-tool-manifest
Campaign: docs/specs/2026-04-12-bl146-tool-manifest/campaign.md

## Problem Statement

ECC duplicates tool-permission lists across 93 locations (59 agents, 29 commands, 3 teams, 2 skills). Adding a new Claude Code tool requires editing `crates/ecc-domain/src/config/validate.rs:21-44` (`VALID_TOOLS` constant) and then touching every dependent file that should adopt it. Renaming `[Read, Grep, Glob]` to include `Bash` across all read-only analyzer agents is a 40+-file change. The 7 audit commands repeat the same `[Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite]` verbatim. This duplication causes drift and blocks cheap refactoring of the tool vocabulary. A declarative manifest with named presets is the single-source-of-truth mechanism inspired by Claw Code's Tool Manifest Framework (see `docs/research/competitor-claw-goose.md:38-43`).

## Research Summary

- **Anthropic ecosystem:** Claude Code permissions today combine frontmatter `tools:` (authoring-time) with `allowedTools` + `permissionMode` + deny/allow rules in `.claude/settings.json` (runtime). BL-146 operates purely at authoring time — zero overlap with runtime permissioning.
- **Rust ecosystem:** `serde-yaml` is archived (2025); `serde-yml` and `serde-saphyr` are active. ECC already uses `serde-saphyr` for `team.rs`, which is the direct precedent to mirror — no new dependency required.
- **Prior art (Claw Code):** YAML tool manifest parsed via Serde into Rust closures. ECC's variant is simpler — data-only (no runtime dispatch), used at validate-time and install-time.
- **DDD pattern:** Typed error enum (`ToolManifestError`) over `Validatable<String>` preserves ubiquitous-language precision. Mirrors `TeamValidationError` pattern exactly.
- **Pitfall to avoid:** worktree-scoped manifest discovery is tempting but mirroring `resolve_state_dir` would be cargo-culting — tool manifest is immutable authoring input, not mutable per-worktree state. Canonical-only is simpler and correct.
- **Security:** zero new runtime surface. Manifest is read-only authoring data. Claude Code's own runtime deny/allow rules in `settings.json` remain authoritative.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Format: YAML (not TOML) | serde-saphyr precedent in `team.rs`; no new dependency; kebab-case alignment | Yes |
| 2 | Location: `manifest/tool-manifest.yaml` at workspace root | Separate dir signals data-not-code; not under `agents/` | Yes |
| 3 | Canonical only, no worktree overrides | Git checkout propagates to worktrees automatically | Yes |
| 4 | Scope: agents + commands + teams + skills (93 files, full coverage) | Anything less ships two sources of truth | No |
| 5 | Dual-mode migration (tool-set + inline both valid) | Incremental, reversible, safer diffs | Yes |
| 6 | Retire `VALID_TOOLS` as US-001 AC | Single source from day one, not cleanup afterthought | Yes |
| 7 | Single preset + inline extension (no array, no `extends:`) | v1 avoids cycle-detection complexity | Yes |
| 8 | `ToolManifest` is Value Object (not aggregate root) | No identity, immutable, structural equality | Yes |
| 9 | Typed `ToolManifestError` enum (not `Validatable<String>`) | Mirror `team.rs`; ubiquitous-language precision | No |
| 10 | Install-time expansion mirrors free-function `expand_agents_tracking` at `steps.rs:159` — implementation shape (trait vs free fn) deferred to design phase | Real precedent is a free function, not a trait; avoid premature refactor | No |
| 11 | `ecc-domain` zero-I/O preserved — path discovery lives in `ecc-app` | Hexagonal boundary enforced | No |
| 12 | No manifest `schema-version:` field in v1 — breaking changes require coordinated commit | Defer version field; adds complexity without a concrete v2 use-case yet | No |

## User Stories

### US-001: Manifest schema, domain model, and retirement of VALID_TOOLS

**As an** ECC maintainer, **I want** a single canonical YAML file declaring every Claude Code tool ECC can reference plus named presets bundling common combinations, **so that** the tool vocabulary lives in data rather than hardcoded in Rust.

#### Acceptance Criteria

- AC-001.1: `manifest/tool-manifest.yaml` exists with top-level `tools:` (atomic vocabulary) and `presets:` (named bundles).
- AC-001.2: `parse_tool_manifest(&str) -> Result<ToolManifest, ToolManifestError>` is a pure function in `ecc-domain`.
- AC-001.3a: `validate_tool_manifest` rejects unknown atomic tools referenced in presets (fixture: preset lists `NotATool`).
- AC-001.3b: Rejects duplicate preset keys (fixture: two presets with the same name at top level).
- AC-001.3c: Rejects duplicate atomic tool names in the `tools:` section (fixture: `Read` listed twice).
- AC-001.3d: Rejects empty presets (fixture: `readonly: []`).
- AC-001.3e: Rejects non-kebab-case preset names per regex `^[a-z][a-z0-9-]*[a-z0-9]$` — fixtures: empty string, `-foo`, `foo-`, `Foo`, `foo_bar`, `foo.bar`, `resto-fr-ça` all reject with `ToolManifestError::InvalidPresetName`.
- AC-001.4: `VALID_TOOLS: &[&str]` constant is **removed entirely** from `crates/ecc-domain/src/config/validate.rs`. Post-migration `grep -r "VALID_TOOLS" crates/` returns matches only in test code. All non-test call sites take a `&ToolManifest` parameter (or a `&HashSet<&str>` view derived from it).
- AC-001.5: Manifest `tools:` section is a superset of old `VALID_TOOLS` (zero vocabulary regression).
- AC-001.6: Manifest contains at minimum **six presets**; preset names are finalized during US-005 based on empirical clustering of existing agent tool lists. The following are recommended starting points but non-binding: `readonly-analyzer`, `readonly-analyzer-shell`, `tdd-executor`, `code-writer`, `orchestrator`, `audit-command`.
- AC-001.7: `ToolManifest` module doc-comment declares it is a Value Object.
- AC-001.8: `parse_tool_manifest` strips a leading U+FEFF BOM before handing bytes to `serde-saphyr`; BOM-prefixed manifest parses identically to one without.
- AC-001.9: Duplicate top-level YAML keys → `ToolManifestError::DuplicateTopLevelKey`, not silent last-wins. Verified via fixture with `presets:` appearing twice at the top level.
- AC-001.10: Manifest file size is bounded to 1 MB; parser rejects manifests exceeding this with `ToolManifestError::ManifestTooLarge`.

**Dependencies:** none (greenfield)

### US-002: Frontmatter parsing of `tool-set:` references

**As an** agent/command author, **I want** to write `tool-set: readonly-analyzer` instead of repeating `tools: [Read, Grep, Glob]`, **so that** one manifest edit propagates everywhere.

#### Acceptance Criteria

- AC-002.1: Agent with `tool-set: readonly-analyzer` and no `tools:` → validation passes.
- AC-002.2: `tool-set: nonexistent` → exit 1, stderr names unknown preset and file path.
- AC-002.3: Both `tool-set:` and `tools:` → effective list is the union, deduped by **exact string equality** (no case folding). If any inline tool is not already in the preset, emit a WARN (exit 0, not exit 1); warning text names each outlier tool individually.
- AC-002.4: Neither `tool-set` nor `tools` → existing "Missing required field: tools" error preserved.
- AC-002.5: Commands use `allowed-tool-set: audit-command` with same resolution rules.
- AC-002.6: `resolve_effective_tools(frontmatter, manifest) -> Result<Vec<String>, ResolveError>` is pure, zero I/O, zero panics. Returns empty-list resolution as `ResolveError::EmptyResolution` (defensive; validator catches this upstream).
- AC-002.7: `tool-set:` accepts single string only (not array). Fixture `tool-set: [a, b]` → `ResolveError::ArrayNotSupported`.
- AC-002.8: If manifest file is missing, validators emit **one** error: `"tool manifest not found at manifest/tool-manifest.yaml"` — no per-file cascade. Verified by deleting the manifest before running `validate agents` against a fixture tree.

**Dependencies:** US-001

### US-003: Validator enforces manifest references

**As an** ECC developer, **I want** every tool mentioned to exist in the manifest, **so that** typos and renames fail CI.

#### Acceptance Criteria

- AC-003.1: `tools: [Read, FakeTool]` → exit 1, stderr contains `FakeTool` and file path.
- AC-003.2: Valid preset reference → exit 0 for that file.
- AC-003.3: Manifest parse error → single clear error pointing to `manifest/tool-manifest.yaml`, no per-file cascade.
- AC-003.4: Post-migration `validate agents && conventions && teams` all exit 0 (meta-test).
- AC-003.5: `validate/teams.rs:131` escalation check resolves agent presets via manifest; WARN semantics preserved.
- AC-003.6: `resolve_tool_manifest_path(fs, cwd) -> Option<PathBuf>` lives in `ecc-app`, canonical path only.
- AC-003.7: Domain `resolve_effective_tools` never takes a path.
- AC-003.8: `validate conventions` parses `allowed-tool-set:` in command frontmatter and fails with the same diagnostic shape as `validate agents` on unknown preset (exit 1, stderr names preset + file path).

**Dependencies:** US-002

### US-004: Tracer bullet — migrate spec-adversary.md end-to-end

**As a** BL-146 implementer, **I want** one agent migrated end-to-end (parse → validate → install expand → Claude Code compatibility), **so that** the architecture is proven before bulk migration.

#### Acceptance Criteria

- AC-004.1: Migrated `agents/spec-adversary.md` → `validate agents` exit 0.
- AC-004.2: Migrated file → `validate conventions` exit 0.
- AC-004.3: Install pipeline output in `~/.claude/agents/spec-adversary.md` contains inline `tools: [Read, Grep, Glob]` (expanded).
- AC-004.4: `cargo test` zero new failures.
- AC-004.5: Pre/post installed-agent diff: byte-identical effective tools list.
- AC-004.6: Install pipeline expands `tool-set:` references into inline `tools:` in written agent files. **Implementation shape (free function vs trait-based transformer) is deferred to the design phase**; the real precedent to mirror is the free function `expand_agents_tracking` in `crates/ecc-app/src/install/global/steps.rs:159`. If design chooses a refactor to a `Transformer` trait, that refactor is tracked as a follow-up story, not bundled into US-004.

**Dependencies:** US-003

### US-005: Bulk migrate 59 agent files

#### Acceptance Criteria

- AC-005.1: Every agent under `agents/` (excluding `.templates/`) uses `tool-set:` where its list matches a preset exactly.
- AC-005.2: A new preset is justified if ≥2 distinct frontmatter files (across agents/commands/teams) would reference the identical tool list after migration. Clustering computed from **pre-migration** inline tools lists. Agents whose list does not meet the ≥2 threshold keep inline `tools:`.
- AC-005.3: All 3 validators exit 0 post-migration.
- AC-005.4: Full test suite zero new failures.
- AC-005.5: Pre/post installed diff: SHA-256 of every installed agent file matches between a pre-migration snapshot and a post-migration snapshot. Integration test runs install against two temp directories seeded from git revisions.
- AC-005.6: An agent is "covered" if its frontmatter contains `tool-set:` and no inline `tools:` field. ≥48/59 agents (≥80%) are covered; total preset count ≤10. Measured via shell one-liner against post-migration `agents/` directory.
- AC-005.7: `comms-generator.md` bare-string form normalized.
- AC-005.8: `web-scout.md` two-occurrence case handled (frontmatter only).
- AC-005.9: `ecc validate teams` stderr and stdout are byte-identical pre/post US-005 (run twice, diff output). Verifies no new false privilege-escalation warnings emerge when agents drop inline `tools:`.

**Dependencies:** US-004, **US-007** (teams validator MUST resolve presets before agents drop inline `tools:`, otherwise `collect_agent_info` at `validate/teams.rs:48` reads empty tool sets and privilege-escalation warnings fire on every team)

### US-006: Bulk migrate 29 command files

#### Acceptance Criteria

- AC-006.1: 7 audit commands use `allowed-tool-set: audit-command`.
- AC-006.2: 3 comms commands normalized from bare-string.
- AC-006.3: `create-component.md` body templates NOT touched; frontmatter only.
- AC-006.4: `implement.md` (17 tools) uses new preset or inline extension.
- AC-006.5: Validators exit 0.

**Dependencies:** US-004

### US-007: Migrate 3 team files and update team validator

#### Acceptance Criteria

- AC-007.1: Team `allowed-tools` validated against manifest atomic tools.
- AC-007.2: `teams/{audit,implement,review}-team.md` migrated.
- AC-007.3: `collect_agent_info` resolves `tool-set:` via manifest.
- AC-007.4: Tool-escalation WARN includes preset name AND missing tool.
- AC-007.5: `TeamAgent` gains `allowed_tool_set: Option<String>`.

**Dependencies:** US-004

### US-008: Migrate 2 skill files

#### Acceptance Criteria

- AC-008.1: `ecc-component-authoring` and `eval-harness` SKILL.md migrated or confirmed tool-agnostic.
- AC-008.2: Skill validator enforces manifest references if `tool-set:` present.
- AC-008.3: Skills without tool declarations remain valid.

**Dependencies:** US-004

### US-009: One-edit propagation E2E test + meta-test

#### Acceptance Criteria

- AC-009.1: Integration test `install_expands_tool_sets_from_manifest` proves propagation via fixture.
- AC-009.2: Meta-test `validate_ecc_content_against_manifest` runs all 4 validators against a read-only snapshot of the committed tree (pinned via `env!("CARGO_MANIFEST_DIR")/..`). Each validator invocation captures stderr; test asserts all four captures are empty.
- AC-009.3: Property test in `tool_manifest_resolver` asserting resolution is non-recursive: fixtures with pathological preset names (e.g., a preset named `resolver`, `::`, `self`) do not cause the resolver to recurse or stack-overflow. Pre-empts v2 `extends:` feature by proving the v1 design has no accidental recursion path.
- AC-009.4: `docs/research/competitor-claw-goose.md` claim updated from "ECC: hardcoded allowedTools" to "ECC: declarative tool manifest".

**Dependencies:** US-005, US-006, US-007, US-008

### US-010: ADR + authoring documentation

#### Acceptance Criteria

- AC-010.1: `docs/adr/NNNN-declarative-tool-manifest.md` documents all 12 decisions.
- AC-010.2: ADR cites BL-146, BL-140, `docs/research/competitor-claw-goose.md`.
- AC-010.3: `docs/tool-manifest-authoring.md` explains adding tools and presets.
- AC-010.4: CLAUDE.md gains **exactly one** Gotchas line for `tool-set`; glossary entry covers three terms (`tool-set`, `preset`, `atomic tool`). No other CLAUDE.md edits in this PR.

**Dependencies:** US-001 through US-008

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `ecc-domain::config::tool_manifest` (new) | domain | Data model + typed errors + pure validator |
| `ecc-domain::config::tool_manifest_resolver` (new) | domain | Pure resolution function |
| `ecc-domain::config::agent_frontmatter` | domain | Add `tool_set: Option<String>` |
| `ecc-domain::config::team` | domain | Add `allowed_tool_set: Option<String>` to `TeamAgent` |
| `ecc-domain::config::validate` | domain | Remove `VALID_TOOLS` entirely; refactor `check_tool_values` to take `&ToolManifest` |
| `ecc-app::validate::agents` | app | Resolve preset before tool validation |
| `ecc-app::validate::conventions` | app | Same for commands via `allowed-tool-set` |
| `ecc-app::validate::teams` | app | Cross-reference via manifest |
| `ecc-app::install::global` | app | Install-time expansion of `tool-set:` references (implementation shape deferred to design per AC-004.6; precedent: `expand_agents_tracking` at `steps.rs:159`) |
| `manifest/tool-manifest.yaml` (new) | data | Canonical vocabulary + presets |
| 93 markdown files | content | Migrated to `tool-set:` references |
| `docs/adr/NNNN-declarative-tool-manifest.md` (new) | docs | ADR |
| `docs/tool-manifest-authoring.md` (new) | docs | Authoring guide |

## Constraints

- Hexagonal: `ecc-domain` zero-I/O (hook-enforced). Path discovery in `ecc-app`.
- Zero new crate dependencies — `serde-saphyr` already vendored.
- Zero new crate edges in workspace graph.
- Claude Code compatibility: installed `~/.claude/agents/` must never contain `tool-set:`.
- Coverage gate: ≥80% function coverage (BL-135).
- Worktree: development in `.claude/worktrees/ecc-bl146-tool-manifest`.
- **Independent shippability:** any subset of US-001 through US-008 **that respects the dependency DAG** is independently shippable. Commands/teams/skills not yet migrated continue to use inline `tools:`. No story beyond the dependency DAG is load-bearing for the next.

## Non-Requirements

- Scoped/glob tool permissions (`Bash(git:*)`) — separate feature
- Preset composition via `extends:` — v2
- Array `tool-set: [a, b]` — v2
- Runtime tool dispatch logic — N/A (Claude Code is the runtime)
- Hook manifest migration — `hooks.json` has no tool lists
- MCP server manifest migration — separate concern
- Per-worktree manifest overrides — explicitly rejected

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| `FileSystem` port | None | Unchanged signature; new read path added |
| `ecc validate agents` CLI | Behavior | New failure: unknown preset or manifest parse error |
| `ecc validate conventions` CLI | Behavior | New failure: unknown `allowed-tool-set` |
| `ecc validate teams` CLI | Behavior | Cross-ref uses manifest |
| `ecc install --global` CLI | Behavior | `tool-set:` expanded in installed output |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New data file | Workspace | `manifest/tool-manifest.yaml` | Create |
| New ADR | Architecture | `docs/adr/NNNN-declarative-tool-manifest.md` | Create |
| New guide | Developer | `docs/tool-manifest-authoring.md` | Create |
| Gotchas | Project | `CLAUDE.md` | Add glossary entry |
| Research sync | Research | `docs/research/competitor-claw-goose.md` | Update claim |
| Component authoring | Skill | `skills/ecc-component-authoring/SKILL.md` | Update examples |

## Implementation DAG

```
Layer 0: US-001
Layer 1: US-002 → US-003
Layer 2: US-004 (tracer bullet)
Layer 3: US-007 (teams validator refactor — MUST precede US-005)
Layer 4: US-005 ∥ US-006 ∥ US-008
Layer 5: US-009 ∥ US-010
```

Critical path: 7 layers. **Key dependency:** US-007 now precedes US-005 because `collect_agent_info` at `validate/teams.rs:48` reads agent `tools:` directly from frontmatter — migrating agents first would cause the teams validator to see empty tool lists and emit false privilege-escalation warnings.

## Open Questions

None. 11 grill-me decisions persisted to `campaign.md`; Decision #12 (no schema-version field) added during adversarial review round 1.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Manifest path discovery? | Canonical only — repo-root path, git checkout propagates | Recommended |
| 2 | Scope (which surfaces)? | Everything — 59 agents + 29 commands + 3 teams + 2 skills = 93 files | User |
| 3 | Migration strategy? | Dual-mode during migration; installer expands | Recommended |
| 4 | Retire VALID_TOOLS? | Yes, as explicit AC in US-001 | Recommended |
| 5 | Preset composition? | Single preset + inline extension (no array, no `extends:`) | Recommended |
| 6 | Performance? | None — authoring-time only | Recommended |
| 7 | Security? | Zero runtime surface | Recommended |
| 8 | Breaking changes? | None during dual-mode | Recommended |
| 9 | Glossary additions? | `tool-set`, `preset`, `atomic tool` | Recommended |
| 10 | ADR needed? | Yes | Recommended |
| 11 | Format and path? | YAML at `manifest/tool-manifest.yaml`; serde-saphyr | Recommended |
| 12 | Schema-version field? | No — deferred to v2 | Adversary-prompted |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Manifest schema + retire VALID_TOOLS | 14 | none |
| US-002 | `tool-set:` frontmatter parsing | 8 | US-001 |
| US-003 | Validator enforces manifest references | 8 | US-002 |
| US-004 | Tracer bullet (spec-adversary.md) | 6 | US-003 |
| US-005 | Bulk migrate 59 agents | 9 | US-004, **US-007** |
| US-006 | Bulk migrate 29 commands | 5 | US-004 |
| US-007 | Migrate 3 teams + update validator | 5 | US-004 |
| US-008 | Migrate 2 skills | 3 | US-004 |
| US-009 | E2E test + meta-test | 4 | US-005–US-008 |
| US-010 | ADR + authoring docs | 4 | US-001–US-008 |

**Total: 10 user stories, 66 acceptance criteria.**

### Adversary Findings

| Round | Dimension | Score | Verdict |
|-------|-----------|-------|---------|
| 1 | Ambiguity | 65 | CONDITIONAL |
| 1 | Edge Cases | 58 | CONDITIONAL |
| 1 | Scope | 72 | PASS |
| 1 | Dependencies | 45 | FAIL |
| 1 | Testability | 62 | CONDITIONAL |
| 1 | Decisions | 82 | PASS |
| 1 | Rollback | 72 | PASS |
| 1 | **Overall** | **65** | **CONDITIONAL** |
| 2 | Ambiguity | 75 | PASS |
| 2 | Edge Cases | 88 | PASS |
| 2 | Scope | 85 | PASS |
| 2 | Dependencies | 90 | PASS |
| 2 | Testability | 86 | PASS |
| 2 | Decisions | 65→100 | PASS (after surgical fixes) |
| 2 | Rollback | 72 | PASS |
| 2 | **Overall** | **78→PASS** | **PASS after 6 surgical text fixes** |

Round-2 CONDITIONAL findings were 6 surgical text drifts from round-1 edits — all applied. Adversary: *"If all six are applied in a single commit, re-review is not required."*

### Artifacts Persisted

| File Path | Contents |
|-----------|----------|
| `docs/specs/2026-04-12-bl146-tool-manifest/spec.md` | Full spec (258 lines, 10 user stories, 66 ACs) |
| `docs/specs/2026-04-12-bl146-tool-manifest/spec-draft.md` | Pre-adversary draft |
| `docs/specs/2026-04-12-bl146-tool-manifest/campaign.md` | 11 grill-me decisions + adversary history |
