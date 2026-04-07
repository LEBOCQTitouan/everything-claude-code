# Spec: BL-128 Local LLM Offload via Ollama MCP Delegation

## Problem Statement

6 ECC agents (cartographer, cartography-flow-generator, cartography-journey-generator, diagram-updater, diagram-generator, convention-auditor) perform mechanical tasks — schema filling, Mermaid generation, grep-output aggregation — that consume Haiku/Sonnet API tokens unnecessarily. Local 7B-13B models (via Ollama) can handle these tasks with comparable quality at zero API cost. The MCP delegation pattern allows Claude to orchestrate while Ollama generates content, with Claude validating the output.

## Research Summary

- Ollama doesn't implement MCP natively — needs a bridge (ollama-mcp by rawveg is most actively maintained: 14 tools, TypeScript/Zod validation, hot-swap model discovery)
- MCP delegation pattern: Claude stays on Haiku for orchestration, calls `ollama_generate` MCP tool for the mechanical subtask. Claude validates output before using it.
- 7B models (Mistral-7B, Qwen2.5-7B) handle schema-fill reliably. 13B (Qwen2.5-14B) needed for Mermaid syntax (bracket sensitivity).
- Context window not a concern: MCP delegation passes only the subtask prompt, not full agent context
- Two patterns rejected: full model replacement (ANTHROPIC_BASE_URL swap — too risky, Ollama must handle full context) and SubagentStart hook intercept (7B models fail on tool use)

## Decisions Made

| # | Decision | Rationale | ADR? |
|---|----------|-----------|------|
| 1 | MCP delegation, not hook intercept | Claude validates Ollama output; graceful fallback; no Rust changes for routing | No |
| 2 | ollama-mcp bridge (rawveg/ollama-mcp) | Most maintained community bridge, 14 tools, Zod validation | No |
| 3 | Convention auditor falls back to Sonnet | Pattern recognition beyond Haiku capability; other agents fall back to Haiku | No |
| 4 | Add ecc config local-llm support | Persist user preference via `~/.ecc/config.toml`; ~30 lines Rust | No |
| 5 | Optional feature — zero degradation without Ollama | ECC must work identically for users who don't install Ollama | No |

## User Stories

### US-001: Ollama MCP server configuration

**As an** ECC user, **I want** to configure Ollama as an MCP server with local model routing, **so that** eligible agents can offload mechanical tasks to local models.

#### Acceptance Criteria

- AC-001.1: MCP server entry for `ollama-mcp` added to user-level settings (via `claude mcp add` or `.mcp.json`)
- AC-001.2: `~/.ecc/config.toml` accepts `[local_llm]` section with fields: `enabled` (bool), `provider` (string), `base_url` (string), `model_small` (string, default "mistral:7b-instruct"), `model_medium` (string, default "qwen2.5:14b-instruct")
- AC-001.3: `ecc config set local-llm.enabled true` persists the value; `ecc config set local-llm.model-small "llama3.2:8b"` overrides default
- AC-001.4: ECC health-check: on relevant commands, verify Ollama endpoint (GET `<base_url>/api/tags`, 2s timeout). Fail → WARN log + runtime flag `local_llm_available = false`
- AC-001.5: `.mcp.json` pins `ollama-mcp` to a minimum compatible version; setup guide documents tested version
- AC-001.6: `ecc config set local-llm.enabled false` immediately disables all MCP delegation globally (kill switch). Agents log which path (local/hosted) was taken per invocation at DEBUG level

#### Dependencies

- Depends on: none

### US-002: Agent MCP delegation pattern

**As an** ECC developer, **I want** a reusable pattern for agents to delegate tasks to Ollama via MCP, **so that** future agents can adopt local offloading consistently.

#### Acceptance Criteria

- AC-002.1: Agents with `local-eligible: true` in frontmatter include instructions to call `ollama_generate` MCP tool for their core mechanical task
- AC-002.2: If `ollama_generate` MCP tool is unavailable or returns error, agent falls back to doing the work itself on hosted model (graceful degradation)
- AC-002.3: Delegation pattern documented in a skill (`skills/local-llm-delegation/SKILL.md`) covering: when to use, prompt template, fallback logic, model selection
- AC-002.4: Each eligible agent explicitly documents which subtask is delegated (e.g., "schema field filling only", "full Mermaid block generation") — not the entire agent task
- AC-002.5: If delegated output fails Claude's validation (schema check, syntax check) after 2 retries, agent falls back to hosted model and logs WARN with truncated output sample
- AC-002.6: Delegation skill includes instructions for offline testing: when `ollama_generate` tool is unavailable, agent executes fallback path without error

#### Dependencies

- Depends on: US-001

### US-003: Cartography trio → local 7B

**As an** ECC operator, **I want** cartographer, cartography-flow-generator, and cartography-journey-generator to delegate schema-fill tasks to local 7B, **so that** documentation generation avoids API costs.

#### Acceptance Criteria

- AC-003.1: All three agents add `local-eligible: true` to frontmatter and include MCP delegation instructions referencing `model_small`
- AC-003.2: Output schema (section markers, Mermaid syntax, GAP annotations) is identical regardless of local/hosted path
- AC-003.3: When Ollama unavailable, agents fall back to Haiku self-execution with no quality or behavioral change

#### Dependencies

- Depends on: US-002

### US-004: Diagram agents → local 13B

**As an** ECC operator, **I want** diagram-updater and diagram-generator to delegate Mermaid generation to local 13B, **so that** diagram tasks avoid API costs.

#### Acceptance Criteria

- AC-004.1: Both agents add `local-eligible: true` to frontmatter and include MCP delegation instructions referencing `model_medium`
- AC-004.2: mmdc validation step (max 3 retries) remains unchanged regardless of generation source
- AC-004.3: When Ollama unavailable, agents fall back to Haiku self-execution

#### Dependencies

- Depends on: US-002

### US-005: Convention auditor → local 13B

**As an** ECC operator, **I want** convention-auditor to delegate grep-output aggregation to local 13B, **so that** audit aggregation avoids Sonnet API costs.

#### Acceptance Criteria

- AC-005.1: convention-auditor adds `local-eligible: true` to frontmatter and includes MCP delegation instructions referencing `model_medium`
- AC-005.2: When Ollama unavailable, agent falls back to Sonnet self-execution (not Haiku — convention audit requires mid-tier reasoning)
- AC-005.3: Finding format `[CONV-NNN]` and severity tiers unchanged

#### Dependencies

- Depends on: US-002

### US-006: Setup documentation and validation

**As an** ECC user, **I want** clear setup instructions for local LLM offloading, **so that** I can configure Ollama + MCP bridge and verify it works.

#### Acceptance Criteria

- AC-006.1: `docs/guides/local-llm-setup.md` covers: Ollama install, model pull (7B + 13B), `claude mcp add` command, verification steps, troubleshooting
- AC-006.2: CLAUDE.md Gotchas entry: "Local LLM offload (BL-128): agents with `local-eligible: true` call Ollama via MCP. Requires ollama-mcp bridge. Without Ollama, agents fall back to hosted model."
- AC-006.3: CHANGELOG entry for BL-128
- AC-006.4: `ecc validate conventions` accepts `local-eligible` as valid frontmatter field

#### Dependencies

- Depends on: US-001, US-003, US-004, US-005

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| .mcp.json or user settings | MCP config | Add ollama-mcp server entry |
| skills/local-llm-delegation/SKILL.md | skill (new) | Delegation pattern documentation |
| agents/cartographer.md | agent | Add local-eligible + MCP delegation |
| agents/cartography-flow-generator.md | agent | Add local-eligible + MCP delegation |
| agents/cartography-journey-generator.md | agent | Add local-eligible + MCP delegation |
| agents/diagram-updater.md | agent | Add local-eligible + MCP delegation |
| agents/diagram-generator.md | agent | Add local-eligible + MCP delegation |
| agents/convention-auditor.md | agent | Add local-eligible + MCP delegation |
| crates/ecc-ports/src/config.rs | Rust port | Add LocalLlmConfig struct |
| crates/ecc-infra/src/file_config_store.rs | Rust infra | Parse [local_llm] TOML section |
| crates/ecc-cli/src/config.rs | Rust CLI | ecc config set local-llm.* subcommands |
| docs/guides/local-llm-setup.md | docs (new) | Setup guide |
| CLAUDE.md | docs | Gotchas entry |
| CHANGELOG.md | docs | Entry |

## Constraints

- Optional feature — ECC must work identically without Ollama installed
- No degradation for users who don't configure local LLM
- Hard exclusion: reasoning agents never use local LLM
- `ecc validate conventions` must accept `local-eligible` frontmatter field
- ollama-mcp bridge is a user-installed dependency, not bundled with ECC

## Non-Requirements

- Full model replacement (ANTHROPIC_BASE_URL swap) — too risky for quality
- Custom MCP bridge — use existing ollama-mcp
- Local LLM for reasoning tasks — explicitly excluded
- Automatic Ollama installation — user responsibility
- Benchmarking local vs hosted quality — deferred to post-implementation evaluation

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|---|---|---|
| ecc-ports ConfigPort | Add optional field | Backward-compatible |
| ecc-infra ConfigToml | Parse new TOML section | Additive, no breaking change |
| Agent frontmatter | New optional field | Validator update needed |
| MCP server | New external dependency | Optional, graceful degradation |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|---|---|---|---|
| Setup guide | docs/guides | local-llm-setup.md | New file |
| CLAUDE.md | Gotchas | CLAUDE.md | Add local LLM note |
| CHANGELOG | Entry | CHANGELOG.md | Add BL-128 entry |

## Open Questions

None — all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | MCP delegation vs hook intercept? | MCP delegation — Claude validates output | Recommended |
| 2 | Scope: MCP + agent edits, or also Rust config? | Include ecc config local-llm support | User |
| 3 | Which MCP bridge? | ollama-mcp (rawveg) — most maintained | Recommended |
| 4 | Convention auditor fallback? | Sonnet (not Haiku) — needs mid-tier reasoning | Recommended |
| 5 | Breaking changes, security, ADR? | No to all | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Ollama MCP server configuration | 6 | none |
| US-002 | Agent MCP delegation pattern | 6 | US-001 |
| US-003 | Cartography trio → local 7B | 3 | US-002 |
| US-004 | Diagram agents → local 13B | 3 | US-002 |
| US-005 | Convention auditor → local 13B | 3 | US-002 |
| US-006 | Setup documentation | 4 | US-001-005 |

### Adversary Findings

| Dimension | Score | Key Fix |
|-----------|-------|---------|
| Ambiguity | 72→fixed | AC-002.4: explicit subtask delegation per agent |
| Edge Cases | 58→fixed | AC-002.5: validation retry + fallback on malformed output |
| Scope | 75 | Clean |
| Dependencies | 62→fixed | AC-001.5: pin ollama-mcp version |
| Testability | 55→fixed | AC-002.6: offline testing instructions |
| Decisions | 78 | Solid |
| Rollback | 48→fixed | AC-001.6: kill switch + path logging |

### Artifacts

| File Path | Content |
|-----------|---------|
| docs/specs/2026-04-07-local-llm-offload/spec.md | Full spec |
| docs/specs/2026-04-07-local-llm-offload/campaign.md | Campaign manifest |
