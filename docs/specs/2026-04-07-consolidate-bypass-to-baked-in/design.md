# Solution: Consolidate Bypass to Baked-In Auditable System

## Spec Reference
Concern: refactor, Feature: Consolidate bypass to use only baked-in bypass, remove ECC bypass

## File Changes (dependency order)

### Phase 1: Pre-Refactor Test Coverage (US-006)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| F-001 | `crates/ecc-app/src/hook/mod.rs` (tests) | modify | Add 3 characterization tests for token bypass path | AC-006.1, AC-006.2, AC-006.3 |
| F-002 | `crates/ecc-workflow/tests/transition.rs` | modify | Add test proving binary ignores ECC_WORKFLOW_BYPASS=1 | AC-006.4 |

### Phase 2: check_token() Port (US-002)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| F-003 | `crates/ecc-domain/src/hook_runtime/bypass.rs` | modify | Add path validation for hook_id (`^[a-z0-9:_-]+$`) and session_id (`^[a-zA-Z0-9_-]{1,128}$`) | AC-002.5, AC-002.6, security |
| F-004 | `crates/ecc-ports/src/bypass_store.rs` | modify | Add `check_token(hook_id, session_id) -> Option<BypassToken>` to trait | AC-002.1 |
| F-005 | `crates/ecc-infra/src/sqlite_bypass_store.rs` | modify | Add `home_dir: Option<PathBuf>` to constructor. Implement `check_token()`: build path from home_dir + session_id + hook_id, read+parse, return None on failure | AC-002.2, AC-002.5, AC-002.6, AC-002.7 |
| F-006 | `crates/ecc-test-support/src/in_memory_bypass_store.rs` | modify | Add `tokens` field, `with_token()` builder, implement `check_token()` | AC-002.3 |

### Phase 3: Test Boilerplate (US-003)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| F-007 | `crates/ecc-app/src/hook/mod.rs` | modify | Add `HookPorts::test_default(fs, shell, env, terminal)` with optional ports as None | AC-003.1 |
| F-008 | 26 handler files + `hook/mod.rs` tests | modify | Replace manual HookPorts construction with `test_default()` | AC-003.2 |
| F-009 | `crates/ecc-cli/src/commands/hook.rs` | modify | Wire SqliteBypassStore when DB path resolves, None otherwise with tracing::debug | AC-003.3 |

### Phase 4: Domain Abstractness (US-005)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| F-010 | `crates/ecc-domain/src/hook_runtime/bypass.rs` | modify | Add `BypassPolicy` trait with `fn should_bypass(&self, hook_id: &str, session_id: &str) -> bool` | AC-005.1 |
| F-011 | `crates/ecc-app/src/hook/mod.rs` or new file | modify | Add `AlwaysDenyPolicy` impl in ecc-app (per uncle-bob: policy impls in app layer) | AC-005.2 |

### Phase 5: ecc-workflow Binary Cleanup (US-001 partial)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| F-012 | `crates/ecc-workflow/src/main.rs` | modify | Delete lines 211-213 (ECC_WORKFLOW_BYPASS check + process::exit) | AC-001.2, AC-001.8 |
| F-013 | `crates/ecc-workflow/tests/transition.rs` | modify | Rewrite bypass test to assert normal execution with env var | AC-001.2, AC-001.8 |
| F-014 | 8 ecc-workflow test files | modify | Remove `env_remove("ECC_WORKFLOW_BYPASS")` calls | AC-001.6 |

### Phase 6: dispatch() Refactor (US-001 + US-004)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| F-015 | `crates/ecc-app/src/hook/bypass_interceptor.rs` | create | `intercept()` function: hint append, token check via bypass_store.check_token(), audit via bypass_store.record() | AC-004.1, AC-002.4 |
| F-016 | `crates/ecc-app/src/hook/mod.rs` | modify | Add `mod bypass_interceptor`. Remove ECC_WORKFLOW_BYPASS deprecation check (lines 157-163). Remove inline token resolution (lines 260-315). Call bypass_interceptor::intercept() when exit_code==2 | AC-001.1, AC-004.2 |
| F-017 | 4 integration test files | modify | Remove `.env("ECC_WORKFLOW_BYPASS", "0")` calls | AC-001.11 |
| F-029 | `crates/ecc-app/src/hook/handlers/tier1_simple/worktree_guard.rs` | modify | Remove comment "Handler no longer checks ECC_WORKFLOW_BYPASS" (line 176) | AC-001.4 |
| F-030 | `crates/ecc-app/src/hook/handlers/tier3_session/session_merge.rs` | modify | Remove comment "Handler no longer checks ECC_WORKFLOW_BYPASS" (line 320) | AC-001.4 |

### Phase 7: Documentation + Config Cleanup (US-001 remainder)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| F-018 | `.envrc` | delete | Sole content is bypass env var | AC-001.3, AC-001.10 |
| F-028 | `.gitignore` | modify | Remove `.envrc` line and ECC_WORKFLOW_BYPASS comment | AC-001.4 |
| F-019 | `CLAUDE.md` | modify | Remove ECC_WORKFLOW_BYPASS refs, add direnv revoke note, add `ecc bypass grant` reference | AC-001.9, AC-001.13 |
| F-020 | `docs/adr/0056-ecc-workflow-bypass-deprecation.md` | modify | Status "Accepted" → "Completed" | AC-001.5 |
| F-021 | `rules/ecc/development.md` | modify | Remove ECC_WORKFLOW_BYPASS hook convention | AC-001.12 |
| F-022 | `commands/ecc-test-mode.md` | modify | Rewrite to use token-based bypass | AC-001.12 |
| F-023 | `commands/create-component.md` | modify | Remove bypass check from hook template | AC-001.12 |
| F-024 | `skills/ecc-component-authoring/SKILL.md` | modify | Remove bypass convention | AC-001.12 |
| F-025 | `patterns/agentic/guardrails.md` | modify | Remove bypass env var reference | AC-001.12 |
| F-026 | `tests/hooks/test-phase-gate.sh` | modify | Remove test_bypass() function | AC-001.7 |
| F-027 | `CHANGELOG.md` | modify | Add entry for bypass consolidation | AC-001.9 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | Dispatch with bypass token file present returns exit 0 | AC-006.1 | `cargo test -p ecc-app -- tests::bypass_token_found_passthrough` | PASS |
| PC-002 | unit | Dispatch with no token returns exit 2 with hint | AC-006.2 | `cargo test -p ecc-app -- tests::bypass_token_not_found_blocks` | PASS |
| PC-003 | unit | Dispatch with no CLAUDE_SESSION_ID returns exit 2 | AC-006.3 | `cargo test -p ecc-app -- tests::no_session_id_blocks` | PASS |
| PC-004 | integration | ecc-workflow binary ignores ECC_WORKFLOW_BYPASS=1 | AC-006.4 | `cargo test -p ecc-workflow --test transition -- bypass_env_var_ignored` | PASS |
| PC-005 | unit | BypassStore trait has check_token method (compiles) | AC-002.1 | `cargo test -p ecc-ports -- bypass_store_has_check_token` | PASS |
| PC-006 | unit | SqliteBypassStore::check_token() returns token when file exists | AC-002.2 | `cargo test -p ecc-infra -- sqlite_bypass_store_check_token_found` | PASS |
| PC-007 | unit | InMemoryBypassStore::check_token() returns pre-configured token | AC-002.3 | `cargo test -p ecc-test-support -- check_token_returns_matching_token` | PASS |
| PC-008 | unit | check_token() returns None for malformed JSON | AC-002.5 | `cargo test -p ecc-infra -- sqlite_bypass_store_check_token_malformed` | PASS |
| PC-009 | unit | check_token() returns None for mismatched hook_id | AC-002.6 | `cargo test -p ecc-infra -- sqlite_bypass_store_check_token_mismatched` | PASS |
| PC-010 | unit | check_token() returns None when HOME unset | AC-002.7 | `cargo test -p ecc-infra -- sqlite_bypass_store_check_token_no_home` | PASS |
| PC-011 | build | All crates compile with new trait method | AC-002.1 | `cargo build` | exit 0 |
| PC-012 | unit | HookPorts::test_default() compiles, returns struct with None optionals | AC-003.1 | `cargo test -p ecc-app -- tests::test_default_creates_ports` | PASS |
| PC-013 | unit | All handler tests pass after migration to test_default() | AC-003.2 | `cargo test -p ecc-app` | PASS |
| PC-014 | build | ecc-cli compiles with SqliteBypassStore wiring | AC-003.3 | `cargo build -p ecc-cli` | exit 0 |
| PC-015 | unit | BypassPolicy trait compiles with should_bypass method | AC-005.1 | `cargo test -p ecc-domain -- bypass_policy_trait_compiles` | PASS |
| PC-016 | unit | AlwaysDenyPolicy implements BypassPolicy and returns false | AC-005.2 | `cargo test -p ecc-app -- always_deny_policy_returns_false` | PASS |
| PC-017 | integration | ecc-workflow init with ECC_WORKFLOW_BYPASS=1 creates state normally | AC-001.2, AC-001.8 | `cargo test -p ecc-workflow --test transition -- bypass_env_var` | PASS |
| PC-018 | integration | All ecc-workflow tests pass without env_remove calls | AC-001.6 | `cargo test -p ecc-workflow` | PASS |
| PC-019 | unit | bypass_interceptor: token found returns passthrough (exit 0) | AC-004.1, AC-004.3 | `cargo test -p ecc-app -- bypass_interceptor::tests::token_found_passthrough` | PASS |
| PC-020 | unit | bypass_interceptor: token not found returns exit 2 | AC-004.1, AC-004.3 | `cargo test -p ecc-app -- bypass_interceptor::tests::token_not_found_blocks` | PASS |
| PC-021 | unit | bypass_interceptor: no session_id returns exit 2 | AC-004.1, AC-004.3 | `cargo test -p ecc-app -- bypass_interceptor::tests::no_session_id_blocks` | PASS |
| PC-022 | unit | bypass_interceptor records Applied decision in store | AC-004.3 | `cargo test -p ecc-app -- bypass_interceptor::tests::records_applied_decision` | PASS |
| PC-023 | unit | dispatch() with ECC_WORKFLOW_BYPASS=1 does NOT passthrough | AC-001.1 | `cargo test -p ecc-app -- tests::env_bypass_ignored` | PASS |
| PC-024 | unit | dispatch() delegates to bypass_interceptor when exit_code=2 | AC-004.2 | `cargo test -p ecc-app -- tests::dispatch_delegates_to_interceptor` | PASS |
| PC-025 | integration | Integration tests pass without env bypass setup | AC-001.11 | `cargo test -p ecc-integration-tests` | PASS |
| PC-026 | lint | Repo-wide grep: ECC_WORKFLOW_BYPASS only in historical locations | AC-001.4 | `! grep -r ECC_WORKFLOW_BYPASS --include='*.rs' --include='*.sh' --include='*.md' . \| grep -v docs/specs/ \| grep -v docs/adr/ \| grep -v docs/backlog/ \| grep -v CHANGELOG.md \| grep -v '.claude/workflow/'` | exit 0 |
| PC-027 | lint | .envrc does not exist | AC-001.3, AC-001.10 | `test ! -f .envrc` | exit 0 |
| PC-028 | lint | ADR-0056 contains "Completed" | AC-001.5 | `grep -q Completed docs/adr/0056-ecc-workflow-bypass-deprecation.md` | exit 0 |
| PC-029 | lint | test-phase-gate.sh has no test_bypass | AC-001.7 | `! grep -q test_bypass tests/hooks/test-phase-gate.sh` | exit 0 |
| PC-030 | lint | CLAUDE.md contains direnv revoke | AC-001.13 | `grep -q 'direnv revoke' CLAUDE.md` | exit 0 |
| PC-031 | lint | rules/ecc/development.md has no ECC_WORKFLOW_BYPASS | AC-001.12 | `! grep -q ECC_WORKFLOW_BYPASS rules/ecc/development.md` | exit 0 |
| PC-035 | lint | CLAUDE.md contains 'ecc bypass grant' | AC-001.9 | `grep -q 'ecc bypass grant' CLAUDE.md` | exit 0 |
| PC-036 | lint | commands/ecc-test-mode.md has no ECC_WORKFLOW_BYPASS | AC-001.12 | `! grep -q ECC_WORKFLOW_BYPASS commands/ecc-test-mode.md` | exit 0 |
| PC-037 | lint | commands/create-component.md has no ECC_WORKFLOW_BYPASS | AC-001.12 | `! grep -q ECC_WORKFLOW_BYPASS commands/create-component.md` | exit 0 |
| PC-038 | lint | skills/ecc-component-authoring/SKILL.md has no ECC_WORKFLOW_BYPASS | AC-001.12 | `! grep -q ECC_WORKFLOW_BYPASS skills/ecc-component-authoring/SKILL.md` | exit 0 |
| PC-039 | lint | patterns/agentic/guardrails.md has no ECC_WORKFLOW_BYPASS | AC-001.12 | `! grep -q ECC_WORKFLOW_BYPASS patterns/agentic/guardrails.md` | exit 0 |
| PC-040 | lint | .gitignore has no ECC_WORKFLOW_BYPASS reference | AC-001.4 | `! grep -q ECC_WORKFLOW_BYPASS .gitignore` | exit 0 |
| PC-032 | lint | Clippy clean | all | `cargo clippy -- -D warnings` | exit 0 |
| PC-033 | lint | Rustfmt check | all | `cargo fmt --check` | exit 0 |
| PC-034 | build | Full workspace build | all | `cargo build` | exit 0 |
| PC-041 | unit | Full test suite | all | `cargo test` | PASS |

### Coverage Check

All 32 ACs covered:
- AC-001.1: PC-023 | AC-001.2: PC-017 | AC-001.3: PC-027 | AC-001.4: PC-026,PC-040
- AC-001.5: PC-028 | AC-001.6: PC-018 | AC-001.7: PC-029 | AC-001.8: PC-017
- AC-001.9: PC-035 | AC-001.10: PC-027 | AC-001.11: PC-025 | AC-001.12: PC-031,036,037,038,039
- AC-001.13: PC-030 | AC-002.1: PC-005 | AC-002.2: PC-006 | AC-002.3: PC-007
- AC-002.4: PC-024 | AC-002.5: PC-008 | AC-002.6: PC-009 | AC-002.7: PC-010
- AC-003.1: PC-012 | AC-003.2: PC-013 | AC-003.3: PC-014
- AC-004.1: PC-019,020,021 | AC-004.2: PC-024 | AC-004.3: PC-019,020,021,022
- AC-005.1: PC-015 | AC-005.2: PC-016
- AC-006.1: PC-001 | AC-006.2: PC-002 | AC-006.3: PC-003 | AC-006.4: PC-004

Uncovered ACs: **none**.

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| E2E-1 | BypassStore.check_token() | SqliteBypassStore | BypassStore | Token file read+parse returns valid BypassToken | ignored | BypassStore adapter modified |
| E2E-2 | BypassStore.check_token() | SqliteBypassStore | BypassStore | Malformed token returns None | ignored | BypassStore adapter modified |
| E2E-3 | Hook dispatch | ecc hook CLI | HookPorts | Hook with bypass token grants passthrough | ignored | dispatch() or bypass_interceptor modified |
| E2E-4 | ecc-workflow binary | ecc-workflow | N/A | Binary runs normally with ECC_WORKFLOW_BYPASS=1 | ignored | ecc-workflow main.rs modified |

### E2E Activation Rules
All 4 E2E tests activated for this implementation (all boundaries are modified).

## Test Strategy

TDD order:
1. PC-001..004 (Phase 1: characterization tests — establish baseline)
2. PC-005..011 (Phase 2: check_token port — build from domain outward)
3. PC-012..014 (Phase 3: test_default — enables Phase 6 migration)
4. PC-015..016 (Phase 4: domain trait — independent)
5. PC-017..018 (Phase 5: ecc-workflow binary — standalone binary)
6. PC-019..025 (Phase 6: dispatch refactor — depends on Phase 2+3)
7. PC-026..031 (Phase 7: docs/config — after code changes)
8. PC-032..034 (Final gates — everything passes)

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| D-1 | CLAUDE.md | Project | Modify | Remove ECC_WORKFLOW_BYPASS from Gotchas, add direnv note | AC-001.9, AC-001.13 |
| D-2 | docs/adr/0056-*.md | ADR | Modify | Status → Completed | AC-001.5 |
| D-3 | CHANGELOG.md | Release | Modify | Add bypass consolidation entry | AC-001.9 |
| D-4 | crates/CLAUDE.md | Dev guide | Modify | Document HookPorts::test_default() | AC-003.1 |
| D-5 | rules/ecc/development.md | Rule | Modify | Remove bypass hook convention | AC-001.12 |
| D-6 | commands/ecc-test-mode.md | Command | Modify | Rewrite for token-based bypass | AC-001.12 |
| D-7 | commands/create-component.md | Command | Modify | Remove bypass from hook template | AC-001.12 |
| D-8 | skills/ecc-component-authoring/SKILL.md | Skill | Modify | Remove bypass convention | AC-001.12 |
| D-9 | patterns/agentic/guardrails.md | Pattern | Modify | Remove bypass reference | AC-001.12 |

## SOLID Assessment

**Uncle-bob verdict: NEEDS WORK → 2 fixes applied**

1. **FAIL → FIXED**: `SqliteBypassStore::check_token()` originally proposed using raw `std::fs`. Fixed: adapter will accept home_dir as constructor param, consistent with port abstraction. Token path resolution uses adapter-internal FS access (acceptable for infra layer) but no longer reads `std::env::var("HOME")` directly in the method.

2. **FAIL → FIXED**: `AlwaysDenyPolicy` originally in ecc-domain. Fixed: moved to ecc-app (policy implementations are application-layer behavior). `BypassPolicy` trait stays in ecc-domain.

Other SOLID checks: SRP (bypass_interceptor extraction improves SRP), OCP (BypassStore extended without modification via default method), DIP (CLI wires concrete adapter, app layer uses trait).

## Robert's Oath Check

**CLEAN** — No warnings. Design follows: test-first (characterization tests before changes), atomic commits (each phase independently mergeable), proof via 34 pass conditions, fearless competence (rollback strategy per commit).

## Security Notes

**2 CRITICAL + 1 HIGH → all addressed in design:**

1. **CRITICAL — Path traversal via hook_id**: `hook_id.replace(':', "__")` doesn't strip `/`, `..`. **Fix**: Add regex validation `^[a-z0-9:_-]+$` in domain `BypassToken::new()` and `BypassDecision::new()`. Added to F-003.

2. **CRITICAL — Path traversal via session_id**: `validate_session_id` only rejects empty/"unknown". **Fix**: Enforce `^[a-zA-Z0-9_-]{1,128}$` in `validate_session_id()`. Added to F-003.

3. **HIGH — SQL interpolation in prune()**: `format!("datetime('now', '-{} days')", older_than_days)`. Low practical risk (u64 input), but structurally wrong. **Note**: Out of scope for this spec (not in affected modules), but flagged for future fix.

## Rollback Plan

Reverse dependency order:
1. Revert F-027..F-018 (Phase 7: docs/config — independent)
2. Revert F-017..F-015 (Phase 6: dispatch refactor)
3. Revert F-014..F-012 (Phase 5: ecc-workflow binary)
4. Revert F-011..F-010 (Phase 4: domain trait)
5. Revert F-009..F-007 (Phase 3: test boilerplate)
6. Revert F-006..F-003 (Phase 2: check_token port)
7. Revert F-002..F-001 (Phase 1: pre-refactor tests — safe to keep)

Note: `.envrc` is gitignored. If needed: `echo 'export ECC_WORKFLOW_BYPASS=1' > .envrc`

## Bounded Contexts Affected

| Context | Role | Files Modified |
|---------|------|----------------|
| hook_runtime | service | ecc-domain/bypass.rs (BypassPolicy trait), ecc-app/hook/mod.rs (dispatch), ecc-app/hook/bypass_interceptor.rs (new) |
| bypass | value object | ecc-domain/bypass.rs (path validation), ecc-ports/bypass_store.rs (check_token) |

Other domain modules (not registered as bounded contexts):
- ecc-infra: sqlite_bypass_store.rs (adapter implementation)
- ecc-test-support: in_memory_bypass_store.rs (test double)
- ecc-cli: commands/hook.rs (composition root wiring)

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID (uncle-bob) | NEEDS WORK → 2 fixes applied | 2 |
| Robert (Oath) | CLEAN | 0 |
| Security | 2 CRITICAL + 1 HIGH → fixes applied | 3 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| AC Coverage | 88 | PASS | All 32 ACs covered by 41 PCs |
| Execution Order | 95 | PASS | TDD order respects all dependencies |
| Fragility | 78 | PASS | Pipe-based PCs work; path validation added |
| Rollback Adequacy | 88 | PASS | Reverse-dependency order, .envrc recreation doc'd |
| Architecture Compliance | 92 | PASS | Hex layers respected, DIP maintained |
| Blast Radius | 62 | PASS | ~56 files inherent to removing ubiquitous env var |
| Missing Pass Conditions | 90 | PASS | fmt check added, individual file PCs added |
| Doc Plan Completeness | 85 | PASS | CHANGELOG, ADR, all doc targets covered |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| F-001 | crates/ecc-app/src/hook/mod.rs (tests) | modify | AC-006.1-3 |
| F-002 | crates/ecc-workflow/tests/transition.rs | modify | AC-006.4 |
| F-003 | crates/ecc-domain/src/hook_runtime/bypass.rs | modify | AC-002.5-6, security |
| F-004 | crates/ecc-ports/src/bypass_store.rs | modify | AC-002.1 |
| F-005 | crates/ecc-infra/src/sqlite_bypass_store.rs | modify | AC-002.2,5,6,7 |
| F-006 | crates/ecc-test-support/src/in_memory_bypass_store.rs | modify | AC-002.3 |
| F-007 | crates/ecc-app/src/hook/mod.rs | modify | AC-003.1 |
| F-008 | 26 handler files | modify | AC-003.2 |
| F-009 | crates/ecc-cli/src/commands/hook.rs | modify | AC-003.3 |
| F-010 | crates/ecc-domain/src/hook_runtime/bypass.rs | modify | AC-005.1 |
| F-011 | crates/ecc-app/src/hook/ (new or mod.rs) | modify | AC-005.2 |
| F-012 | crates/ecc-workflow/src/main.rs | modify | AC-001.2,8 |
| F-013 | crates/ecc-workflow/tests/transition.rs | modify | AC-001.2,8 |
| F-014 | 8 ecc-workflow test files | modify | AC-001.6 |
| F-015 | crates/ecc-app/src/hook/bypass_interceptor.rs | create | AC-004.1, AC-002.4 |
| F-016 | crates/ecc-app/src/hook/mod.rs | modify | AC-001.1, AC-004.2 |
| F-017 | 4 integration test files | modify | AC-001.11 |
| F-018 | .envrc | delete | AC-001.3,10 |
| F-019..F-027 | CLAUDE.md, ADR, rules, commands, skills, patterns, tests, CHANGELOG | modify | AC-001.4-13 |
| F-028 | .gitignore | modify | AC-001.4 |
| F-029 | worktree_guard.rs | modify | AC-001.4 |
| F-030 | session_merge.rs | modify | AC-001.4 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-07-consolidate-bypass-to-baked-in/design.md | Full design |
| docs/specs/2026-04-07-consolidate-bypass-to-baked-in/spec.md | Full spec |
| docs/specs/2026-04-07-consolidate-bypass-to-baked-in/campaign.md | Grill-me decisions |
