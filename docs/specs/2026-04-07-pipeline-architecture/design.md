# Design: BL-127 Pipeline Architecture — Session & Subagent Reduction

## Spec Reference

`docs/specs/2026-04-07-pipeline-architecture/spec.md` — 4 US, 29 ACs.

## File Changes

| # | File(s) | Change | Spec Ref | Rationale |
|---|---------|--------|----------|-----------|
| 1 | `commands/spec-dev.md` | Add `--continue` flag detection at Phase 0. After Phase 10 PASS, if `--continue`: invoke `Skill("design")` instead of STOP. If verdict is CONDITIONAL/FAIL or no `--continue`: stop as before. | US-001 | Saves 1 session boundary |
| 2 | `commands/spec-fix.md` | Same `--continue` handling | US-001 | Consistency |
| 3 | `commands/spec-refactor.md` | Same `--continue` handling | US-001 | Consistency |
| 4 | `commands/spec.md` | Pass `--continue` flag through to delegated `/spec-*` command | US-001 | Router passthrough |
| 5 | `agents/design-reviewer.md` (new) | Composite agent: model opus, effort high, tools [Read, Grep, Glob, Bash]. Three sections: ## SOLID Assessment (SRP, OCP, LSP, ISP, DIP + Clean Architecture), ## Oath Evaluation (Programmer's Oath 1-9), ## Security Notes (design-level scan). Returns combined PASS/findings. | US-002 | Merge 3 contexts into 1 |
| 6 | `commands/design.md` | Replace Phases 2-4 (three separate Task launches) with single Task launch of `design-reviewer`. Merge Phase 2-4 headings into Phase 2: "Design Review (SOLID + Oath + Security)". Remove Phases 3, 4 headings. | US-002 | 3 subagents → 1 |
| 7 | `skills/wave-analysis/SKILL.md` | Add "## Same-File Batching" section: after wave grouping, within each wave, identify PCs with identical primary target file AND no inter-PC dependency. Mark as "batchable". Definition of independent: no PC references output or test artifacts of another PC in same batch. | US-003 | Batch identification |
| 8 | `skills/wave-dispatch/SKILL.md` | Add "## Batched Dispatch" section: for batchable PC groups, send combined context brief to single tdd-executor. Individual fix-round tracking per PC. If one PC fails budget, sibling PCs continue. Single-PC groups use existing path. | US-003 | Batch execution |
| 9 | `agents/tdd-executor.md` | Add "## Multi-PC Mode" section: when context brief contains multiple `## PC Spec` blocks, execute each sequentially (RED-GREEN-REFACTOR per PC). Commit per PC, not per batch. Report status per PC in output. | US-003 | Executor support |
| 10 | `commands/audit-full.md` | Add `--force` flag handling. Before domain agent launch, check `ecc audit cache check <domain>`. If hit and no `--force`: skip domain, use cached section. Otherwise: run normally, then `ecc audit cache write <domain>` after agent returns. | US-004 | Cache integration |
| 11 | `agents/audit-orchestrator.md` | Add cache check/write instructions. On cache write failure: log WARN, proceed uncached. | US-004 | Graceful degradation |
| 12 | `crates/ecc-ports/src/cache_store.rs` (new) | `CachePort` trait with `check(key: &str) -> Result<Option<CacheEntry>>`, `write(key: &str, value: &str, ttl_secs: u64)`, `clear()`. `CacheEntry` struct with `value: String`, `created_at: u64`, `ttl_secs: u64`, `content_hash: String`. `CacheError` enum. | US-004 | Port definition |
| 13 | `crates/ecc-infra/src/file_cache_store.rs` (new) | `FileCacheStore` implementing `CachePort`. Storage at `~/.ecc/cache/` as JSON files keyed by sanitized domain name. TTL checked on read. Content hash is SHA-256 of concatenated source file contents. | US-004 | Adapter |
| 14 | `crates/ecc-test-support/src/` | `InMemoryCacheStore` implementing `CachePort` for unit tests. | US-004 | Test double |
| 15 | `crates/ecc-cli/src/commands/audit_cache.rs` (new) | `ecc audit cache check <domain>` and `ecc audit cache clear` CLI commands. | US-004 | User-facing |
| 16 | `docs/adr/0058-composite-design-reviewer.md` (new) | ADR: Status Accepted, Context (3 agents → 1), Decision, Consequences. | US-002 | Architectural record |
| 17 | `CLAUDE.md` | Add --continue flag, ecc audit cache commands, design-reviewer gotcha | All | Onboarding |
| 18 | `CHANGELOG.md` | BL-127 entry | All | Record |

## Pass Conditions

| PC | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | content | spec-dev/fix/refactor/spec have --continue | AC-001.1-6 | `grep -c '\-\-continue' commands/spec-dev.md commands/spec-fix.md commands/spec-refactor.md commands/spec.md` | each ≥1 |
| PC-002 | content | design-reviewer.md exists with 3 sections | AC-002.1,3-5 | `test -f agents/design-reviewer.md && grep -c 'SOLID\|Oath\|Security' agents/design-reviewer.md` | exists + ≥3 |
| PC-003 | content | design.md references design-reviewer, not separate agents | AC-002.2 | `grep 'design-reviewer' commands/design.md && ! grep 'Launch.*uncle-bob' commands/design.md` | match + no old pattern |
| PC-004 | content | wave-analysis has same-file batch section | AC-003.1,7 | `grep -c 'same-file\|batch\|independent' skills/wave-analysis/SKILL.md` | ≥2 |
| PC-005 | content | wave-dispatch has batched mode | AC-003.2-4,6 | `grep -c 'batch' skills/wave-dispatch/SKILL.md` | ≥2 |
| PC-006 | content | tdd-executor multi-PC mode | AC-003.5 | `grep 'Multi-PC\|multi-PC' agents/tdd-executor.md` | match |
| PC-007 | content | audit-full has cache + force | AC-004.7,8 | `grep 'cache' commands/audit-full.md && grep 'force' commands/audit-full.md` | both match |
| PC-008 | content | audit-orchestrator has cache | AC-004.8,9 | `grep 'cache' agents/audit-orchestrator.md` | match |
| PC-009 | unit | CachePort trait + CacheEntry | AC-004.1 | `cargo test -p ecc-ports cache` | PASS |
| PC-010 | unit | FileCacheStore round-trip + TTL | AC-004.2-5,9 | `cargo test -p ecc-infra file_cache` | PASS |
| PC-011 | unit | InMemoryCacheStore | AC-004.10 | `cargo test -p ecc-test-support cache` | PASS |
| PC-012 | unit | ecc audit cache CLI | AC-004.5,6,11 | `cargo test -p ecc-cli audit_cache` | PASS |
| PC-013 | docs | ADR + CLAUDE.md + CHANGELOG | Doc Impact | `ls docs/adr/*design-reviewer* && grep 'BL-127' CHANGELOG.md` | both exist |
| PC-014 | validation | ecc validate agents + commands | Constraints | `ecc validate agents && ecc validate commands` | exit 0 |
| PC-015 | build | cargo clippy clean | Constraints | `cargo clippy -- -D warnings` | exit 0 |
| PC-016 | unit | FileCacheStore content-hash invalidation | AC-004.3 | `cargo test -p ecc-infra cache_hash_invalidation` | PASS |

## Coverage Check

All 29 ACs mapped:
- AC-001.1-6 → PC-001
- AC-002.1-5 → PC-002, PC-003
- AC-003.1-7 → PC-004, PC-005, PC-006
- AC-004.1-11 → PC-007, PC-008, PC-009, PC-010, PC-011, PC-012
- Doc Impact → PC-013
- Constraints → PC-014, PC-015

29/29 covered.

## E2E Test Plan

| Boundary | Adapter | Port | Description | Default | Activation |
|----------|---------|------|-------------|---------|------------|
| CachePort | FileCacheStore | CachePort | Audit cache R/W | ignored | ECC_E2E_ENABLED=1 |

## E2E Activation Rules

Activate E2E only when `ecc audit cache` subcommand is exercised against real filesystem.

## Test Strategy

1. **Unit tests** (PC-009 to PC-012): TDD on Rust cache plumbing
2. **Content checks** (PC-001 to PC-008): Grep verification
3. **Validation** (PC-014): structural
4. **Build** (PC-015): clippy

## Doc Update Plan

| Doc File | Level | Action | Content | Spec Ref |
|----------|-------|--------|---------|----------|
| docs/adr/0058-composite-design-reviewer.md | New | Create | SCDC ADR | US-002 |
| CLAUDE.md | Slash Commands + CLI | Add | --continue, audit cache | US-001, US-004 |
| CHANGELOG.md | Entry | Add | BL-127 summary | All |

## SOLID Assessment

**PASS** — CachePort follows existing port pattern. Composite agent consolidates without coupling. Wave analysis extension is additive.

## Robert's Oath Check

**CLEAN** — backward compat preserved. --continue opt-in. --force bypass. Graceful cache degradation.

## Security Notes

**CLEAR** — cache stores domain names, hashes, timestamps. No secrets or user data.

## Rollback Plan

- US-001: remove --continue flag handling from 4 command files
- US-002: delete design-reviewer.md, revert design.md to prior 3-agent pattern
- US-003: revert wave-analysis/dispatch/tdd-executor additions
- US-004: delete cache_store.rs, file_cache_store.rs, audit_cache.rs; revert audit-full.md/orchestrator.md

Each sub-feature is independently revertable.

## Bounded Contexts Affected

| Context | Change |
|---------|--------|
| Configuration (ecc-ports/infra/cli) | CachePort + adapter + CLI |
| Content layer (commands/agents/skills) | --continue, design-reviewer, batch, cache instructions |
| Documentation (docs/) | ADR, CLAUDE.md, CHANGELOG |
