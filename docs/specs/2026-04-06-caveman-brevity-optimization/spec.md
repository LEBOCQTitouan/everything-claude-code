# Spec: Caveman-Style Brevity Token Optimization

## Problem Statement

ECC's agents produce verbose output and its instruction files (commands, agents, skills) consume large input context windows. Caveman research shows ~65% output token reduction through linguistic constraint, with a 26pp accuracy improvement. ECC has 190+ instruction files totaling ~38,000 lines. Compressing these saves both input tokens (smaller system prompts) and output tokens (agents follow the terse style). Reference: https://github.com/JuliusBrussee/caveman.

## Research Summary

- Caveman: 65% avg output reduction, 72% on technical explanations
- March 2026 research: brevity improved accuracy by 26pp on benchmarks
- Current ECC baseline: agents=7,928 lines, commands=5,878, skills=24,418 -- 38,224 total
- Largest files: implement.md=502, spec-dev.md=374, design.md=363, doc-orchestrator.md=398

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Global brevity rule in rules/common/ | All agents inherit automatically | No |
| 2 | Full 190+ file audit | Comprehensive compression, not phased | No |
| 3 | No configuration -- single default | Simplicity over flexibility | No |
| 4 | Preserve code blocks, technical terms, error msgs | Same as caveman's preservation list | No |

## User Stories

### US-001: Global Brevity Rule

**As a** developer, **I want** a brevity rule that all agents inherit, **so that** output tokens are reduced without per-agent changes.

#### Acceptance Criteria

- AC-001.1: Given `rules/common/brevity.md`, when loaded, then it instructs agents to eliminate filler words, pleasantries, hedging language, and redundant transitions
- AC-001.2: Given the rule, when agents produce output, then code blocks, technical terms, error messages, and commit messages are preserved verbatim
- AC-001.3: Given CLAUDE.md, when updated, then it references the brevity rule

#### Dependencies
- Depends on: none

### US-002: Command Instruction Compression

**As a** developer, **I want** command instruction files compressed, **so that** less input context is consumed per session.

#### Acceptance Criteria

- AC-002.1: Given each command file in `commands/*.md`, when compressed, then verbose prose is replaced with tables/bullet points where equivalent
- AC-002.2: Given the compression, when measured, then the total command line count is reduced by >= 30%
- AC-002.3: Given the compression, when commands are executed, then behavioral fidelity is maintained (no instructions lost)

#### Dependencies
- Depends on: none

### US-003: Agent Instruction Compression

**As a** developer, **I want** agent instruction files compressed, **so that** less input context is consumed when agents are dispatched.

#### Acceptance Criteria

- AC-003.1: Given each agent file in `agents/*.md`, when compressed, then redundant examples, repeated boilerplate, and verbose explanations are removed
- AC-003.2: Given the compression, when measured, then the total agent line count is reduced by >= 30%
- AC-003.3: Given the compression, when agents execute, then behavioral fidelity is maintained

#### Dependencies
- Depends on: none

### US-004: Skill Instruction Compression

**As a** developer, **I want** skill instruction files compressed, **so that** less context is consumed when skills are loaded.

#### Acceptance Criteria

- AC-004.1: Given each skill file in `skills/*/SKILL.md`, when compressed, then multi-paragraph explanations are collapsed into concise directives
- AC-004.2: Given the compression, when measured, then the total skill line count is reduced by >= 30%
- AC-004.3: Given the compression, when skills are referenced, then behavioral fidelity is maintained

#### Dependencies
- Depends on: none

## Affected Modules

| Module | Layer | Change Type |
|--------|-------|-------------|
| `rules/common/brevity.md` (new) | Rules | New brevity rule |
| `commands/*.md` (30+ files) | Commands | Instruction compression |
| `agents/*.md` (57 files) | Agents | Instruction compression |
| `skills/*/SKILL.md` (100+ files) | Skills | Instruction compression |
| `CLAUDE.md` | Docs | Reference brevity rule |

## Constraints

- No behavioral changes -- same instructions, fewer words
- Code blocks, technical terms, error messages preserved verbatim
- Frontmatter unchanged -- only prose body compressed
- No Rust code changes

## Non-Requirements

- Configurable intensity levels
- Output format changes (JSON, structured output)
- CI integration
- Per-session brevity toggling

## E2E Boundaries Affected
None -- instruction text changes only.

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New rule | rules/common/brevity.md | Create brevity rule | AC-001.1 |
| Reference | CLAUDE.md | Add brevity rule pointer | AC-001.3 |
| Changelog | CHANGELOG.md | Add entry | all |

## Rollback Plan

1. Revert `rules/common/brevity.md`
2. Revert all compressed files (git revert the compression commits)
3. No data/config/code changes to revert

## Open Questions
None -- all questions resolved during grill-me interview.

## Baseline Measurements

| Category | Files | Lines | Target (30% reduction) |
|----------|-------|-------|------------------------|
| Commands | 30+ | 5,878 | <= 4,115 |
| Agents | 57 | 7,928 | <= 5,550 |
| Skills | 100+ | 24,418 | <= 17,093 |
| **Total** | **190+** | **38,224** | **<= 26,757** |
