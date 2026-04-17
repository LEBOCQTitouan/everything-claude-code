# Solution: Declarative Tool Manifest for ECC

## Spec Reference
Concern: `dev` | Feature: BL-146 Declarative tool manifest for ECC agents | Spec: `spec.md`

## Summary

**29 file changes**, **74 pass conditions** (69 original + 5 from design-review fold-in), **71/71 AC coverage** (66 original + 5 new). 8-phase TDD order with Phase 6 sub-batched by agent family (Robert oath #4). Option A: install-time expansion is a free function `expand_agents_tool_sets` alongside existing `expand_agents_tracking`.

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| **Layer 0 — Domain model + retire VALID_TOOLS (US-001)** |
| 1 | `manifest/tool-manifest.yaml` | create | Canonical YAML: atomic `tools:` + ≥6 `presets:` | US-001, AC-001.1, .5, .6 |
| 2 | `crates/ecc-domain/src/config/tool_manifest.rs` | create | `ToolManifest` VO + typed `ToolManifestError` + parser with BOM strip + 1 MB bound + duplicate-top-level-key pre-scan + **anchor/alias rejection [S1]** + kebab regex. Doc-comment declares VO. | US-001, AC-001.2, .3a–e, .7, .8, .9, .10, **.11** |
| 3 | `crates/ecc-domain/src/config/tool_manifest_resolver.rs` | create | Pure `resolve_effective_tools(FrontmatterToolSpec, &ToolManifest) -> Result<ResolvedTools, ResolveError>`. Non-recursive. Narrow input VO [M2]. **Validates `tool-set:` value regex [S3]** before lookup. | US-002, AC-002.3, .6, .7, **.9** |
| 4 | `crates/ecc-domain/src/config/mod.rs` | edit | Export new modules | US-001 |
| 5 | `crates/ecc-domain/src/config/validate.rs` | edit | **Delete** `VALID_TOOLS` constant (lines 21-44). Refactor `check_tool_values` to take `&ToolManifest`. Update tests. | US-001, AC-001.4 |
| 6 | `crates/ecc-domain/src/config/agent_frontmatter.rs` | edit | Add `tool_set: Option<String>` | US-002 |
| 7 | `crates/ecc-domain/src/config/team.rs` | edit | Add `allowed_tool_set: Option<String>` to `TeamAgent` | US-007, AC-007.5 |
| **Layer 1 — Validator injection (US-002 app + US-003)** |
| 8 | `crates/ecc-app/src/validate/tool_manifest_path_resolver.rs` | create | **[M1 split]** Pure canonical path resolution — `resolve_tool_manifest_path(root) -> PathBuf` via `root.join("manifest/tool-manifest.yaml")`. **Fixed suffix only, no parent walk, symlink-reject [S2]** | US-003, AC-003.6, **.9** |
| 9 | `crates/ecc-app/src/validate/tool_manifest_loader.rs` | create | **[M1 split]** `load_tool_manifest(fs, path) -> Result<ToolManifest, LoadError>` — pure delegation: read → parse → map error. Single-error semantics. | US-002, AC-002.8; US-003, AC-003.3 |
| 10 | `crates/ecc-app/src/validate/mod.rs` | edit | Load manifest once in `run_validate`; inject `&ToolManifest` into validators; single stderr line on load failure, no cascade | US-003 |
| 11 | `crates/ecc-app/src/validate/agents.rs` | edit | Parse `tool_set`, build `FrontmatterToolSpec`, call resolver; emit WARN on outliers, errors named with file path | US-002, AC-002.1, .2, .3, .4, .7; US-003, AC-003.1, .2, .7 |
| 12 | `crates/ecc-app/src/validate/conventions.rs` | edit | Parse `allowed-tool-set:` for commands; resolve via manifest; feed to `check_tool_values` | US-002, AC-002.5; US-003, AC-003.8 |
| 13 | `crates/ecc-app/src/validate/teams.rs` | edit | `collect_agent_info(root, fs, manifest)` resolves presets; escalation WARN names preset+missing tool | US-003, AC-003.5; US-007, AC-007.1, .3, .4 |
| **Layer 2 — Tracer bullet (US-004)** |
| 14 | `crates/ecc-app/src/install/global/steps.rs` | edit | Free fn `expand_agents_tool_sets(fs, dest, manifest)` after `expand_agents_tracking` call. **Code comment: third transformation triggers trait refactor (BL-150) [M3]**. **Atomic write (`mktemp+fsync+rename`) + symlink-reject [S4]** | US-004, AC-004.3, .5, .6, **.7** |
| 15 | `agents/spec-adversary.md` | edit | `tools: [Read, Grep, Glob]` → `tool-set: readonly-analyzer` | US-004, AC-004.1, .2, .4, .5 |
| **Layer 3 — Teams validator + 3 team files (US-007)** — MUST precede Layer 4 |
| 16 | `teams/audit-team.md` | edit | Migrate to `allowed-tool-set:` where preset matches | US-007, AC-007.2 |
| 17 | `teams/implement-team.md` | edit | Same | US-007, AC-007.2 |
| 18 | `teams/review-team.md` | edit | Same | US-007, AC-007.2 |
| **Layer 4a — Bulk agents (US-005) — sub-batched by family [Robert O4]** |
| 19 | `agents/*reviewer*.md` (~14 files) | edit | Batch 1: language reviewers (python, rust, go, java, kotlin, csharp, cpp, typescript, shell, database, robert, component-auditor, doc-validator, design-reviewer) | US-005, AC-005.1 |
| 20 | `agents/*auditor*.md` + audit agents (~8 files) | edit | Batch 2: audit-orchestrator, evolution-analyst, test-auditor, observability-auditor, error-handling-auditor, convention-auditor, doc-reporter, audit-challenger | US-005 |
| 21 | `agents/planner*.md` + executors (~10 files) | edit | Batch 3: planner, architect, architect-module, tdd-executor, tdd-guide, qa-strategist, requirements-analyst, drift-checker, solution-adversary, spec-adversary (already tracer) | US-005 |
| 22 | `agents/*orchestrator*.md` + orchestrators (~13 files) | edit | Batch 4: doc-orchestrator, doc-analyzer, doc-generator, doc-updater, comms-generator (normalize bare-string), refactor-cleaner, build-error-resolver, go-build-resolver, kotlin-build-resolver, cartographer, cartography-flow-generator, cartography-journey-generator, cartography-element-generator | US-005, AC-005.7 |
| 23 | `agents/*specialist*.md` (~8 files) | edit | Batch 5: domain specialists — web-scout (frontmatter-only, AC-005.8), web-radar-analyst, harness-optimizer, interviewer, interface-designer, backlog-curator, diagram-generator, diagram-updater | US-005, AC-005.8 |
| 24 | `agents/*.md` remaining (~5 files) | edit | Batch 6: e2e-runner, module-summary-updater, security-reviewer, code-reviewer, uncle-bob | US-005 |
| **Layer 4b — Commands (US-006)** |
| 25 | `commands/*.md` (29 files) | edit | 7 audit commands → `allowed-tool-set: audit-command`; 3 comms normalized; `create-component.md` body untouched; `implement.md` preset+extension | US-006, AC-006.1–5 |
| **Layer 4c — Skills (US-008)** |
| 26 | `crates/ecc-app/src/validate/skills.rs` | edit | Parse `tool-set:` in SKILL.md frontmatter; resolve when present | US-008, AC-008.2, .3 |
| 27 | `skills/{ecc-component-authoring,eval-harness}/SKILL.md` | edit | Migrate or confirm tool-agnostic | US-008, AC-008.1 |
| **Layer 5 — E2E + docs (US-009, US-010)** |
| 28 | `crates/ecc-integration-tests/tests/tool_manifest_propagation.rs` | create | `install_expands_tool_sets_from_manifest`, `validate_ecc_content_against_manifest`, `install_sha256_pre_post_match`, `validate_teams_byte_identical_pre_post_us_005`, **`no_tool_set_in_installed_output` [S6]** | US-009, AC-009.1, .2, .4, **.5**; US-005, AC-005.5, .9 |
| 29 | `docs/adr/0060-declarative-tool-manifest.md`, `docs/tool-manifest-authoring.md`, `docs/research/competitor-claw-goose.md`, `CLAUDE.md` | create/edit | ADR 12 decisions, authoring guide, research sync, one Gotchas line | US-009, AC-009.4; US-010, AC-010.1–4 |

## New ACs Added by Design Review

| AC | Source | Description |
|----|--------|-------------|
| AC-001.11 | Security S1 | `parse_tool_manifest` rejects YAML `&` anchor and `*` alias indicators via pre-parse scan (billion-laughs DoS defense). Error: `ToolManifestError::YamlAnchorsNotAllowed`. |
| AC-002.9 | Security S3 | `resolve_effective_tools` rejects `tool-set:` frontmatter values not matching `^[a-z][a-z0-9-]*[a-z0-9]$` before manifest lookup. Error: `ResolveError::InvalidToolSetReference`. Closes YAML injection via malformed preset names. |
| AC-003.9 | Security S2 | `resolve_tool_manifest_path` uses fixed suffix `root.join("manifest/tool-manifest.yaml")` — no parent traversal, no symlink following. Rejects `..` in resolved path. |
| AC-004.7 | Security S4 | `expand_agents_tool_sets` rewrites via `write_atomic` (mktemp + fsync + rename). Rejects symlink targets via `fs::symlink_metadata`. |
| AC-009.5 | Security S6 | Integration test greps `~/.claude/agents/*.md` post-install for `^tool-set:` and asserts zero matches. Negative assertion ensures installer never leaks unexpanded references. |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | Parser round-trips canonical YAML | AC-001.1, .2, .5 | `cargo test -p ecc-domain tool_manifest::tests::parses_valid_manifest_with_all_presets` | PASS |
| PC-002 | unit | Module doc declares VO | AC-001.7 | `cargo test -p ecc-domain tool_manifest::tests::module_doc_declares_value_object` | PASS |
| PC-003 | unit | Unknown tool in preset rejected | AC-001.3a | `cargo test -p ecc-domain tool_manifest::tests::rejects_unknown_tool_in_preset` | PASS |
| PC-004 | unit | Duplicate preset key rejected | AC-001.3b | `cargo test -p ecc-domain tool_manifest::tests::rejects_duplicate_preset_keys` | PASS |
| PC-005 | unit | Duplicate atomic tool rejected | AC-001.3c | `cargo test -p ecc-domain tool_manifest::tests::rejects_duplicate_atomic_tools` | PASS |
| PC-006 | unit | Empty preset rejected | AC-001.3d | `cargo test -p ecc-domain tool_manifest::tests::rejects_empty_preset` | PASS |
| PC-007 | unit | Invalid preset names rejected (7 fixtures) | AC-001.3e | `cargo test -p ecc-domain tool_manifest::tests::rejects_invalid_preset_names` | PASS |
| PC-008 | unit | `VALID_TOOLS` absent from source | AC-001.4 | `cargo test -p ecc-domain tool_manifest::tests::valid_tools_constant_removed` | PASS |
| PC-009 | lint | No build references to VALID_TOOLS | AC-001.4 | `cargo build --workspace 2>&1 \| grep -c "VALID_TOOLS"` | `0` |
| PC-010 | unit | Tools is superset of legacy | AC-001.5 | `cargo test -p ecc-domain tool_manifest::tests::manifest_tools_is_superset_of_legacy` | PASS |
| PC-011 | unit | ≥6 presets in canonical manifest | AC-001.6 | `cargo test -p ecc-app tool_manifest_loader::tests::canonical_manifest_has_six_plus_presets` | PASS |
| PC-012 | unit | BOM-prefixed manifest parses | AC-001.8 | `cargo test -p ecc-domain tool_manifest::tests::bom_prefix_stripped_before_parse` | PASS |
| PC-013 | unit | Duplicate top-level keys rejected | AC-001.9 | `cargo test -p ecc-domain tool_manifest::tests::rejects_duplicate_top_level_keys` | PASS |
| PC-014 | unit | Oversized manifest rejected | AC-001.10 | `cargo test -p ecc-domain tool_manifest::tests::rejects_oversized_manifest` | PASS |
| **PC-070** | unit | YAML anchors/aliases rejected | **AC-001.11** | `cargo test -p ecc-domain tool_manifest::tests::rejects_yaml_anchors_and_aliases` | PASS |
| PC-015 | integration | Agent with tool-set validates | AC-002.1 | `cargo test -p ecc-app validate::agents::tests::tool_set_only_validates` | PASS |
| PC-016 | integration | Unknown preset named in error | AC-002.2 | `cargo test -p ecc-app validate::agents::tests::unknown_preset_names_file_and_preset` | PASS |
| PC-017 | unit | Union + WARN on outliers | AC-002.3 | `cargo test -p ecc-domain tool_manifest_resolver::tests::union_dedupes_and_warns_on_outliers` | PASS |
| PC-018 | integration | Legacy "Missing tools" preserved | AC-002.4 | `cargo test -p ecc-app validate::agents::tests::neither_field_preserves_legacy_error` | PASS |
| PC-019 | integration | Command `allowed-tool-set` validates | AC-002.5 | `cargo test -p ecc-app validate::conventions::tests::allowed_tool_set_validates` | PASS |
| PC-020 | unit | Resolver never panics (proptest) | AC-002.6 | `cargo test -p ecc-domain tool_manifest_resolver::tests::proptests::never_panics` | PASS |
| PC-021 | unit | Array `tool-set` rejected | AC-002.7 | `cargo test -p ecc-domain tool_manifest_resolver::tests::rejects_array_tool_set` | PASS |
| PC-022 | integration | Missing manifest single error | AC-002.8 | `cargo test -p ecc-app validate::tool_manifest_loader::tests::missing_manifest_single_error` | PASS |
| **PC-071** | unit | `tool-set:` value regex enforced | **AC-002.9** | `cargo test -p ecc-domain tool_manifest_resolver::tests::rejects_invalid_tool_set_value` | PASS |
| PC-023 | integration | Inline FakeTool fails | AC-003.1 | `cargo test -p ecc-app validate::agents::tests::unknown_atomic_tool_reported` | PASS |
| PC-024 | integration | Valid preset passes | AC-003.2 | `cargo test -p ecc-app validate::agents::tests::valid_preset_passes` | PASS |
| PC-025 | integration | Manifest parse error once | AC-003.3 | `cargo test -p ecc-app validate::tool_manifest_loader::tests::parse_error_once_with_path` | PASS |
| PC-026 | integration | Meta-test real tree passes | AC-003.4 | `cargo test -p ecc-integration-tests tool_manifest_propagation::validate_ecc_content_against_manifest` | PASS |
| PC-027 | integration | Teams WARN names preset+missing | AC-003.5, .007.4 | `cargo test -p ecc-app validate::teams::tests::escalation_warn_names_preset_and_missing_tool` | PASS |
| PC-028 | unit | Path resolver canonical only | AC-003.6 | `cargo test -p ecc-app validate::tool_manifest_path_resolver::tests::path_is_canonical_only` | PASS |
| PC-029 | unit | Resolver signature takes no path | AC-003.7 | `cargo build -p ecc-domain` | exit 0 |
| PC-030 | integration | Conventions validates allowed-tool-set | AC-003.8 | `cargo test -p ecc-app validate::conventions::tests::unknown_allowed_tool_set_reported` | PASS |
| **PC-072** | unit | Path resolver rejects parent walk + symlinks | **AC-003.9** | `cargo test -p ecc-app validate::tool_manifest_path_resolver::tests::rejects_parent_walk_and_symlinks` | PASS |
| PC-031 | e2e | ecc validate agents exit 0 | AC-004.1 | `cargo run -p ecc-cli --quiet -- validate agents` | exit 0 |
| PC-032 | e2e | ecc validate conventions exit 0 | AC-004.2 | `cargo run -p ecc-cli --quiet -- validate conventions` | exit 0 |
| PC-033 | integration | Install writes expanded inline tools | AC-004.3, .6 | `cargo test -p ecc-app install::global::steps::tests::expand_tool_sets_inlines_spec_adversary` | PASS |
| PC-034 | build | Full test suite clean | AC-004.4 | `cargo nextest run --workspace` | exit 0 |
| PC-035 | integration | Pre/post install byte-identical | AC-004.5 | `cargo test -p ecc-app install::global::steps::tests::pre_post_effective_tools_byte_identical` | PASS |
| **PC-073** | integration | Atomic write + symlink reject | **AC-004.7** | `cargo test -p ecc-app install::global::steps::tests::write_atomic_and_rejects_symlinks` | PASS |
| PC-036 | integration | Team validator cross-refs manifest | AC-007.1 | `cargo test -p ecc-app validate::teams::tests::team_allowed_tools_cross_ref_manifest` | PASS |
| PC-037 | e2e | 3 team files validate | AC-007.2 | `cargo run -p ecc-cli --quiet -- validate teams` | exit 0 |
| PC-038 | integration | collect_agent_info resolves tool-set | AC-007.3 | `cargo test -p ecc-app validate::teams::tests::collect_agent_info_resolves_tool_set` | PASS |
| PC-039 | unit | TeamAgent has allowed_tool_set | AC-007.5 | `cargo test -p ecc-domain config::team::tests::team_agent_has_allowed_tool_set_field` | PASS |
| PC-040 | e2e | Post-migration agents validate | AC-005.3 | `cargo run -p ecc-cli --quiet -- validate agents` | exit 0 |
| PC-041 | integration | ≥48/59 covered, ≤10 presets | AC-005.2, .6 | `cargo test -p ecc-integration-tests tool_manifest_coverage::agent_coverage_ge_48_of_59_presets_le_10` | PASS |
| PC-042 | integration | comms-generator normalized | AC-005.7 | `cargo test -p ecc-integration-tests tool_manifest_coverage::comms_generator_normalized` | PASS |
| PC-043 | integration | web-scout frontmatter-only | AC-005.8 | `cargo test -p ecc-integration-tests tool_manifest_coverage::web_scout_frontmatter_normalized` | PASS |
| PC-044 | integration | SHA-256 pre/post match | AC-005.5 | `cargo test -p ecc-integration-tests tool_manifest_propagation::install_sha256_pre_post_match` | PASS |
| PC-045 | integration | Teams validator byte-identical | AC-005.9 | `cargo test -p ecc-integration-tests tool_manifest_propagation::validate_teams_byte_identical_pre_post_us_005` | PASS |
| PC-046 | build | Full suite clean | AC-005.4 | `cargo nextest run --workspace` | exit 0 |
| PC-047 | integration | 7 audit commands use preset | AC-006.1 | `cargo test -p ecc-integration-tests tool_manifest_coverage::audit_commands_use_preset` | PASS |
| PC-048 | integration | 3 comms commands normalized | AC-006.2 | `cargo test -p ecc-integration-tests tool_manifest_coverage::comms_commands_normalized` | PASS |
| PC-049 | integration | create-component body unchanged | AC-006.3 | `cargo test -p ecc-integration-tests tool_manifest_coverage::create_component_body_unchanged` | PASS |
| PC-050 | e2e | implement.md validates | AC-006.4 | `cargo run -p ecc-cli --quiet -- validate conventions` | exit 0 |
| PC-051 | e2e | Commands + conventions pass | AC-006.5 | `cargo run -p ecc-cli --quiet -- validate commands && cargo run -p ecc-cli --quiet -- validate conventions` | exit 0 |
| PC-052 | integration | Skills migrated or agnostic | AC-008.1 | `cargo test -p ecc-integration-tests tool_manifest_coverage::skills_migrated_or_agnostic` | PASS |
| PC-053 | integration | Skill validator enforces tool-set | AC-008.2 | `cargo test -p ecc-app validate::skills::tests::skill_tool_set_enforced` | PASS |
| PC-054 | integration | Skill without tools valid | AC-008.3 | `cargo test -p ecc-app validate::skills::tests::skill_no_tools_valid` | PASS |
| PC-055 | integration | Install propagation E2E | AC-009.1 | `cargo test -p ecc-integration-tests tool_manifest_propagation::install_expands_tool_sets_from_manifest` | PASS |
| PC-056 | integration | Meta-test all 4 validators | AC-009.2 | `cargo test -p ecc-integration-tests tool_manifest_propagation::validate_ecc_content_against_manifest` | PASS |
| PC-057 | unit | Non-recursion property test | AC-009.3 | `cargo test -p ecc-domain tool_manifest_resolver::tests::proptests::pathological_names_no_recursion` | PASS |
| PC-058 | lint | Research doc claim updated | AC-009.4 | `cargo test -p ecc-integration-tests tool_manifest_docs::competitor_research_claim_updated` | PASS |
| **PC-074** | integration | No tool-set in installed output | **AC-009.5** | `cargo test -p ecc-integration-tests tool_manifest_propagation::no_tool_set_in_installed_output` | PASS |
| PC-059 | lint | ADR exists with cites | AC-010.1, .2 | `cargo test -p ecc-integration-tests tool_manifest_docs::adr_exists_with_required_cites` | PASS |
| PC-060 | lint | Authoring guide exists | AC-010.3 | `cargo test -p ecc-integration-tests tool_manifest_docs::authoring_guide_exists_with_sections` | PASS |
| PC-061 | lint | CLAUDE.md glossary entry | AC-010.4 | `cargo test -p ecc-integration-tests tool_manifest_docs::claude_md_gotcha_and_glossary` | PASS |
| PC-062 | lint | ecc-domain clippy clean | — | `cargo clippy -p ecc-domain -- -D warnings` | exit 0 |
| PC-063 | lint | Workspace clippy clean | — | `cargo clippy --workspace -- -D warnings` | exit 0 |
| PC-064 | build | Workspace builds | — | `cargo build --workspace` | exit 0 |
| PC-065 | fmt | Formatting clean | — | `cargo fmt --all -- --check` | exit 0 |
| PC-066 | e2e | validate agents | — | `cargo run -p ecc-cli --quiet -- validate agents` | exit 0 |
| PC-067 | e2e | validate commands | — | `cargo run -p ecc-cli --quiet -- validate commands` | exit 0 |
| PC-068 | e2e | validate teams | — | `cargo run -p ecc-cli --quiet -- validate teams` | exit 0 |
| PC-069 | e2e | validate conventions | — | `cargo run -p ecc-cli --quiet -- validate conventions` | exit 0 |

### Coverage Check

**All 71 ACs covered** (66 original + 5 new from design review). Bold PCs (PC-070, PC-071, PC-072, PC-073, PC-074) cover the 5 design-review-added ACs.

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | `ecc validate agents` | CLI | — | Post-migration tree validates cleanly | active | always |
| 2 | `ecc validate commands` | CLI | — | Commands with `allowed-tool-set:` validate | active | always |
| 3 | `ecc validate teams` | CLI | — | Teams cross-ref via manifest | active | always |
| 4 | `ecc validate conventions` | CLI | — | Conventions enforce manifest refs | active | always |
| 5 | `ecc install --global` (fixture) | CLI + FileSystem | `FileSystem` | Install expands `tool-set:` to inline | active | `install/global/steps.rs` modified |
| 6 | Meta-test on real tree | integration-tests | `FileSystem` | All 4 validators, empty stderr, pinned via `CARGO_MANIFEST_DIR` | active | always |
| 7 | Install byte-identity (SHA-256) | integration-tests | `FileSystem` | Pre/post installed agents hash-identical | active | `install/global/steps.rs` modified |
| 8 | No-`tool-set:` in installed output | integration-tests | `FileSystem` | Negative grep assertion | active | always |

### E2E Activation Rules

- **PC-066..PC-069** (the 4 CLI validators) run as final gates for every phase.
- **PC-055, PC-056** run in Layer 5 after all migrations land.
- **PC-044, PC-045, PC-074** run in Layer 4 and Layer 5 (bulk migration + E2E).
- **PC-033, PC-035, PC-073** run in Layer 2 (tracer bullet).

## Test Strategy (TDD order)

**Phase 1 — Domain model + retire VALID_TOOLS (US-001):**
PC-001, PC-003..PC-007, PC-012, PC-013, PC-014, PC-070 (RED) → implement `tool_manifest.rs` (GREEN) → PC-010, PC-008, PC-009 (RED) → delete VALID_TOOLS, refactor `check_tool_values` (GREEN). Boy Scout: extract BOM-strip helper if duplicated.

**Phase 2 — Resolver (US-002 domain):**
PC-017, PC-020, PC-021, PC-057, PC-071 (RED) → implement `tool_manifest_resolver.rs` with `FrontmatterToolSpec` VO (GREEN).

**Phase 3 — Validator injection (US-002 app + US-003):**
PC-022, PC-025, PC-028, PC-029, PC-072 (RED) → create path_resolver + loader split (GREEN) → PC-015, PC-016, PC-018, PC-023, PC-024, PC-019, PC-030 (RED) → extend validators (GREEN) → PC-011 (GREEN).

**Phase 4 — Tracer bullet (US-004):**
PC-033, PC-035, PC-073 (RED) → implement `expand_agents_tool_sets` with atomic write + symlink reject (GREEN) → PC-031, PC-032 (RED) → migrate `agents/spec-adversary.md` (GREEN).

**Phase 5 — Teams validator refactor (US-007) — MUST precede Phase 6:**
PC-036, PC-038, PC-039 (RED) → refactor teams validator (GREEN) → PC-027, PC-037 (RED) → migrate 3 team files (GREEN).

**Phase 6 — Bulk migration (6 sub-batches for agents + commands + skills):**
- 6a.1 Reviewers batch (14 files) → 6a.2 Auditors (8) → 6a.3 Planners (10) → 6a.4 Orchestrators (13) → 6a.5 Specialists (8) → 6a.6 Remaining (5)
- 6b Commands (29 files)
- 6c Skills (skill validator + 2 SKILL.md files)
Each sub-batch: cluster RED → migrate GREEN → CI gate.

**Phase 7 — E2E + meta-test (US-009):**
PC-055, PC-056, PC-058, PC-074 (RED) → create integration-tests (GREEN).

**Phase 8 — ADR + docs (US-010):**
PC-059, PC-060, PC-061 (RED) → create ADR 0060 + authoring guide + CLAUDE.md gotcha (GREEN).

**Final gates:** PC-062..PC-069 all PASS.

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/adr/0060-declarative-tool-manifest.md` | architecture | create | All 12 decisions + citations | US-010, AC-010.1, .2 |
| 2 | `docs/tool-manifest-authoring.md` | developer | create | "How to add a tool" + "How to add a preset" | US-010, AC-010.3 |
| 3 | `CLAUDE.md` | project | edit | Exactly one gotcha line + 3-term glossary | US-010, AC-010.4 |
| 4 | `docs/research/competitor-claw-goose.md` | research | edit | Update ECC claim | US-009, AC-009.4 |
| 5 | `CHANGELOG.md` | project | edit | feat: declarative tool manifest (BL-146); migrated 93 files; retired VALID_TOOLS | mandatory |
| 6 | `docs/domain/bounded-contexts.md` | architecture | edit | Add `ToolManifest` + `FrontmatterToolSpec` VOs under Configuration/Content Validation context | US-001 |
| 7 | `MODULE-SUMMARIES.md` | reference | edit | 4 new module summaries (tool_manifest, tool_manifest_resolver, tool_manifest_path_resolver, tool_manifest_loader) | US-001, US-003 |
| 8 | `docs/architecture/tool-manifest.md` | architecture | create | New component flowchart (Mermaid) | US-001 |

**ADRs required:** ADR 0060 covers all 12 decisions (spec Decisions #1-12 with "ADR Needed? Yes" or bundled).

## SOLID Assessment

**Uncle-Bob: NEEDS WORK → ACCEPTED** (all 3 MEDIUM findings folded into design)

- **M1 SRP (tool_manifest_loader conflation):** split into `tool_manifest_path_resolver.rs` (pure, no I/O) + `tool_manifest_loader.rs` (FileSystem port + parser delegation). `run_validate` composes.
- **M2 ISP (resolver signature):** `FrontmatterToolSpec { explicit_tools, tool_set }` narrow VO already in planner's design — accepted as-is.
- **M3 god-function (install/steps.rs):** Option A free function accepted for 2nd transformation; code comment at `expand_agents_tool_sets` documents "third transformation triggers BL-150 Transformer trait refactor".
- **L1 (co-location)** and **L2 (duplicate-key pre-scan)**: accepted as-described.

**Credit:** manifest-as-data is textbook OCP; injection-over-globals is proper DIP; non-recursive resolver + property test is correct.

## Robert's Oath Check

**CLEAN → WARNINGS + 1 VIOLATION → ACCEPTED**

- **Oath 4 VIOLATION (small releases):** Phase 6 split into **6 sub-batches** (reviewers, auditors, planners/executors, orchestrators, specialists, remaining) — each independently shippable, each its own commit, each passes CI alone.
- **Oath 1 WARNING (harmful code):** N/A — manifest is authoring-time only, Claude Code runtime never reads it (reads inline expanded tools post-install). Runtime fallback not applicable; the only failure mode is CI error before ship, which IS the safety net.
- **Oath 2/5 WARNING (refactor deferral):** **BL-150 backlog entry** to be filed as follow-up — "Extract AgentFrontmatterTransform trait when third transformation lands in `install/global/steps.rs`". Trigger condition concrete; can-kicking closed.
- **Oath 3 (proof):** CLEAN — 74 PCs, 71/71 AC coverage, property tests, SHA-256 pre/post, byte-identical validator stdout/stderr.
- **Oath 6 (productivity):** CLEAN — single-source manifest lowers friction; authors edit one YAML.

## Security Notes

**CLEAR → 4 MEDIUM + 2 LOW → ACCEPTED (5 new ACs folded)**

- **S1 MEDIUM (YAML anchor/alias DoS):** AC-001.11 rejects `&`/`*` indicators via pre-parse scan. PC-070 asserts.
- **S2 MEDIUM (path traversal via cwd):** AC-003.9 pins path to fixed suffix `root.join("manifest/tool-manifest.yaml")`; no parent walk; symlink-reject. PC-072 asserts.
- **S3 MEDIUM (YAML injection via tool-set value):** AC-002.9 validates `tool-set:` value against kebab regex before lookup. `ResolveError::InvalidToolSetReference`. PC-071 asserts.
- **S4 MEDIUM (symlink TOCTOU on install rewrite):** AC-004.7 adds `write_atomic` (mktemp + fsync + rename) + `symlink_metadata` check. PC-073 asserts.
- **S5 LOW (manifest signing/hashing):** deferred to v2 / follow-up backlog.
- **S6 LOW→adopted (no `tool-set:` negative assertion):** AC-009.5 greps installed output, PC-074 asserts zero matches.

Input validation at 5 layers: size, BOM, duplicate keys, anchors/aliases, kebab regex. DIP preserved, runtime surface zero.

## Rollback Plan

Reverse dependency order:

1. Revert Phase 8 (docs) — 4 commits
2. Revert Phase 7 (integration tests) — 1 commit
3. Revert Phase 6c (skills), 6b (commands), 6a.6..6a.1 (agents) — 8 commits
4. Revert Phase 5 (teams validator + 3 team files) — 2 commits
5. Revert Phase 4 (tracer bullet + install expansion) — 2 commits
6. Revert Phase 3 (validator injection) — 1 commit
7. Revert Phase 2 (resolver) — 1 commit
8. Revert Phase 1 (domain model + VALID_TOOLS deletion) — 2 commits

Dual-mode design means any intermediate state is shippable: commands/teams/skills not yet migrated continue to use inline `tools:`. Manifest file can be deleted and all inline fallbacks will continue working (post-Phase-1 validator requires manifest exists, so rollback Phase 1 last).

## Bounded Contexts Affected

| Context | Role | Files Modified |
|---------|------|----------------|
| Configuration / Content Validation | Entity + VO + Pure Services | `tool_manifest.rs`, `tool_manifest_resolver.rs`, `agent_frontmatter.rs`, `team.rs`, `validate.rs` |
| Install Pipeline | Use Case + Transformer | `install/global/steps.rs`, `validate/tool_manifest_path_resolver.rs`, `validate/tool_manifest_loader.rs` |

**Other domain modules:** None outside these two contexts. Zero new crate edges.
