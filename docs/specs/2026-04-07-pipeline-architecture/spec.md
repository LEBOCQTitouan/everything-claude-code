# Spec: BL-127 Pipeline Architecture — Session & Subagent Reduction

## Problem Statement

The spec→design→implement pipeline requires 3 separate sessions per feature, each cold-starting context. Within `/design`, 3 sequential read-only reviewers (uncle-bob, robert, security-reviewer) launch in separate contexts despite sharing identical tool access and having no data dependency. `/implement` spawns one tdd-executor per PC even when multiple independent PCs target the same file. `/audit-full` re-runs all domain audits from scratch even when code is unchanged since the last audit.

## Research Summary

- Combined spec+design saves ~1 session boundary per feature (context cold-start + full spec re-read eliminated)
- Composite reviewer pattern: merging read-only scanners with identical tools into one context is a standard multi-agent optimization
- TDD batch dispatch: extending existing wave model to group independent same-file PCs reduces per-PC subagent overhead
- Audit caching with content-hash + TTL invalidation prevents redundant analysis; disk-backed cache survives sessions

## Decisions Made

| # | Decision | Rationale | ADR? |
|---|----------|-----------|------|
| 1 | `--continue` is opt-in flag, not default | Preserve explicit user control at phase boundaries | No |
| 2 | Composite design-reviewer merges 3 agents into 1 | All are read-only with identical tool access; no sequential dependency | Yes |
| 3 | Batch only independent same-file PCs | Preserves RED-GREEN TDD ordering for dependent PCs | No |
| 4 | Rust cache with CachePort trait | Precise content-hash + TTL invalidation; survives across sessions; `--force` bypass | No |

## User Stories

### US-001: Combined spec+design flow

**As a** developer, **I want** to run `/spec-dev --continue` so that `/design` starts automatically after spec adversarial PASS, **so that** I save one session boundary and spec re-read per feature.

#### Acceptance Criteria

- AC-001.1: `--continue` flag recognized by `/spec-dev`, `/spec-fix`, `/spec-refactor` and `/spec` router
- AC-001.2: After spec adversarial review PASS, `/design` Skill is invoked in the same session without user re-running the command
- AC-001.3: Without `--continue`, behavior is unchanged — spec stops at "Run `/design` to continue"
- AC-001.4: User still sees Plan Mode preview before design executes (EnterPlanMode is not skipped)
- AC-001.5: If spec adversarial verdict is CONDITIONAL (not PASS), `--continue` stops and prompts the user — design does not auto-launch on non-PASS verdicts
- AC-001.6: `/spec` router passes `--continue` flag through to the delegated `/spec-*` command unchanged

#### Dependencies

- Depends on: none

### US-002: Composite design-reviewer

**As a** developer, **I want** the three sequential design reviewers consolidated into one invocation, **so that** `/design` Phases 2-4 run in one context instead of three.

#### Acceptance Criteria

- AC-002.1: New `agents/design-reviewer.md` with frontmatter (model: opus, effort: high, tools: [Read, Grep, Glob]) covering SOLID principles, Programmer's Oath, and security scan
- AC-002.2: `commands/design.md` Phases 2-4 replaced by single Task launch of design-reviewer
- AC-002.3: Output structured with labeled sections (## SOLID Assessment, ## Oath Evaluation, ## Security Notes) so findings are distinguishable by dimension
- AC-002.4: Existing agents `uncle-bob.md`, `robert.md`, `security-reviewer.md` remain in repo — only `/design` stops calling them separately
- AC-002.5: Rollback: removing `design-reviewer.md` and reverting `design.md` to prior version restores original 3-agent behavior with no other changes needed

#### Dependencies

- Depends on: none

### US-003: Batched tdd-executor for same-file PCs

**As a** developer, **I want** independent PCs targeting the same file grouped into single tdd-executor invocations, **so that** redundant file reads and context overhead are reduced.

#### Acceptance Criteria

- AC-003.1: Wave analysis in `skills/wave-analysis/SKILL.md` adds same-file grouping: PCs with identical primary `## Files to Modify` AND no inter-PC dependency are candidates for batching
- AC-003.2: Wave dispatch in `skills/wave-dispatch/SKILL.md` sends batched PCs to tdd-executor with a combined context brief listing all PCs
- AC-003.3: Fix-round budget is tracked per-PC within the batch (not per-batch)
- AC-003.4: Single-PC waves use existing sequential dispatch path (backward compatibility preserved)
- AC-003.5: tdd-executor agent instructions in `agents/tdd-executor.md` accept multi-PC context briefs and execute each PC's RED-GREEN-REFACTOR cycle sequentially within one invocation
- AC-003.6: If one PC in a batch fails its fix-round budget, sibling PCs in the same batch continue — failure does not abort the batch
- AC-003.7: "Independent" means no PC in the batch references output or test artifacts created by another PC in the same batch (no inter-PC dependency)

#### Dependencies

- Depends on: none

### US-004: Per-domain audit caching

**As a** developer, **I want** audit results cached per domain with TTL expiry, **so that** re-running `/audit-full` on unchanged domains skips redundant analysis.

#### Acceptance Criteria

- AC-004.1: `CachePort` trait in `ecc-ports` with `check(key) -> Option<CacheEntry>` and `write(key, value, ttl)` methods
- AC-004.2: `FileCacheStore` adapter in `ecc-infra` implementing `CachePort` with disk-backed storage at `~/.ecc/cache/`
- AC-004.3: Cache keyed on `<domain-name>:<content-hash>` where content-hash is SHA-256 of all source files in the domain directory
- AC-004.4: TTL configurable via `ecc config set audit-cache.ttl 24h` (default: 24 hours)
- AC-004.5: `ecc audit cache check <domain>` CLI command returns hit/miss with metadata
- AC-004.6: `ecc audit cache clear` CLI command purges all cache entries
- AC-004.7: `--force` flag on individual `/audit-*` commands and `/audit-full` bypasses cache
- AC-004.8: `audit-orchestrator.md` and `audit-full.md` check cache before launching domain analysis agents
- AC-004.9: Cache write failure degrades gracefully — audit proceeds uncached, logs WARN
- AC-004.10: Unit tests use `InMemoryCacheStore` test double; no disk I/O in domain/port tests
- AC-004.11: `ecc audit cache clear` resolves corrupted cache; documented in setup guide

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| commands/spec-dev.md | command | Add --continue flag handling + /design invocation |
| commands/spec-fix.md | command | Add --continue flag handling |
| commands/spec-refactor.md | command | Add --continue flag handling |
| commands/spec.md | command | Pass --continue through to delegated command |
| agents/design-reviewer.md | agent (new) | Composite SOLID + Oath + Security reviewer |
| commands/design.md | command | Replace Phases 2-4 with single design-reviewer Task |
| skills/wave-analysis/SKILL.md | skill | Add same-file grouping to wave algorithm |
| skills/wave-dispatch/SKILL.md | skill | Add batched dispatch mode |
| agents/tdd-executor.md | agent | Accept multi-PC context briefs |
| commands/implement.md | command | Reference updated wave dispatch |
| commands/audit-full.md | command | Add cache check + --force flag |
| agents/audit-orchestrator.md | agent | Add cache check/write per domain |
| crates/ecc-ports/src/cache_store.rs | Rust port (new) | CachePort trait + CacheEntry |
| crates/ecc-infra/src/file_cache_store.rs | Rust infra (new) | FileCacheStore adapter |
| crates/ecc-cli/src/commands/audit_cache.rs | Rust CLI (new) | ecc audit cache check/clear |
| crates/ecc-test-support/src/ | Rust test | InMemoryCacheStore |
| docs/adr/ | ADR (new) | Design-reviewer consolidation |

## Constraints

- All changes backward-compatible — no existing behavior altered without explicit opt-in
- `--continue` is opt-in, never default
- Batch only independent PCs (no dependency between batched PCs allowed)
- Cache `--force` always available to bypass
- ADR required for design-reviewer consolidation decision
- Existing agents (uncle-bob, robert, security-reviewer) remain in repo for standalone use

## Non-Requirements

- Changing the 3-phase pipeline itself (spec/design/implement remain separate commands)
- Auto-merging phases without user approval (Plan Mode always shown)
- Caching spec or design outputs (only audit results cached)
- Deleting old agents (uncle-bob, robert, security-reviewer still used by /review and other commands)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|---|---|---|
| CachePort (new) | New trait | Needs adapter + test double |
| FileCacheStore (new) | New adapter | Disk I/O at ~/.ecc/cache/ |
| Agent frontmatter | New agent file | ecc validate agents must accept |
| Command markdown | Modified instructions | Behavioral change behind flags |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|---|---|---|---|
| --continue flag | CLAUDE.md | Slash Commands | Document flag on /spec-* |
| design-reviewer | docs/adr/ | New ADR | Consolidation decision |
| audit cache | CLAUDE.md | CLI Commands | Add ecc audit cache commands |
| BL-127 | CHANGELOG.md | Entry | Summary of all 4 changes |

## Open Questions

None — all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Single spec or split into 4? | Single spec, 4 US | Recommended |
| 2 | Audit cache: markdown-only or Rust? | Rust with CachePort trait | User |
| 3 | Batch scope: all same-file or independent only? | Independent same-file PCs only | Recommended |
| 4 | Breaking changes, security, ADR? | No breaking changes, ADR for design-reviewer | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Combined spec+design flow | 6 | none |
| US-002 | Composite design-reviewer | 5 | none |
| US-003 | Batched tdd-executor | 7 | none |
| US-004 | Per-domain audit caching | 11 | none |

### Adversary Findings

| Dimension | Score | Key Fix |
|-----------|-------|---------|
| Ambiguity | 72 | AC-003.7: defined "independent" |
| Edge Cases | 55→fixed | AC-001.5, AC-003.6, AC-004.9 |
| Scope | 78→fixed | AC-001.6 |
| Dependencies | 80 | Clean |
| Testability | 62→fixed | AC-004.10 |
| Decisions | 85 | Solid |
| Rollback | 45→fixed | AC-002.5, AC-004.11 |

### Artifacts

| File Path | Content |
|-----------|---------|
| docs/specs/2026-04-07-pipeline-architecture/spec.md | Full spec |
| docs/specs/2026-04-07-pipeline-architecture/campaign.md | Campaign manifest |
