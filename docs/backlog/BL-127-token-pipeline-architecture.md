---
id: BL-127
title: "Token optimization wave 4 — pipeline architecture for session and subagent reduction"
status: open
created: 2026-04-06
promoted_to: ""
tags: [token-optimization, pipeline, sessions, subagents]
scope: HIGH
target_command: /spec-dev
dependencies: [BL-121, BL-124]
---

## Optimized Prompt

Implement 4 pipeline architecture improvements from the BL-121 audit (Wave 4) to reduce session count and subagent explosion:

1. **Combined spec+design flow** — add `--continue` flag to `/spec-dev` (and `/spec-fix`, `/spec-refactor`) that flows directly into `/design` after adversarial PASS. Gate on Plan Mode approval so user still reviews. Saves one session boundary and full spec re-read per feature. Update `ecc-workflow` state machine to support `spec → solution` transition within a single session.
2. **Composite design-reviewer** — merge `uncle-bob`, `robert`, and `security-reviewer` design review passes in `/design` into a single subagent with a combined prompt. All three are read-only scanners with no sequential dependency — one context instead of three. Create `agents/design-reviewer.md` with merged criteria.
3. **Batched tdd-executor** — in `/implement` Phase 3, group Pass Conditions that share identical `## Files to Modify` targets into single tdd-executor invocations. The wave model already groups independent PCs by file overlap for parallelism; extend it to also batch sequential PCs targeting the same files. Reduces per-PC subagent overhead.
4. **Per-domain audit caching** — `audit-orchestrator` writes timestamped per-domain cache sections to the full-audit report. Individual `/audit-*` commands check for a recent full report (configurable TTL, default 7 days) and surface cached findings instead of re-running. Add `--force` to bypass cache.

Reference: `docs/audits/token-optimization-2026-04-06.md` findings 4.1, 4.2, 4.3, 4.4.

## Original Input

BL-121 audit Wave 4: combine spec+design sessions, merge design reviewers, batch tdd-executor, cache audit results.

## Challenge Log

**Source:** BL-121 token optimization audit (2026-04-06). Pre-challenged during audit — session count analysis validated by pipeline flow mapping.
