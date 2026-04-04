# Spec: BL-095 Thinking Budget Tuning

## Problem Statement

Extended thinking defaults to 31,999 tokens per request — the single largest cost driver at scale. Every agent invocation, regardless of complexity, burns the same thinking budget. A drift-checker comparing two strings uses the same allocation as an architect designing bounded contexts. Anthropic recommends starting at minimum and increasing incrementally, and has deprecated fixed `budget_tokens` in favor of adaptive thinking on Opus/Sonnet 4.6. ECC currently has no per-agent thinking budget control.

## Research Summary

- **Effort levels (low/medium/high/max)** are the primary thinking control in Claude Code, replacing budget_tokens. Set via `/effort` in CLI or `output_config.effort` in API.
- **Adaptive thinking** is Anthropic's recommended mode for Opus/Sonnet 4.6 — model self-determines reasoning depth. Fixed `budget_tokens` is deprecated on these models.
- **Community pattern**: route effort by task type — low for lookups/formatting, medium for standard implementation, high for complex reasoning, max for architecture/security review.
- **Anthropic recommends `medium` as default for Sonnet 4.6** for agentic coding and tool-heavy workflows, reserving high for complex reasoning.
- **Token usage compounds** across spawned agent instances — each gets its own thinking budget. Lower effort on subagents is the primary cost lever.
- **Thinking behavior is promptable** for fine-grained tuning beyond effort levels, but Anthropic warns to measure before deploying prompt-based tuning.
- **Prior art**: Goose, Roo-Code, OpenCode all converging on dynamic effort-based routing by task classification.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Add `effort` field to agent frontmatter | Per-agent thinking budget configuration. Advisory convention + hook-based enforcement via MAX_THINKING_TOKENS | Yes |
| 2 | Effort-to-tokens mapping: low=2K, medium=8K, high=16K, max=32K | Reasonable defaults following Anthropic guidance. Tunable with BL-096 data later | Yes |
| 3 | Reject `budget_tokens` as deprecated | Anthropic deprecated fixed budgets on Opus/Sonnet 4.6. Defensive validation prevents reintroduction | No |
| 4 | Cross-validate model/effort as warnings | Haiku+high is a mismatch. Warn (not error) to avoid breaking existing agents | No |
| 5 | Hook-based enforcement via MAX_THINKING_TOKENS | Claude Code doesn't natively read effort from agent frontmatter. A Rust hook reads the field and sets the env var before spawn | No |

## User Stories

### US-001: Add `effort` Frontmatter Field to Agents

**As a** ECC maintainer, **I want** each agent to declare its thinking effort level in YAML frontmatter, **so that** the intended thinking budget is documented per-agent and enforced by the effort hook.

#### Acceptance Criteria

- AC-001.1: Given an agent Markdown file, when it contains `effort: <value>` in frontmatter, then the value must be one of: `low`, `medium`, `high`, `max`
- AC-001.2: Given all 56+ existing agent files, when the effort field is added, then each agent has an effort value consistent with its role per the mapping table
- AC-001.3: Given agents with `model: haiku`, when effort is assigned, then it is `low`
- AC-001.4: Given agents with `model: sonnet`, when effort is assigned, then it is `medium` or `high`
- AC-001.5: Given agents with `model: opus`, when effort is assigned, then it is `high` or `max`

#### Dependencies

- Depends on: none

### US-002: Validate `effort` in Rust (Domain + App)

**As a** ECC developer, **I want** the `ecc validate agents` command to validate the `effort` frontmatter field, **so that** invalid effort values are caught before they ship.

#### Acceptance Criteria

- AC-002.1: Given `VALID_EFFORT_LEVELS` constant defined in `ecc-domain/src/config/validate.rs`, then it contains exactly `["low", "medium", "high", "max"]`
- AC-002.2: Given the `AgentFrontmatter` struct, when updated, then it includes an `effort: Option<String>` field
- AC-002.3: Given an agent file with `effort: invalid`, when `ecc validate agents` runs, then it reports an error with the invalid value and valid options
- AC-002.4: Given an agent file with no `effort` field, when `ecc validate agents` runs, then validation passes (field is optional)
- AC-002.5: Given an agent with `model: haiku` and `effort: high`, when `ecc validate agents` runs, then it reports a warning about model/effort mismatch
- AC-002.6: Given an agent with `model: opus` and `effort: low`, when `ecc validate agents` runs, then it reports a warning about underutilized effort

#### Dependencies

- Depends on: none

### US-003: Defensive `budget_tokens` Rejection

**As a** ECC maintainer, **I want** the validator to reject any agent declaring `budget_tokens` or `budget-tokens` in frontmatter, **so that** deprecated fixed thinking budgets cannot be reintroduced.

#### Acceptance Criteria

- AC-003.1: Given an agent file with `budget_tokens: 8000` in frontmatter, when `ecc validate agents` runs, then it reports an error: "budget_tokens is deprecated — use effort field instead"
- AC-003.2: Given an agent file with `budget-tokens: 8000` in frontmatter, when `ecc validate agents` runs, then it reports the same deprecation error (kebab-case variant)
- AC-003.3: Given an agent file without either field, when `ecc validate agents` runs, then no deprecation warning is emitted

#### Dependencies

- Depends on: none

### US-004: Hook-Based Effort Enforcement

**As a** ECC user spawning agents, **I want** a hook that reads the agent's `effort` field and sets `MAX_THINKING_TOKENS` accordingly, **so that** each agent gets an appropriate thinking budget without manual configuration.

#### Acceptance Criteria

- AC-004.1: Given a hook handler for agent spawn events, when an agent with `effort: low` is spawned, then `MAX_THINKING_TOKENS` is set to 2048
- AC-004.2: Given a hook handler, when an agent with `effort: medium` is spawned, then `MAX_THINKING_TOKENS` is set to 8192
- AC-004.3: Given a hook handler, when an agent with `effort: high` is spawned, then `MAX_THINKING_TOKENS` is set to 16384
- AC-004.4: Given a hook handler, when an agent with `effort: max` is spawned, then `MAX_THINKING_TOKENS` is set to 32768
- AC-004.5: Given a hook handler, when an agent without `effort` is spawned, then `MAX_THINKING_TOKENS` is not modified (session default applies)
- AC-004.6: Given the hook handler, when the agent's Markdown file cannot be found or parsed, then the hook exits silently (no crash, no modification)
- AC-004.7: Given the effort-to-tokens mapping, when defined in the hook, then it is centralized in a single lookup table (not scattered across conditions)
- AC-004.8: Given the hook handler, when `ECC_EFFORT_BYPASS=1` is set in the environment, then the hook exits immediately without modifying `MAX_THINKING_TOKENS` (bypass mechanism consistent with `ECC_WORKFLOW_BYPASS` pattern)
- AC-004.9: Given the hook handler, when `MAX_THINKING_TOKENS` is already set by the user in the environment, then the hook does not override it (user-set values take precedence)
- AC-004.10: Given the hook mechanism, when setting `MAX_THINKING_TOKENS`, then it outputs the env var assignment to stdout for Claude Code to apply (hooks communicate via stdout, not direct env mutation)

#### Dependencies

- Depends on: US-001 (agents must have effort field), US-002 (validation ensures valid values)

### US-005: Documentation Updates

**As a** ECC contributor, **I want** thinking effort guidance documented in performance rules, development conventions, and the glossary, **so that** new agents are created with correct effort levels from the start.

#### Acceptance Criteria

- AC-005.1: Given `rules/common/performance.md`, when updated, then it contains a "Thinking Effort Tiers" section with the effort-to-category mapping table
- AC-005.2: Given `rules/ecc/development.md`, when updated, then the "Agent Conventions" section lists `effort` as a recommended frontmatter field
- AC-005.3: Given `CLAUDE.md`, when reviewed, then the agent frontmatter description mentions `effort` alongside existing fields
- AC-005.4: Given `docs/domain/glossary.md`, when updated, then it contains entries for: Effort Level, Adaptive Thinking, Thinking Tier
- AC-005.5: Given `docs/adr/`, when two ADRs are created, then ADR 0043 covers the effort approach (hook enforcement + advisory convention) and ADR 0044 covers the effort-to-tokens mapping values

#### Dependencies

- Depends on: US-001, US-002, US-004

## Affected Modules

| Module | Layer | Change Type |
|--------|-------|-------------|
| `crates/ecc-domain/src/config/validate.rs` | Domain | Add `VALID_EFFORT_LEVELS` constant |
| `crates/ecc-domain/src/config/agent_frontmatter.rs` | Domain | Add `effort: Option<String>` field + validation |
| `crates/ecc-app/src/validate/agents.rs` | App | Validate effort field, reject budget_tokens, cross-validate model/effort |
| `crates/ecc-app/src/hook/` | App | New hook handler: effort → MAX_THINKING_TOKENS |
| `agents/*.md` (56+ files) | Content | Add `effort: <value>` to frontmatter |
| `rules/common/performance.md` | Content | Add Thinking Effort Tiers section |
| `rules/ecc/development.md` | Content | Add effort to Agent Conventions |
| `docs/domain/glossary.md` | Documentation | Add 3 terms |
| `docs/adr/` | Documentation | 2 new ADRs (0043, 0044) |
| `CLAUDE.md` | Documentation | Add effort to agent frontmatter mention |
| `CHANGELOG.md` | Documentation | Add BL-095 entry |

## Constraints

- No breaking changes to existing agent validation — `effort` is optional
- Hook must complete in <100ms (Rust binary, file read + YAML parse + env set)
- Cross-validation is warning-only (not error) for model/effort mismatch
- BL-096 cost tracking is NOT a prerequisite — proceed with defaults, tune later
- Effort values must match Claude Code's recognized levels (low/medium/high/max)
- `MAX_THINKING_TOKENS` is read by Claude Code at the session/subagent level — the hook outputs the value via stdout for Claude Code to apply. If Claude Code changes this behavior, the hook mechanism may need updating.
- Must include `ECC_EFFORT_BYPASS=1` bypass mechanism (consistent with `ECC_WORKFLOW_BYPASS` pattern) for disabling effort enforcement without removing frontmatter fields

## Non-Requirements

- BL-096 cost/token tracking for before/after measurement
- Claude Code API-level `reasoning_effort` per-spawn (use hook + env var instead)
- Effort enum in domain model (keep as validated string for now)
- Dynamic effort adjustment at runtime (static per-agent configuration only)
- Prompt-based thinking tuning (out of scope — separate concern)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| AgentFrontmatter (domain) | Extend struct | Existing validation tests need new field coverage |
| Agent validator (app) | Extend validation | New test cases for effort + budget_tokens rejection |
| Hook handler (app) | New handler | Integration tests for effort → MAX_THINKING_TOKENS mapping |
| Agent files (content) | Bulk update | `ecc validate agents` must pass with all effort fields |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| ADR | Documentation | `docs/adr/0043-effort-based-thinking-enforcement.md` | Create |
| ADR | Documentation | `docs/adr/0044-effort-to-tokens-mapping.md` | Create |
| Glossary | Documentation | `docs/domain/glossary.md` | Add 3 terms |
| Convention | Content | `rules/ecc/development.md` | Extend Agent Conventions |
| Performance | Content | `rules/common/performance.md` | Add Thinking Effort Tiers |
| Onboarding | Documentation | `CLAUDE.md` | Add effort to agent description |
| Changelog | Documentation | `CHANGELOG.md` | Add BL-095 entry |

## Open Questions

None — all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Scope | Include hook-based enforcement (wider scope) | User |
| 2 | Edge cases | Cross-validation warnings for model/effort mismatch | Recommended |
| 3 | Test strategy | 100% on Rust validation/hook, 80% on content | Recommended |
| 4 | Performance | No explicit SLA needed | Recommended |
| 5 | Security | No concerns | Recommended |
| 6 | Breaking changes | None — additive, backward-compatible | Recommended |
| 7 | Domain concepts | Add all 3 glossary terms (Effort Level, Adaptive Thinking, Thinking Tier) | User |
| 8 | ADR decisions | Two ADRs: effort approach + tokens mapping | User |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Add effort frontmatter field to agents | 5 | none |
| US-002 | Validate effort in Rust (domain + app) | 6 | none |
| US-003 | Defensive budget_tokens rejection | 3 | none |
| US-004 | Hook-based effort enforcement | 10 | US-001, US-002 |
| US-005 | Documentation updates | 5 | US-001, US-002, US-004 |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | Effort must be low/medium/high/max | US-001 |
| AC-001.2 | All agents assigned effort per role | US-001 |
| AC-001.3 | Haiku agents get effort: low | US-001 |
| AC-001.4 | Sonnet agents get medium or high | US-001 |
| AC-001.5 | Opus agents get high or max | US-001 |
| AC-002.1 | VALID_EFFORT_LEVELS constant | US-002 |
| AC-002.2 | AgentFrontmatter effort field | US-002 |
| AC-002.3 | Invalid effort reports error | US-002 |
| AC-002.4 | Missing effort passes validation | US-002 |
| AC-002.5 | Haiku+high warns mismatch | US-002 |
| AC-002.6 | Opus+low warns underutilized | US-002 |
| AC-003.1 | budget_tokens rejected (snake_case) | US-003 |
| AC-003.2 | budget-tokens rejected (kebab-case) | US-003 |
| AC-003.3 | No field = no warning | US-003 |
| AC-004.1 | effort: low → 2048 tokens | US-004 |
| AC-004.2 | effort: medium → 8192 tokens | US-004 |
| AC-004.3 | effort: high → 16384 tokens | US-004 |
| AC-004.4 | effort: max → 32768 tokens | US-004 |
| AC-004.5 | No effort = no modification | US-004 |
| AC-004.6 | Missing/unparseable file = silent exit | US-004 |
| AC-004.7 | Centralized lookup table | US-004 |
| AC-004.8 | ECC_EFFORT_BYPASS=1 bypass | US-004 |
| AC-004.9 | User-set MAX_THINKING_TOKENS takes precedence | US-004 |
| AC-004.10 | Hook outputs via stdout | US-004 |
| AC-005.1 | performance.md Thinking Effort Tiers | US-005 |
| AC-005.2 | development.md effort convention | US-005 |
| AC-005.3 | CLAUDE.md mentions effort | US-005 |
| AC-005.4 | Glossary: 3 terms | US-005 |
| AC-005.5 | Two ADRs (0043, 0044) | US-005 |

### Adversary Findings

| Dimension | R1 Score | R2 Score | Verdict | Key Rationale |
|-----------|----------|----------|---------|---------------|
| Completeness | 72 | 82 | PASS | ACs for bypass, precedence, stdout mechanism added |
| Testability | 78 | 85 | PASS | All ACs follow Given/When/Then with concrete values |
| Consistency | 75 | 80 | PASS | Effort levels uniform, bypass pattern consistent |
| Feasibility | 70 | 78 | PASS | Rust hook + YAML parse is straightforward |
| Dependencies | 58 | 72 | PASS | Constraint documents Claude Code behavior assumption |
| Rollback | 55 | 68 | PASS | ECC_EFFORT_BYPASS=1 + optional field = clear rollback |
| Scope | 76 | 80 | PASS | Well-bounded with explicit non-requirements |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| `docs/specs/2026-04-04-bl095-thinking-budget-tuning/spec.md` | Full spec + Phase Summary |
