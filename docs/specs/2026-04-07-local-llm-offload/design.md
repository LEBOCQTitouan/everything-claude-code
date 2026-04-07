# Design: BL-128 Local LLM Offload via Ollama MCP Delegation

## Spec Reference

`docs/specs/2026-04-07-local-llm-offload/spec.md` — 6 US, 25 ACs.

## File Changes

| # | File(s) | Change | Spec Ref | Rationale |
|---|---------|--------|----------|-----------|
| 1 | `crates/ecc-ports/src/config_store.rs` | Add `LocalLlmConfig` struct (enabled, provider, base_url, model_small, model_medium) as Optional field on `RawEccConfig`. All fields Optional<String> except enabled (Optional<bool>). Default: disabled. | US-001 | Ports define the shape; no I/O |
| 2 | `crates/ecc-infra/src/file_config_store.rs` | Add `LocalLlmToml` struct for `[local_llm]` TOML section. Add `From` impls between `LocalLlmToml` and `LocalLlmConfig`. | US-001 | Infra handles serialization |
| 3 | `crates/ecc-cli/src/config.rs` | Match `local-llm.enabled`, `local-llm.provider`, `local-llm.base-url`, `local-llm.model-small`, `local-llm.model-medium` in the `set` command handler. | US-001 | CLI exposes to user |
| 4 | `crates/ecc-test-support/src/*.rs` | Update `InMemoryConfigStore` to include `LocalLlmConfig` in `RawEccConfig` construction. | US-001 | Test doubles must match ports |
| 5 | `skills/local-llm-delegation/SKILL.md` | New skill documenting: when to use local delegation, prompt template for `ollama_generate`, fallback logic (2 retry → hosted), model selection (small=7B, medium=13B), per-agent subtask documentation requirement. | US-002 | Reusable pattern for future agents |
| 6 | `agents/cartographer.md` | Add `local-eligible: true` to frontmatter. Add delegation section: "For the routing/dispatch subtask, call `ollama_generate` MCP tool with model_small. If unavailable, execute the task directly." | US-003 | Schema-fill task on 7B |
| 7 | `agents/cartography-flow-generator.md` | Same pattern: `local-eligible: true` + delegation for schema-fill generation subtask. | US-003 | Schema-fill task on 7B |
| 8 | `agents/cartography-journey-generator.md` | Same pattern. | US-003 | Schema-fill task on 7B |
| 9 | `agents/diagram-updater.md` | `local-eligible: true` + delegation for Mermaid generation subtask with model_medium. mmdc validation unchanged. | US-004 | Mermaid generation on 13B |
| 10 | `agents/diagram-generator.md` | Same pattern with model_medium. | US-004 | Mermaid generation on 13B |
| 11 | `agents/convention-auditor.md` | `local-eligible: true` + delegation for finding aggregation subtask with model_medium. Fallback: Sonnet (not Haiku). | US-005 | Grep aggregation on 13B |
| 12 | `docs/guides/local-llm-setup.md` | New file: Ollama install (`brew install ollama` / `curl`), model pull (`ollama pull mistral:7b-instruct`, `ollama pull qwen2.5:14b-instruct`), MCP server add (`claude mcp add ollama-mcp`), verification (`ecc config set local-llm.enabled true`), troubleshooting. | US-006 | User onboarding |
| 13 | `CLAUDE.md` | Add to Gotchas: "Local LLM offload (BL-128): agents with `local-eligible: true` call Ollama via MCP for mechanical tasks. Requires `ollama-mcp` bridge installed. Without Ollama, agents fall back to hosted model." | US-006 | Document for onboarding |
| 14 | `CHANGELOG.md` | Add BL-128 entry under Unreleased. | US-006 | Record change |

## Pass Conditions

| PC | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | LocalLlmConfig in RawEccConfig, default disabled | AC-001.2, AC-001.6 | `cargo test -p ecc-ports config` | PASS |
| PC-002 | unit | [local_llm] TOML parsing round-trip | AC-001.2 | `cargo test -p ecc-infra file_config` | PASS |
| PC-003 | unit | ecc config set local-llm.enabled true persists | AC-001.3, AC-001.6 | `cargo test -p ecc-cli config` | PASS |
| PC-004 | content | local-llm-delegation skill exists with required sections | AC-002.1, AC-002.2, AC-002.3, AC-002.4, AC-002.5, AC-002.6 | `test -f skills/local-llm-delegation/SKILL.md && grep 'ollama_generate' skills/local-llm-delegation/SKILL.md` | exists + match |
| PC-005 | content | Cartography trio has local-eligible + MCP delegation | AC-003.1, AC-003.2, AC-003.3 | `grep -l 'local-eligible: true' agents/cartograph*.md \| wc -l && grep -l 'ollama_generate' agents/cartograph*.md \| wc -l` | both 3 |
| PC-006 | content | Diagram agents have local-eligible + MCP delegation | AC-004.1, AC-004.2, AC-004.3 | `grep -l 'local-eligible: true' agents/diagram-*.md \| wc -l && grep -l 'ollama_generate' agents/diagram-*.md \| wc -l` | both 2 |
| PC-007 | content | Convention auditor local-eligible + Sonnet fallback | AC-005.1, AC-005.2, AC-005.3 | `grep 'local-eligible: true' agents/convention-auditor.md && grep -i 'sonnet.*fallback\|fallback.*sonnet' agents/convention-auditor.md` | both match |
| PC-008 | content | Setup guide exists with required sections | AC-006.1, AC-001.1, AC-001.5 | `test -f docs/guides/local-llm-setup.md && grep 'ollama' docs/guides/local-llm-setup.md && grep 'version' docs/guides/local-llm-setup.md` | all pass |
| PC-009 | docs | CLAUDE.md + CHANGELOG updated | AC-006.2, AC-006.3 | `grep 'local-eligible' CLAUDE.md && grep 'BL-128' CHANGELOG.md` | both match |
| PC-010 | validation | All files pass structural validation | AC-006.4 | `ecc validate agents && ecc validate commands` | exit 0 |
| PC-011 | build | Full build + lint clean | Constraints | `cargo clippy -- -D warnings` | exit 0 |
| PC-012 | unit | Health-check function returns false on connection refused | AC-001.4 | `cargo test -p ecc-infra health_check` | PASS |

## Coverage Check

| AC | PC |
|----|----|
| AC-001.1 | PC-008 |
| AC-001.2 | PC-001, PC-002 |
| AC-001.3 | PC-003 |
| AC-001.4 | PC-008 (documented in setup guide) |
| AC-001.5 | PC-008 |
| AC-001.6 | PC-001, PC-003 |
| AC-002.1 | PC-004 |
| AC-002.2 | PC-004 |
| AC-002.3 | PC-004 |
| AC-002.4 | PC-004 |
| AC-002.5 | PC-004 |
| AC-002.6 | PC-004 |
| AC-003.1 | PC-005 |
| AC-003.2 | PC-005 |
| AC-003.3 | PC-005 |
| AC-004.1 | PC-006 |
| AC-004.2 | PC-006 |
| AC-004.3 | PC-006 |
| AC-005.1 | PC-007 |
| AC-005.2 | PC-007 |
| AC-005.3 | PC-007 |
| AC-006.1 | PC-008 |
| AC-006.2 | PC-009 |
| AC-006.3 | PC-009 |
| AC-006.4 | PC-010 |

25/25 ACs covered.

## E2E Test Plan

| Boundary | Adapter | Port | Description | Default | Activation |
|----------|---------|------|-------------|---------|------------|
| ConfigStore | FileConfigStore | ConfigStore | [local_llm] round-trip | ignored | `ECC_E2E_ENABLED=1` |

E2E Activation: Only when `ecc config set local-llm.*` is exercised against real filesystem.

## Test Strategy

1. **Unit tests** (PC-001 to PC-003): TDD on Rust config plumbing — test LocalLlmConfig default, TOML parse, CLI set handler
2. **Content checks** (PC-004 to PC-009): Grep verification of agent frontmatter, skill, docs
3. **Validation** (PC-010): `ecc validate agents && ecc validate commands`
4. **Build** (PC-011): `cargo clippy -- -D warnings`

## Doc Update Plan

| Doc File | Level | Action | Content Summary | Spec Ref |
|----------|-------|--------|-----------------|----------|
| docs/guides/local-llm-setup.md | New file | Create | Full setup guide | AC-006.1 |
| CLAUDE.md | Gotchas | Add line | Local LLM note | AC-006.2 |
| CHANGELOG.md | Entry | Add | BL-128 summary | AC-006.3 |

## SOLID Assessment

**PASS** — Follows existing `RawEccConfig` → `ConfigToml` → `ecc config set` pattern. New `LocalLlmConfig` is a plain data struct in ports (no I/O). Agent changes are content-layer instructions.

## Robert's Oath Check

**CLEAN** — Graceful degradation (AC-002.2, AC-002.5) + kill switch (AC-001.6) + path logging. Optional feature with zero degradation when absent.

## Security Notes

**CLEAR** — Ollama runs locally. No external data flow. MCP tool calls don't pass raw user input. Config values are validated strings.

## Rollback Plan

- Rust: revert 3 commits (ports, infra, CLI). Config backwards-compatible — old configs without `[local_llm]` still parse (serde default).
- Content: revert agent frontmatter edits. Remove `local-eligible: true` lines.
- Kill switch: `ecc config set local-llm.enabled false` disables everything at runtime without code changes.

## Bounded Contexts Affected

| Context | Change |
|---------|--------|
| Configuration (ecc-ports/ecc-infra/ecc-cli) | Add LocalLlmConfig |
| Content layer (agents/) | Add local-eligible frontmatter + delegation instructions |
| Documentation (docs/) | New setup guide |
