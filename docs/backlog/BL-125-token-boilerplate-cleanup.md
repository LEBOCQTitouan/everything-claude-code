---
id: BL-125
title: "Token optimization wave 2 — boilerplate extraction and context trimming"
status: open
created: 2026-04-06
promoted_to: ""
tags: [token-optimization, agents, rules, context-bloat]
scope: LOW
target_command: direct edit
dependencies: [BL-121]
---

## Optimized Prompt

Apply 5 context-bloat reduction fixes from the BL-121 audit (Wave 2):

1. Extract TodoWrite graceful degradation boilerplate from 25 agents — replace inline blocks with frontmatter convention `tracking: todowrite`. Document in `rules/ecc/development.md` that agents with this field get automatic TodoWrite behavior. Remove the ~40-word inline boilerplate from all 25 agent files.
2. Slim CLAUDE.md CLI reference — the full CLI command listing (~60 lines) is duplicated in `docs/commands-reference.md`. Keep only top 10 most-used commands in CLAUDE.md, add pointer to full reference.
3. Verify language rule conditional loading — confirm `paths:` frontmatter in `rules/{perl,swift,java,...}/` correctly excludes those rules from context in a Rust-only project. If loaded regardless, file a bug or add exclusion logic.
4. Trim `rules/common/performance.md` (82 lines) — keep the model routing table (~20 lines) and thinking effort tiers; move context management prose and build troubleshooting to `docs/`.
5. Trim `rules/common/agents.md` (53 lines) — the system-reminder already lists all agents with descriptions. Slim to 10-line summary with pointer to full listing.

**Verification:** Measure total rule line count before and after. Confirm `ecc validate rules` passes. Run a test session to verify no behavioral regression.

Reference: `docs/audits/token-optimization-2026-04-06.md` findings 3.1, 3.3, 3.5, 3.6, 3.7.

## Original Input

BL-121 audit Wave 2: extract TodoWrite boilerplate to frontmatter, slim CLAUDE.md, verify language rule loading, trim generic rules.

## Challenge Log

**Source:** BL-121 token optimization audit (2026-04-06). Pre-challenged during audit — boilerplate counts validated by component-auditor agent.
