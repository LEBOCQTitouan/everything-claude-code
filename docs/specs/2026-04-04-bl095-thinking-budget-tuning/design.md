# Design: BL-095 Thinking Budget Tuning

## Overview

Add per-agent thinking effort control via frontmatter `effort` field, Rust-based validation, a SubagentStart hook that maps effort to `MAX_THINKING_TOKENS`, and documentation updates. Five phases, each independently testable.

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/config/validate.rs` | Edit | Add `VALID_EFFORT_LEVELS` constant | AC-002.1 |
| 2 | `crates/ecc-domain/src/config/agent_frontmatter.rs` | Edit | Add `effort: Option<String>` to `AgentFrontmatter`, validate in `Validatable` impl | AC-002.2 |
| 3 | `crates/ecc-app/src/validate/agents.rs` | Edit | Validate effort value, reject `budget_tokens`/`budget-tokens`, cross-validate model/effort as warnings | AC-002.3, AC-002.4, AC-002.5, AC-002.6, AC-003.1, AC-003.2, AC-003.3 |
| 4 | `crates/ecc-app/src/hook/handlers/tier1_simple/effort_enforcement.rs` | Create | New handler: parse agent_type from stdin, read agent .md, extract effort, map to tokens, output `MAX_THINKING_TOKENS` | AC-004.1–AC-004.10 |
| 5 | `crates/ecc-app/src/hook/handlers/tier1_simple/mod.rs` | Edit | Declare + re-export `effort_enforcement` module | AC-004.* |
| 6 | `crates/ecc-app/src/hook/handlers/mod.rs` | Edit | Re-export `subagent_start_effort` from tier1_simple | AC-004.* |
| 7 | `crates/ecc-app/src/hook/mod.rs` | Edit | Add dispatch arm for `"subagent:start:effort"` | AC-004.* |
| 8 | `hooks/hooks.json` | Edit | Register `subagent:start:effort` hook under `SubagentStart` | AC-004.* |
| 9 | `agents/*.md` (57 files) | Edit | Add `effort: <value>` to each agent's frontmatter | AC-001.1–AC-001.5 |
| 10 | `rules/common/performance.md` | Edit | Add "Thinking Effort Tiers" section | AC-005.1 |
| 11 | `rules/ecc/development.md` | Edit | Add `effort` to Agent Conventions | AC-005.2 |
| 12 | `CLAUDE.md` | Edit | Mention `effort` in agent frontmatter description | AC-005.3 |
| 13 | `docs/domain/glossary.md` | Edit | Add Effort Level, Adaptive Thinking, Thinking Tier terms | AC-005.4 |
| 14 | `docs/adr/0045-effort-based-thinking-enforcement.md` | Create | ADR for effort approach | AC-005.5 |
| 15 | `docs/adr/0046-effort-to-tokens-mapping.md` | Create | ADR for mapping values | AC-005.5 |

## Agent Effort Mapping

Based on the model routing table in `rules/common/performance.md` and spec AC-001.3–AC-001.5:

| Model | Default Effort | Agents |
|-------|---------------|--------|
| haiku | low | drift-checker, diagram-generator, diagram-updater, web-radar-analyst, comms-generator |
| sonnet | medium | Most reviewers (rust, python, go, typescript, java, kotlin, cpp, csharp, shell, database), convention-auditor, error-handling-auditor, observability-auditor, test-auditor, component-auditor, doc-* (analyzer, generator, orchestrator, reporter, updater, validator), tdd-executor, tdd-guide, build-error-resolver, go-build-resolver, kotlin-build-resolver, refactor-cleaner, backlog-curator, harness-optimizer, module-summary-updater, web-scout, e2e-runner, cartographer, cartography-element-generator, cartography-flow-generator, cartography-journey-generator |
| sonnet | high | evolution-analyst, audit-challenger |
| opus | high | code-reviewer, architect-module, uncle-bob, robert, interface-designer, interviewer, arch-reviewer |
| opus | max | architect, security-reviewer, spec-adversary, solution-adversary, planner, requirements-analyst, audit-orchestrator |

## Effort-to-Tokens Lookup Table

Centralized in the hook handler (AC-004.7):

```rust
const EFFORT_TOKENS: &[(&str, u32)] = &[
    ("low", 2_048),
    ("medium", 8_192),
    ("high", 16_384),
    ("max", 32_768),
];
```

## Model/Effort Cross-Validation Matrix (warnings only)

| Model | Allowed Effort | Warning Condition |
|-------|---------------|-------------------|
| haiku | low | effort != low -> "model/effort mismatch: haiku should use low" |
| sonnet | medium, high | effort not in {medium, high} -> warning |
| opus | high, max | effort not in {high, max} -> "underutilized effort for opus" |

## Hook Architecture

The effort enforcement hook fires on `SubagentStart`. It:
1. Checks `ECC_EFFORT_BYPASS=1` -> exit immediately (AC-004.8)
2. Checks if `MAX_THINKING_TOKENS` is already set -> exit without override (AC-004.9)
3. Parses `agent_type` from stdin JSON
4. Resolves agent .md file path: `CLAUDE_PROJECT_DIR/agents/{agent_type}.md`
5. Reads + parses YAML frontmatter for `effort` field
6. If no effort field -> passthrough (AC-004.5)
7. Looks up tokens from centralized table (AC-004.7)
8. Outputs `MAX_THINKING_TOKENS={value}` to stdout (AC-004.10)
9. On any error (missing file, bad YAML) -> passthrough silently (AC-004.6)

This is a Tier 1 hook (no external tool spawning needed — it reads a local file via the FileSystem port).

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | Unit | `VALID_EFFORT_LEVELS` contains exactly `["low", "medium", "high", "max"]` | AC-002.1 | `cargo test -p ecc-domain valid_effort_levels_defined -- --exact` | PASS |
| PC-002 | Unit | `AgentFrontmatter` with `effort: Some("medium")` passes validation | AC-002.2 | `cargo test -p ecc-domain agent_with_valid_effort_passes -- --exact` | PASS |
| PC-003 | Unit | `AgentFrontmatter` with `effort: Some("invalid")` fails validation with descriptive error | AC-002.3 | `cargo test -p ecc-domain agent_with_invalid_effort_reports_error -- --exact` | PASS |
| PC-004 | Unit | `AgentFrontmatter` with `effort: None` passes validation (optional field) | AC-002.4 | `cargo test -p ecc-domain agent_without_effort_passes -- --exact` | PASS |
| PC-005 | Unit | Agent validator rejects `budget_tokens` in frontmatter with deprecation error | AC-003.1 | `cargo test -p ecc-app agents_budget_tokens_rejected -- --exact` | PASS |
| PC-006 | Unit | Agent validator rejects `budget-tokens` (kebab-case) with deprecation error | AC-003.2 | `cargo test -p ecc-app agents_budget_tokens_kebab_rejected -- --exact` | PASS |
| PC-007 | Unit | Agent without `budget_tokens` or `budget-tokens` emits no deprecation warning | AC-003.3 | `cargo test -p ecc-app agents_no_budget_tokens_no_warning -- --exact` | PASS |
| PC-008 | Unit | Agent validator warns on model/effort mismatch: haiku + high | AC-002.5 | `cargo test -p ecc-app agents_haiku_high_effort_warns -- --exact` | PASS |
| PC-009 | Unit | Agent validator warns on model/effort mismatch: opus + low | AC-002.6 | `cargo test -p ecc-app agents_opus_low_effort_warns -- --exact` | PASS |
| PC-010 | Unit | Agent with valid model/effort pair (sonnet + medium) emits no warning | AC-002.5, AC-002.6 | `cargo test -p ecc-app agents_valid_model_effort_no_warning -- --exact` | PASS |
| PC-011 | Unit | Effort hook: agent with `effort: low` -> stdout contains `MAX_THINKING_TOKENS=2048` | AC-004.1 | `cargo test -p ecc-app effort_hook_low_maps_to_2048 -- --exact` | PASS |
| PC-012 | Unit | Effort hook: `effort: medium` -> `MAX_THINKING_TOKENS=8192` | AC-004.2 | `cargo test -p ecc-app effort_hook_medium_maps_to_8192 -- --exact` | PASS |
| PC-013 | Unit | Effort hook: `effort: high` -> `MAX_THINKING_TOKENS=16384` | AC-004.3 | `cargo test -p ecc-app effort_hook_high_maps_to_16384 -- --exact` | PASS |
| PC-014 | Unit | Effort hook: `effort: max` -> `MAX_THINKING_TOKENS=32768` | AC-004.4 | `cargo test -p ecc-app effort_hook_max_maps_to_32768 -- --exact` | PASS |
| PC-015 | Unit | Effort hook: agent without effort field -> passthrough (no MAX_THINKING_TOKENS) | AC-004.5 | `cargo test -p ecc-app effort_hook_no_effort_passthrough -- --exact` | PASS |
| PC-016 | Unit | Effort hook: agent file not found -> passthrough silently | AC-004.6 | `cargo test -p ecc-app effort_hook_missing_file_passthrough -- --exact` | PASS |
| PC-017 | Unit | Effort hook: `ECC_EFFORT_BYPASS=1` -> immediate passthrough | AC-004.8 | `cargo test -p ecc-app effort_hook_bypass_passthrough -- --exact` | PASS |
| PC-018 | Unit | Effort hook: `MAX_THINKING_TOKENS` already set -> no override | AC-004.9 | `cargo test -p ecc-app effort_hook_user_override_preserved -- --exact` | PASS |
| PC-019 | Unit | Effort hook: unparseable YAML -> passthrough silently | AC-004.6 | `cargo test -p ecc-app effort_hook_bad_yaml_passthrough -- --exact` | PASS |
| PC-020 | Unit | Hook dispatch routes `"subagent:start:effort"` to handler (not Unknown hook ID) | AC-004.* | `cargo test -p ecc-app dispatches_effort_hook -- --exact` | PASS |
| PC-021 | Content | All 57 agent files have `effort:` field in frontmatter | AC-001.2 | `for f in agents/*.md; do grep -q '^effort:' "$f" \|\| echo "MISSING: $f"; done \| grep -c MISSING \| grep -q '^0$' && echo PASS \|\| echo FAIL` | PASS |
| PC-022 | Content | All haiku agents have `effort: low` | AC-001.3 | `for f in agents/*.md; do model=$(grep '^model:' "$f" \| awk '{print $2}'); effort=$(grep '^effort:' "$f" \| awk '{print $2}'); [ "$model" = "haiku" ] && [ "$effort" != "low" ] && echo "BAD: $f"; done \| grep -c BAD \| grep -q '^0$' && echo PASS \|\| echo FAIL` | PASS |
| PC-023 | Content | All sonnet agents have `effort: medium` or `effort: high` | AC-001.4 | `for f in agents/*.md; do model=$(grep '^model:' "$f" \| awk '{print $2}'); effort=$(grep '^effort:' "$f" \| awk '{print $2}'); [ "$model" = "sonnet" ] && [ "$effort" != "medium" ] && [ "$effort" != "high" ] && echo "BAD: $f"; done \| grep -c BAD \| grep -q '^0$' && echo PASS \|\| echo FAIL` | PASS |
| PC-024 | Content | All opus agents have `effort: high` or `effort: max` | AC-001.5 | `for f in agents/*.md; do model=$(grep '^model:' "$f" \| awk '{print $2}'); effort=$(grep '^effort:' "$f" \| awk '{print $2}'); [ "$model" = "opus" ] && [ "$effort" != "high" ] && [ "$effort" != "max" ] && echo "BAD: $f"; done \| grep -c BAD \| grep -q '^0$' && echo PASS \|\| echo FAIL` | PASS |
| PC-025 | Content | `hooks/hooks.json` contains `subagent:start:effort` entry | AC-004.* | `grep -q 'subagent:start:effort' hooks/hooks.json && echo PASS \|\| echo FAIL` | PASS |
| PC-026 | Content | `rules/common/performance.md` contains "Thinking Effort Tiers" section | AC-005.1 | `grep -q 'Thinking Effort Tiers' rules/common/performance.md && echo PASS \|\| echo FAIL` | PASS |
| PC-027 | Content | `rules/ecc/development.md` mentions `effort` in Agent Conventions | AC-005.2 | `grep -q 'effort' rules/ecc/development.md && echo PASS \|\| echo FAIL` | PASS |
| PC-028 | Content | `CLAUDE.md` mentions `effort` in agent frontmatter context | AC-005.3 | `grep -q 'effort' CLAUDE.md && echo PASS \|\| echo FAIL` | PASS |
| PC-029 | Content | `docs/domain/glossary.md` contains all 3 terms | AC-005.4 | `grep -q 'Effort Level' docs/domain/glossary.md && grep -q 'Adaptive Thinking' docs/domain/glossary.md && grep -q 'Thinking Tier' docs/domain/glossary.md && echo PASS \|\| echo FAIL` | PASS |
| PC-030 | Content | ADR 0045 exists (effort approach) | AC-005.5 | `test -f docs/adr/0045-effort-based-thinking-enforcement.md && echo PASS \|\| echo FAIL` | PASS |
| PC-031 | Content | ADR 0046 exists (tokens mapping) | AC-005.5 | `test -f docs/adr/0046-effort-to-tokens-mapping.md && echo PASS \|\| echo FAIL` | PASS |
| PC-032 | Unit | Effort hook rejects path traversal in agent_type (e.g., `../../etc/passwd`) with passthrough | AC-004.6 | `cargo test -p ecc-app effort_hook_rejects_path_traversal -- --exact` | PASS |
| PC-033 | Unit | Effort hook rejects unrecognized effort value (e.g., `effort: banana`) with passthrough | AC-004.6 | `cargo test -p ecc-app effort_hook_rejects_invalid_effort -- --exact` | PASS |
| PC-034 | Gate | `cargo clippy -- -D warnings` passes | All | `cargo clippy -- -D warnings` | exit 0 |
| PC-035 | Gate | `cargo test` passes (all tests) | All | `cargo test` | exit 0 |
| PC-036 | Gate | `ecc validate agents` passes with all effort fields | AC-001.*, AC-002.* | `cargo run --bin ecc -- validate agents` | exit 0 |

## TDD Order

### Phase 1: Domain Layer (PC-001 through PC-004)
Layers: [Entity]

1. **PC-001** — Add `VALID_EFFORT_LEVELS` constant to `validate.rs`. RED: test that constant exists with exact values. GREEN: add constant. Trivial change, foundational for all other validation.
2. **PC-002** — Add `effort: Option<String>` to `AgentFrontmatter` and validate in `Validatable` impl. RED: test valid effort passes. GREEN: add field + validation.
3. **PC-003** — Invalid effort value produces error. RED: test invalid effort rejected. GREEN: already implemented in PC-002's validation logic (test exercises error path).
4. **PC-004** — Missing effort (None) passes. RED: test None effort passes. GREEN: already works since field is `Option`.

### Phase 2: App Validation Layer (PC-005 through PC-010)
Layers: [UseCase]

5. **PC-005** — Reject `budget_tokens` in agent frontmatter. RED: test budget_tokens produces deprecation error. GREEN: add check in `validate_agent_file`.
6. **PC-006** — Reject `budget-tokens` (kebab-case). RED: test kebab variant rejected. GREEN: extend check from PC-005.
7. **PC-007** — No budget_tokens = no warning. RED: test clean agent produces no warning. GREEN: already satisfied by guard conditions.
8. **PC-008** — Model/effort cross-validation: haiku+high warns. RED: test haiku+high produces warning. GREEN: add cross-validation logic.
9. **PC-009** — Model/effort cross-validation: opus+low warns. RED: test opus+low warns. GREEN: extend cross-validation matrix.
10. **PC-010** — Valid model/effort pair emits no warning. RED: test sonnet+medium produces no warning. GREEN: already satisfied.

### Phase 3: Effort Hook Handler (PC-011 through PC-020)
Layers: [UseCase, Adapter]

11. **PC-011** — Hook maps `effort: low` to 2048. RED: test with in-memory agent file. GREEN: implement handler + lookup table.
12. **PC-012** — Hook maps `effort: medium` to 8192. RED/GREEN: exercises same lookup.
13. **PC-013** — Hook maps `effort: high` to 16384.
14. **PC-014** — Hook maps `effort: max` to 32768.
15. **PC-015** — No effort field -> passthrough.
16. **PC-016** — Missing agent file -> passthrough.
17. **PC-017** — `ECC_EFFORT_BYPASS=1` -> passthrough.
18. **PC-018** — User-set `MAX_THINKING_TOKENS` preserved.
19. **PC-019** — Bad YAML -> passthrough.
20. **PC-020** — Dispatch routes `subagent:start:effort` correctly. RED: test dispatch. GREEN: add match arm.

### Phase 4: Content Updates (PC-021 through PC-025)
Layers: [Framework]

21. **PC-021** — Add `effort:` to all 57 agent files.
22. **PC-022** — Verify haiku agents use `effort: low`.
23. **PC-023** — Verify sonnet agents use `effort: medium` or `effort: high`.
24. **PC-024** — Verify opus agents use `effort: high` or `effort: max`.
25. **PC-025** — Register `subagent:start:effort` in `hooks/hooks.json`.

### Phase 5: Documentation (PC-026 through PC-031)
Layers: [Framework]

26. **PC-026** — Add "Thinking Effort Tiers" section to `performance.md`.
27. **PC-027** — Add `effort` to Agent Conventions in `development.md`.
28. **PC-028** — Mention `effort` in `CLAUDE.md` agent description.
29. **PC-029** — Add 3 glossary terms.
30. **PC-030** — Create ADR 0045.
31. **PC-031** — Create ADR 0046.

### Phase 3b: Security (PC-032, PC-033)
32. **PC-032** — Hook rejects path traversal in agent_type
33. **PC-033** — Hook rejects unrecognized effort values

### Phase 6: Gate (PC-034 through PC-036)
34. **PC-034** — `cargo clippy -- -D warnings`
35. **PC-035** — `cargo test`
36. **PC-036** — `ecc validate agents` with all new fields

## E2E Assessment

- **Touches user-facing flows?** Yes — `ecc validate agents` CLI and SubagentStart hook
- **Crosses 3+ modules end-to-end?** Yes — domain -> app validation -> hook handler -> content
- **New E2E tests needed?** No — the existing `ecc validate agents` integration test and the unit tests in hook dispatch provide sufficient coverage. The hook is tested via in-memory ports. PC-034 serves as the integration gate.

## Risks & Mitigations

- **Risk**: Claude Code SubagentStart stdin schema changes (agent_type field removed or renamed)
  - Mitigation: Silent passthrough on parse failure (AC-004.6). Hook never crashes.
- **Risk**: `MAX_THINKING_TOKENS` stdout mechanism changes in future Claude Code versions
  - Mitigation: Spec constraint documents this assumption. Bypass mechanism (AC-004.8) allows instant disable.
- **Risk**: Bulk agent file edits introduce frontmatter parse errors
  - Mitigation: PC-034 gates with `ecc validate agents` after all content changes.
- **Risk**: Path traversal via agent_type field in hook stdin (security finding MEDIUM-1)
  - Mitigation: Validate agent_type against slug regex `^[a-z0-9][a-z0-9-]*[a-z0-9]$` before path construction. Reject with passthrough on failure.

## Coverage Check

All 29 ACs covered. See Pass Conditions table for AC-to-PC mapping.

Uncovered ACs: none.

## E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| — | None | — | — | No port/adapter boundary changes | — | — |

## E2E Activation Rules

No E2E tests to activate. PC-034 (`ecc validate agents`) serves as the integration gate.

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/adr/0045-effort-based-thinking-enforcement.md` | Documentation | Create | ADR: hook-based effort enforcement + advisory convention | AC-005.5 |
| 2 | `docs/adr/0046-effort-to-tokens-mapping.md` | Documentation | Create | ADR: low=2K, medium=8K, high=16K, max=32K | AC-005.5 |
| 3 | `docs/domain/glossary.md` | Documentation | Extend | Effort Level, Adaptive Thinking, Thinking Tier | AC-005.4 |
| 4 | `rules/common/performance.md` | Content | Extend | Thinking Effort Tiers section | AC-005.1 |
| 5 | `rules/ecc/development.md` | Content | Extend | effort in Agent Conventions | AC-005.2 |
| 6 | `CLAUDE.md` | Documentation | Extend | Mention effort in agent frontmatter | AC-005.3 |
| 7 | `CHANGELOG.md` | Documentation | Extend | BL-095 entry | — |

## SOLID Assessment

**Verdict: PASS** — 2 MEDIUM recommendations:
1. MEDIUM: Move `EFFORT_TOKENS` lookup table to `ecc-domain::config::validate` (not hook handler) to keep authoritative values in the domain
2. MEDIUM: Explicitly reuse `extract_frontmatter` from ecc-domain in the hook handler (avoid DRY violation)

## Robert's Oath Check

**Verdict: CLEAN** — 2 non-blocking observations:
1. ADR numbering: spec says 0043/0044 but actual next numbers are 0045/0046. Use 0045/0046.
2. Stdout contract: add implementation comment explaining why hook outputs to stdout rather than mutating env directly.

## Security Notes

**Verdict: CLEAR** — 2 MEDIUM findings:
1. MEDIUM: Path traversal on `agent_type` → file path. Mitigate with slug regex validation before path construction.
2. MEDIUM: Validate effort value in hook handler against closed set (not just in domain validation). Reject unrecognized values in hook too.
LOW: Cap emitted token values at `MAX_ALLOWED_THINKING_TOKENS` constant.

## Rollback Plan

Reverse dependency order:
1. Revert `CHANGELOG.md` entry
2. Revert `docs/domain/glossary.md` (remove 3 terms)
3. Revert `CLAUDE.md` (remove effort mention)
4. Revert `rules/ecc/development.md` (remove effort convention)
5. Revert `rules/common/performance.md` (remove Thinking Effort Tiers)
6. Delete `docs/adr/0046-effort-to-tokens-mapping.md`
7. Delete `docs/adr/0045-effort-based-thinking-enforcement.md`
8. Revert `agents/*.md` (remove effort fields from all 57 agents)
9. Revert `hooks/hooks.json` (remove subagent:start:effort entry)
10. Delete `crates/ecc-app/src/hook/handlers/tier1_simple/effort_enforcement.rs`
11. Revert `crates/ecc-app/src/hook/` module wiring (mod.rs, handlers/mod.rs, tier1_simple/mod.rs)
12. Revert `crates/ecc-app/src/validate/agents.rs` (remove effort validation + budget_tokens rejection)
13. Revert `crates/ecc-domain/src/config/agent_frontmatter.rs` (remove effort field)
14. Revert `crates/ecc-domain/src/config/validate.rs` (remove VALID_EFFORT_LEVELS, EFFORT_TOKENS)

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID (uncle-bob) | PASS | 2 MEDIUM, 2 LOW |
| Robert | CLEAN | 2 observations |
| Security | CLEAR | 2 MEDIUM, 1 LOW |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Coverage | 88 | PASS | All 29 ACs mapped, no gaps |
| Order | 90 | PASS | Correct hexagonal dependency order |
| Fragility | 72 | PASS | Claude Code behavior assumptions documented |
| Rollback | 85 | PASS | 14-step plan + ECC_EFFORT_BYPASS kill switch |
| Architecture | 78 | PASS | EFFORT_TOKENS placement resolved |
| Blast Radius | 75 | PASS | 57 agent files gated by PC-036 |
| Missing PCs | 70 | PASS | Security PCs added (PC-032, PC-033) |
| Doc Plan | 82 | PASS | 7 doc files + 2 ADRs |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `crates/ecc-domain/src/config/validate.rs` | Modify | AC-002.1 |
| 2 | `crates/ecc-domain/src/config/agent_frontmatter.rs` | Modify | AC-002.2 |
| 3 | `crates/ecc-app/src/validate/agents.rs` | Modify | AC-002.3-6, AC-003.1-3 |
| 4 | `crates/ecc-app/src/hook/handlers/tier1_simple/effort_enforcement.rs` | Create | AC-004.1-10 |
| 5 | `crates/ecc-app/src/hook/` (3 mod files) | Modify | AC-004.* |
| 6 | `hooks/hooks.json` | Modify | AC-004.* |
| 7 | `agents/*.md` (57 files) | Modify | AC-001.1-5 |
| 8 | `rules/common/performance.md` | Modify | AC-005.1 |
| 9 | `rules/ecc/development.md` | Modify | AC-005.2 |
| 10 | `CLAUDE.md` | Modify | AC-005.3 |
| 11 | `docs/domain/glossary.md` | Modify | AC-005.4 |
| 12 | `docs/adr/0045-*.md` | Create | AC-005.5 |
| 13 | `docs/adr/0046-*.md` | Create | AC-005.5 |
| 14 | `CHANGELOG.md` | Modify | — |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| `docs/specs/2026-04-04-bl095-thinking-budget-tuning/spec.md` | Full spec |
| `docs/specs/2026-04-04-bl095-thinking-budget-tuning/design.md` | Full design + Phase Summary |
