# Solution: Domain-Specialized Agent Generator with Pipeline Injection

## Spec Reference
Concern: dev, Feature: domain-specialized-agent-generator

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/config/agent_frontmatter.rs` | Modify | Add `generated: Option<bool>`, `generated_at: Option<String>` to AgentFrontmatter | US-003: AC-003.4 |
| 2 | `crates/ecc-app/src/validate/agents.rs` | Modify | Recurse into `agents/domain/` when it exists | US-003: AC-003.1-3 |
| 3 | `skills/domain-agents/SKILL.md` | Create | Discovery and usage pattern documentation | US-006: AC-006.1-4 |
| 4 | `commands/generate-domain-agents.md` | Create | Command: parse bounded-contexts, extract domain data, generate agents | US-001: AC-001.1-9, US-002: AC-002.1-6, US-005: AC-005.1-3 |
| 5 | `commands/spec-dev.md` | Modify | Insert "Phase 0.7: Domain Context" section AFTER Phase 3.5 (Sources) text block. "0.7" is a cross-cutting label, not sequential — insertion is position-based (after Sources, before next phase) | US-004: AC-004.1, AC-004.4-7 |
| 6 | `commands/spec-fix.md` | Modify | Insert Phase 0.7: Domain Context after Phase 3.5 | US-004: AC-004.1, AC-004.4-7 |
| 7 | `commands/spec-refactor.md` | Modify | Insert Phase 0.7: Domain Context after Phase 3.5 | US-004: AC-004.1, AC-004.4-7 |
| 8 | `commands/design.md` | Modify | Insert Phase 0.7: Domain Context after Phase 0.5 | US-004: AC-004.2, AC-004.4-7 |
| 9 | `commands/implement.md` | Modify | Insert Phase 0.7: Domain Context after Phase 0.5 | US-004: AC-004.3, AC-004.4-7 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | AgentFrontmatter with generated=true + generated_at validates | AC-003.4 | `cargo test -p ecc-domain -- agent_frontmatter::tests::agent_with_generated_fields_passes` | PASS |
| PC-002 | unit | AgentFrontmatter without generated fields (backward compat) | AC-003.4 | `cargo test -p ecc-domain -- agent_frontmatter::tests::agent_without_generated_fields_passes` | PASS |
| PC-003 | integration | validate agents passes for valid agents/domain/ file | AC-003.1 | `cargo test -p ecc-app -- validate::agents::tests::agents_domain_subdir_valid_file` | PASS |
| PC-004 | integration | validate agents fails for invalid agents/domain/ file | AC-003.2 | `cargo test -p ecc-app -- validate::agents::tests::agents_domain_subdir_missing_model_fails` | PASS |
| PC-005 | integration | validate agents succeeds when agents/domain/ absent | AC-003.3 | `cargo test -p ecc-app -- validate::agents::tests::agents_domain_subdir_absent_succeeds` | PASS |
| PC-006 | integration | Validated count includes domain subdir agents | AC-003.1 | `cargo test -p ecc-app -- validate::agents::tests::agents_count_includes_domain_subdir` | PASS |
| PC-007 | lint | clippy ecc-domain clean | Build | `cargo clippy -p ecc-domain -- -D warnings` | exit 0 |
| PC-008 | lint | clippy ecc-app clean | Build | `cargo clippy -p ecc-app -- -D warnings` | exit 0 |
| PC-009 | build | cargo build succeeds | Build | `cargo build` | exit 0 |
| PC-010 | content | skills/domain-agents/SKILL.md passes validation | AC-006.1 | `ecc validate skills` | PASS |
| PC-011 | content | Skill has name: domain-agents, origin: ECC | AC-006.1 | `grep -c 'name: domain-agents' skills/domain-agents/SKILL.md && grep -c 'origin: ECC' skills/domain-agents/SKILL.md` | Both 1 |
| PC-012 | content | Skill documents agents/domain/ structure | AC-006.2 | `grep -c 'agents/domain/' skills/domain-agents/SKILL.md` | >= 1 |
| PC-013 | content | Skill under 500 words | AC-006.3 | `wc -w < skills/domain-agents/SKILL.md \| awk '{print ($1 < 500)}'` | 1 |
| PC-014 | content | Skill includes graceful degradation note | AC-006.4 | `grep -ci 'graceful degradation\|skip silently\|does not exist.*skip' skills/domain-agents/SKILL.md` | >= 1 |
| PC-015 | content | Command passes validation | AC-001.4 | `ecc validate commands` | PASS |
| PC-016 | content | Command references bounded-contexts.md + AskUserQuestion | AC-001.1 | `grep -c 'bounded-contexts.md' commands/generate-domain-agents.md && grep -c 'AskUserQuestion' commands/generate-domain-agents.md` | Both >= 1 |
| PC-017 | content | Command handles missing bounded-contexts.md | AC-001.7 | `grep -c 'bounded-contexts.md not found' commands/generate-domain-agents.md` | >= 1 |
| PC-018 | content | Command handles empty bounded-contexts.md | AC-001.8 | `grep -c 'No modules found' commands/generate-domain-agents.md` | >= 1 |
| PC-019 | content | Command handles missing source with skip warning | AC-001.2 | `grep -c 'Source not found.*skipping' commands/generate-domain-agents.md` | >= 1 |
| PC-020 | content | Command handles single-file modules | AC-001.9 | `grep -c '<module>.rs' commands/generate-domain-agents.md` | >= 1 |
| PC-021 | content | Command extracts pub struct/enum, #[error], #[test] | AC-001.3 | `grep -c 'pub struct\|pub enum' commands/generate-domain-agents.md` | >= 1 |
| PC-022 | content | Command writes to agents/domain/ with generated frontmatter | AC-001.4 | `grep -c 'agents/domain/' commands/generate-domain-agents.md && grep -c 'generated: true' commands/generate-domain-agents.md` | Both >= 1 |
| PC-023 | content | Command asks before overwriting | AC-001.5 | `grep -c 'already exists.*Overwrite' commands/generate-domain-agents.md` | >= 1 |
| PC-024 | content | Command commits with conventional message | AC-001.6 | `grep -c 'feat: generate domain agents' commands/generate-domain-agents.md` | >= 1 |
| PC-025 | content | Body template has Domain Model section | AC-002.1 | `grep -c '## Domain Model' commands/generate-domain-agents.md` | >= 1 |
| PC-026 | content | Body template has Error Catalogue section | AC-002.2 | `grep -c '## Error Catalogue' commands/generate-domain-agents.md` | >= 1 |
| PC-027 | content | Body template has Test Patterns section | AC-002.3 | `grep -c '## Test Patterns' commands/generate-domain-agents.md` | >= 1 |
| PC-028 | content | Body template has Cross-Module Dependencies section | AC-002.4 | `grep -c '## Cross-Module Dependencies' commands/generate-domain-agents.md` | >= 1 |
| PC-029 | content | Body template has Naming Conventions section | AC-002.5 | `grep -c '## Naming Conventions' commands/generate-domain-agents.md` | >= 1 |
| PC-030 | content | Verification step checks all 5 placeholder patterns | AC-002.6 | `grep -cE 'TODO.*describe.*fill.*step.*your\|placeholder.*TODO\|grep.*TODO.*describe.*fill' commands/generate-domain-agents.md` | >= 1 |
| PC-031 | content | Staleness via git log --since | AC-005.1 | `grep -c 'git log --since' commands/generate-domain-agents.md` | >= 1 |
| PC-032 | content | --check-staleness flag documented | AC-005.2 | `grep -c '\-\-check-staleness' commands/generate-domain-agents.md` | >= 1 |
| PC-033 | content | Git unavailable handled gracefully | AC-005.3 | `grep -c 'git not available\|staleness check skipped' commands/generate-domain-agents.md` | >= 1 |
| PC-034 | content | spec-dev.md has Phase 0.7 Domain Context | AC-004.1, AC-004.7 | `grep -c 'Phase 0.7.*Domain Context' commands/spec-dev.md` | >= 1 |
| PC-035 | content | spec-fix.md has Phase 0.7 Domain Context | AC-004.1, AC-004.7 | `grep -c 'Phase 0.7.*Domain Context' commands/spec-fix.md` | >= 1 |
| PC-036 | content | spec-refactor.md has Phase 0.7 Domain Context | AC-004.1, AC-004.7 | `grep -c 'Phase 0.7.*Domain Context' commands/spec-refactor.md` | >= 1 |
| PC-037 | content | design.md has Phase 0.7 Domain Context | AC-004.2, AC-004.7 | `grep -c 'Phase 0.7.*Domain Context' commands/design.md` | >= 1 |
| PC-038 | content | implement.md has Phase 0.7 Domain Context | AC-004.3, AC-004.7 | `grep -c 'Phase 0.7.*Domain Context' commands/implement.md` | >= 1 |
| PC-039 | content | Phase 0.7 in spec-dev contains exact-match instruction (anchored to Domain Context section) | AC-004.1 | `awk '/Phase 0.7/,/^## Phase/' commands/spec-dev.md \| grep -c 'exact.match\|exact match'` | >= 1 |
| PC-040 | content | Phase 0.7 in design.md references Affected Modules for matching | AC-004.2 | `awk '/Phase 0.7/,/^## Phase/' commands/design.md \| grep -c 'Affected Modules'` | >= 1 |
| PC-041 | content | Phase 0.7 in implement.md parses File Changes crate paths | AC-004.3 | `awk '/Phase 0.7/,/^## Phase/' commands/implement.md \| grep -c 'File Changes'` | >= 1 |
| PC-042 | content | Phase 0.7 section specifies read-only subagent tools | AC-004.4 | `awk '/Phase 0.7/,/^## Phase/' commands/spec-dev.md \| grep -c 'Read.*Grep.*Glob'` | >= 1 |
| PC-043 | content | Phase 0.7 section contains graceful degradation for absent dir | AC-004.5 | `awk '/Phase 0.7/,/^## Phase/' commands/spec-dev.md \| grep -c 'does not exist\|skip'` | >= 1 |
| PC-044 | content | Phase 0.7 section caps at 3 alphabetically | AC-004.6 | `awk '/Phase 0.7/,/^## Phase/' commands/spec-dev.md \| grep -c 'first 3\|alphabetical'` | >= 1 |
| PC-045 | content | Command file has no leaked placeholders (all 5 patterns, allow <=2 for verification instruction) | AC-002.6 | `grep -ciE 'TODO\|<describe>\|<fill>\|\[step\|\[your' commands/generate-domain-agents.md \| awk '{print ($1 <= 2)}'` | 1 |
| PC-046 | content | All pipeline commands pass validation after Phase 0.7 insertion | AC-004.7 | `ecc validate commands` | PASS |
| PC-047 | content | Phase 0.7 output stored in Domain Context section | AC-004.4 | `awk '/Phase 0.7/,/^## Phase/' commands/spec-dev.md \| grep -c 'Domain Context'` | >= 2 |
| PC-048 | content | spec-dev Phase 0.7 tokenizes feature description (dual-format: both table + freestanding) | AC-004.1 | `awk '/Phase 0.7/,/^## Phase/' commands/spec-dev.md \| grep -c 'tokeniz\|bounded-contexts'` | >= 1 |
| PC-049 | content | Staleness --check-staleness exits 1 if stale, 0 if fresh | AC-005.2 | `grep -c 'exit 1.*stale\|exit 0.*fresh' commands/generate-domain-agents.md` | >= 1 |
| PC-050 | lint | cargo fmt --check | Build | `cargo fmt --check` | exit 0 |
| PC-051 | build | cargo test full workspace | Build | `cargo test` | exit 0 |

### Coverage Check

All 33 ACs covered. Zero uncovered.

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | agents/ directory | FileSystem | validate_agents | Subdirectory recursion | ignored | validate/agents.rs modified |

### E2E Activation Rules

Boundary 1 activated — PC-003 through PC-006 cover it.

## Test Strategy

TDD order:
1. **Phase 1** (PC-001-009): Rust infrastructure first — frontmatter fields + validate recursion
2. **Phase 2** (PC-010-014): Discovery skill — documents the convention before content uses it
3. **Phase 3** (PC-015-024): Command skeleton — core generation logic
4. **Phase 4** (PC-025-030, PC-045): Body content standard — extends command with section templates
5. **Phase 5** (PC-034-044): Pipeline injection — Phase 0.7 in all 5 commands
6. **Phase 6** (PC-031-033): Staleness detection — extends command

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/adr/0066-domain-agent-generation.md` | Docs | Create | LLM-reads-code, agents/domain/, generated marker | Decision 3 |
| 2 | `docs/adr/0067-pipeline-domain-injection.md` | Docs | Create | Phase 0.7 pattern, matching algorithm, cap at 3 | Decision 4 |
| 3 | `CLAUDE.md` | Project | Modify | Add /generate-domain-agents CLI entry + 3 glossary terms (domain agent, domain context injection, agent generation profile) | US-001, US-004 |
| 4 | `CHANGELOG.md` | Project | Modify | Add feature entry | All US |

## SOLID Assessment

**PASS** — 0 findings. All changes are additive: optional fields on existing struct, optional directory in existing validation.

## Robert's Oath Check

**CLEAN** — 0 warnings. 45 PCs, 6 phases, each independently shippable.

## Security Notes

**CLEAR** — 0 findings. Domain agents are read-only (no Write/Edit). No secrets, no network, no shell execution.

## Rollback Plan

1. Revert pipeline command modifications (spec-dev, spec-fix, spec-refactor, design, implement)
2. Delete `commands/generate-domain-agents.md`
3. Delete `skills/domain-agents/SKILL.md`
4. Revert `crates/ecc-app/src/validate/agents.rs`
5. Revert `crates/ecc-domain/src/config/agent_frontmatter.rs`
6. Delete `agents/domain/` directory (generated output)
7. Delete ADRs and revert CLAUDE.md/CHANGELOG

## Bounded Contexts Affected

| Context | Role | Files Modified |
|---------|------|----------------|
| config | Value object | agent_frontmatter.rs |

Other domain modules: none

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | PASS | 0 |
| Robert | CLEAN | 0 |
| Security | CLEAR | 0 |

### Adversary Findings

| Dimension | Score | Verdict |
|-----------|-------|---------|
| AC Coverage | 92 | PASS |
| Execution Order | 95 | PASS |
| Fragility | 78 | PASS |
| Rollback | 90 | PASS |
| Architecture | 95 | PASS |
| Blast Radius | 82 | PASS |
| Missing PCs | 90 | PASS |
| Doc Plan | 85 | PASS |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | crates/ecc-domain/src/config/agent_frontmatter.rs | modify | US-003 |
| 2 | crates/ecc-app/src/validate/agents.rs | modify | US-003 |
| 3 | skills/domain-agents/SKILL.md | create | US-006 |
| 4 | commands/generate-domain-agents.md | create | US-001, US-002, US-005 |
| 5-9 | commands/spec-dev,fix,refactor,design,implement.md | modify | US-004 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-17-domain-agent-generator/spec.md | Full spec |
| docs/specs/2026-04-17-domain-agent-generator/design.md | Full design |
