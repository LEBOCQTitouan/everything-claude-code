# Tasks: Consolidate Bypass to Baked-In Auditable System

## Pass Conditions

### Phase 1: Pre-Refactor Tests (US-006)
- [ ] PC-001: Dispatch with bypass token file present returns exit 0 passthrough | `cargo test -p ecc-app -- tests::bypass_token_found_passthrough` | PASS
- [ ] PC-002: Dispatch with no token returns exit 2 with hint | `cargo test -p ecc-app -- tests::bypass_token_not_found_blocks` | PASS
- [ ] PC-003: Dispatch with no CLAUDE_SESSION_ID returns exit 2 | `cargo test -p ecc-app -- tests::no_session_id_blocks` | PASS
- [ ] PC-004: ecc-workflow binary ignores ECC_WORKFLOW_BYPASS=1 | `cargo test -p ecc-workflow --test transition -- bypass_env_var_ignored` | PASS

### Phase 2: check_token() Port (US-002)
- [ ] PC-005: BypassStore trait has check_token method | `cargo test -p ecc-ports -- bypass_store_has_check_token` | PASS
- [ ] PC-006: SqliteBypassStore::check_token() returns token when file exists | `cargo test -p ecc-infra -- sqlite_bypass_store_check_token_found` | PASS
- [ ] PC-007: InMemoryBypassStore::check_token() returns pre-configured token | `cargo test -p ecc-test-support -- check_token_returns_matching_token` | PASS
- [ ] PC-008: check_token() returns None for malformed JSON | `cargo test -p ecc-infra -- sqlite_bypass_store_check_token_malformed` | PASS
- [ ] PC-009: check_token() returns None for mismatched hook_id | `cargo test -p ecc-infra -- sqlite_bypass_store_check_token_mismatched` | PASS
- [ ] PC-010: check_token() returns None when HOME unset | `cargo test -p ecc-infra -- sqlite_bypass_store_check_token_no_home` | PASS
- [ ] PC-011: All crates compile with new trait method | `cargo build` | exit 0

### Phase 3: Test Boilerplate (US-003)
- [ ] PC-012: HookPorts::test_default() compiles | `cargo test -p ecc-app -- tests::test_default_creates_ports` | PASS
- [ ] PC-013: All handler tests pass after migration | `cargo test -p ecc-app` | PASS
- [ ] PC-014: ecc-cli compiles with SqliteBypassStore wiring | `cargo build -p ecc-cli` | exit 0

### Phase 4: Domain Abstractness (US-005)
- [ ] PC-015: BypassPolicy trait compiles | `cargo test -p ecc-domain -- bypass_policy_trait_compiles` | PASS
- [ ] PC-016: AlwaysDenyPolicy implements BypassPolicy | `cargo test -p ecc-app -- always_deny_policy_returns_false` | PASS

### Phase 5: ecc-workflow Binary Cleanup (US-001 partial)
- [ ] PC-017: ecc-workflow init with ECC_WORKFLOW_BYPASS=1 creates state normally | `cargo test -p ecc-workflow --test transition -- bypass_env_var` | PASS
- [ ] PC-018: All ecc-workflow tests pass without env_remove calls | `cargo test -p ecc-workflow` | PASS

### Phase 6: dispatch() Refactor (US-001 + US-004)
- [ ] PC-019: bypass_interceptor: token found returns passthrough | `cargo test -p ecc-app -- bypass_interceptor::tests::token_found_passthrough` | PASS
- [ ] PC-020: bypass_interceptor: token not found returns exit 2 | `cargo test -p ecc-app -- bypass_interceptor::tests::token_not_found_blocks` | PASS
- [ ] PC-021: bypass_interceptor: no session_id returns exit 2 | `cargo test -p ecc-app -- bypass_interceptor::tests::no_session_id_blocks` | PASS
- [ ] PC-022: bypass_interceptor records Applied decision | `cargo test -p ecc-app -- bypass_interceptor::tests::records_applied_decision` | PASS
- [ ] PC-023: dispatch() with ECC_WORKFLOW_BYPASS=1 does NOT passthrough | `cargo test -p ecc-app -- tests::env_bypass_ignored` | PASS
- [ ] PC-024: dispatch() delegates to bypass_interceptor | `cargo test -p ecc-app -- tests::dispatch_delegates_to_interceptor` | PASS
- [ ] PC-025: Integration tests pass without env bypass setup | `cargo test -p ecc-integration-tests` | PASS

### Phase 7: Docs + Config
- [ ] PC-026: Repo-wide grep: ECC_WORKFLOW_BYPASS only in historical locations | grep check | exit 0
- [ ] PC-027: .envrc does not exist | `test ! -f .envrc` | exit 0
- [ ] PC-028: ADR-0056 contains "Completed" | grep check | exit 0
- [ ] PC-029: test-phase-gate.sh has no test_bypass | grep check | exit 0
- [ ] PC-030: CLAUDE.md contains direnv revoke | grep check | exit 0
- [ ] PC-031: rules/ecc/development.md has no ECC_WORKFLOW_BYPASS | grep check | exit 0
- [ ] PC-035: CLAUDE.md contains 'ecc bypass grant' | grep check | exit 0
- [ ] PC-036: commands/ecc-test-mode.md has no ECC_WORKFLOW_BYPASS | grep check | exit 0
- [ ] PC-037: commands/create-component.md has no ECC_WORKFLOW_BYPASS | grep check | exit 0
- [ ] PC-038: skills/ecc-component-authoring/SKILL.md has no ECC_WORKFLOW_BYPASS | grep check | exit 0
- [ ] PC-039: patterns/agentic/guardrails.md has no ECC_WORKFLOW_BYPASS | grep check | exit 0
- [ ] PC-040: .gitignore has no ECC_WORKFLOW_BYPASS reference | grep check | exit 0

### Final Gates
- [ ] PC-032: Clippy clean | `cargo clippy -- -D warnings` | exit 0
- [ ] PC-033: Rustfmt check | `cargo fmt --check` | exit 0
- [ ] PC-034: Full workspace build | `cargo build` | exit 0
- [ ] PC-041: Full test suite | `cargo test` | PASS

### Post-TDD
- [ ] E2E tests
- [ ] Code review
- [ ] Doc updates
- [ ] Supplemental docs
- [ ] Write implement-done.md

## Status Trail
<!-- Status updates appended below -->
