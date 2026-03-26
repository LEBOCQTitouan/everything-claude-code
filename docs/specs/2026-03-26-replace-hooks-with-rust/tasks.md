# Tasks: BL-052 Replace Shell Hooks with Compiled Rust Binaries

## Pass Conditions

- [ ] PC-001: Phase VO with four variants | `cargo test -p ecc-domain --lib workflow::phase` | pending@2026-03-26T23:00:00Z
- [ ] PC-005: Illegal transition returns domain error | `cargo test -p ecc-domain --lib workflow::transition::tests::illegal_transitions` | pending@2026-03-26T23:00:00Z
- [ ] PC-003: Legal transitions allowed | `cargo test -p ecc-domain --lib workflow::transition::tests::legal_transitions` | pending@2026-03-26T23:00:00Z
- [ ] PC-004: Phase aliases accepted | `cargo test -p ecc-domain --lib workflow::transition::tests::alias_transitions` | pending@2026-03-26T23:00:00Z
- [ ] PC-007: Idempotent re-entry | `cargo test -p ecc-domain --lib workflow::transition::tests::reentry_transitions` | pending@2026-03-26T23:00:00Z
- [ ] PC-002: WorkflowState aggregate fields | `cargo test -p ecc-domain --lib workflow::state::tests::creates_workflow_state_with_all_fields` | pending@2026-03-26T23:00:00Z
- [ ] PC-006: JSON round-trip matches state.json | `cargo test -p ecc-domain --lib workflow::state::tests::json_round_trip` | pending@2026-03-26T23:00:00Z
- [ ] PC-008: Corrupted JSON returns domain error | `cargo test -p ecc-domain --lib workflow::state::tests::corrupted_json` | pending@2026-03-26T23:00:00Z
- [ ] PC-009: Domain purity lint | `! grep -rn 'use std::fs\|use std::process\|use std::net\|use tokio' crates/ecc-domain/src/` | pending@2026-03-26T23:00:00Z
- [ ] PC-010: Binary compiles | `cargo build -p ecc-workflow && test -f target/debug/ecc-workflow` | pending@2026-03-26T23:00:00Z
- [ ] PC-016: No bash/sh in production code | `! grep -rn 'Command::new("bash")\|Command::new("sh")' crates/ecc-workflow/src/` | pending@2026-03-26T23:00:00Z
- [ ] PC-015: Structured JSON output | `cargo test -p ecc-workflow --test integration output_is_structured_json` | pending@2026-03-26T23:00:00Z
- [ ] PC-014: Missing state.json exits 0 with warning | `cargo test -p ecc-workflow --test integration missing_state_exits_zero_with_warning` | pending@2026-03-26T23:00:00Z
- [ ] PC-011: init creates state.json | `cargo test -p ecc-workflow --test integration init_creates_state_json` | pending@2026-03-26T23:00:00Z
- [ ] PC-012: transition updates state.json | `cargo test -p ecc-workflow --test integration transition_updates_state` | pending@2026-03-26T23:00:00Z
- [ ] PC-013: Illegal transition non-zero exit | `cargo test -p ecc-workflow --test integration transition_illegal_exits_nonzero` | pending@2026-03-26T23:00:00Z
- [ ] PC-017: Dual invocation mode | `cargo test -p ecc-workflow --test integration dual_invocation` | pending@2026-03-26T23:00:00Z
- [ ] PC-031: ECC_WORKFLOW_BYPASS skips all | `cargo test -p ecc-workflow --test integration bypass_env_var` | pending@2026-03-26T23:00:00Z
- [ ] PC-020: toolchain-persist | `cargo test -p ecc-workflow --test integration toolchain_persist` | pending@2026-03-26T23:00:00Z
- [ ] PC-018: init matches shell behavior | `cargo test -p ecc-workflow --test integration init_matches_shell` | pending@2026-03-26T23:00:00Z
- [ ] PC-019: transition full sequence | `cargo test -p ecc-workflow --test integration transition_full_sequence` | pending@2026-03-26T23:00:00Z
- [ ] PC-032: transition writes memory | `cargo test -p ecc-workflow --test integration transition_writes_memory` | pending@2026-03-26T23:00:00Z
- [ ] PC-021: memory-write subcommands | `cargo test -p ecc-workflow --test integration memory_write_subcommands` | pending@2026-03-26T23:00:00Z
- [ ] PC-022: phase-gate | `cargo test -p ecc-workflow --test integration phase_gate` | pending@2026-03-26T23:00:00Z
- [ ] PC-023: stop-gate | `cargo test -p ecc-workflow --test integration stop_gate` | pending@2026-03-26T23:00:00Z
- [ ] PC-024: grill-me-gate | `cargo test -p ecc-workflow --test integration grill_me_gate` | pending@2026-03-26T23:00:00Z
- [ ] PC-025: tdd-enforcement | `cargo test -p ecc-workflow --test integration tdd_enforcement` | pending@2026-03-26T23:00:00Z
- [ ] PC-026: scope-check | `cargo test -p ecc-workflow --test integration scope_check` | pending@2026-03-26T23:00:00Z
- [ ] PC-027: doc-enforcement | `cargo test -p ecc-workflow --test integration doc_enforcement` | pending@2026-03-26T23:00:00Z
- [ ] PC-028: doc-level-check | `cargo test -p ecc-workflow --test integration doc_level_check` | pending@2026-03-26T23:00:00Z
- [ ] PC-029: pass-condition-check | `cargo test -p ecc-workflow --test integration pass_condition_check` | pending@2026-03-26T23:00:00Z
- [ ] PC-030: e2e-boundary-check | `cargo test -p ecc-workflow --test integration e2e_boundary_check` | pending@2026-03-26T23:00:00Z
- [ ] PC-033: No bash hooks in commands | `! grep -rn 'bash \.claude/hooks/' commands/` | pending@2026-03-26T23:00:00Z
- [ ] PC-034: No bash hooks in skills | `! grep -rn 'bash \.claude/hooks/' skills/` | pending@2026-03-26T23:00:00Z
- [ ] PC-035: hooks.json references ecc-workflow | `! grep -n 'bash \.claude/hooks/' hooks/hooks.json .claude/settings.json` | pending@2026-03-26T23:00:00Z
- [ ] PC-036: ecc-workflow in commands | `grep -c 'ecc-workflow' commands/spec-dev.md commands/spec-fix.md commands/spec-refactor.md commands/design.md commands/implement.md` | pending@2026-03-26T23:00:00Z
- [ ] PC-037: No .sh files in hooks | `test "$(find .claude/hooks/ -name '*.sh' | wc -l | tr -d ' ')" = "0"` | pending@2026-03-26T23:00:00Z
- [ ] PC-038: CLAUDE.md references ecc-workflow | `grep -q 'ecc-workflow' CLAUDE.md` | pending@2026-03-26T23:00:00Z
- [ ] PC-039: Glossary defines WorkflowState/Phase | `grep -q 'WorkflowState' docs/domain/glossary.md && grep -q 'Phase' docs/domain/glossary.md` | pending@2026-03-26T23:00:00Z
- [ ] PC-040: Test count updated | `grep -oE '[0-9]+ tests' CLAUDE.md` | pending@2026-03-26T23:00:00Z
- [ ] PC-044: ARCHITECTURE.md mentions ecc-workflow | `grep -c 'ecc-workflow' docs/ARCHITECTURE.md && grep -q '8 crates\|eight crates' docs/ARCHITECTURE.md` | pending@2026-03-26T23:00:00Z
- [ ] PC-045: Stale workflow archiving | `cargo test -p ecc-workflow --test stale_archive` | pending@2026-03-26T23:00:00Z
- [ ] PC-046: Fixture-based equivalence tests | `grep -rL 'bash\|\.sh' crates/ecc-workflow/tests/ || echo "clean"` | pending@2026-03-26T23:00:00Z
- [ ] PC-041: clippy clean | `cargo clippy -- -D warnings` | pending@2026-03-26T23:00:00Z
- [ ] PC-042: cargo build succeeds | `cargo build` | pending@2026-03-26T23:00:00Z
- [ ] PC-043: All tests pass | `cargo test` | pending@2026-03-26T23:00:00Z

## Post-TDD

- [ ] E2E tests | pending@2026-03-26T23:00:00Z
- [ ] Code review | pending@2026-03-26T23:00:00Z
- [ ] Doc updates | pending@2026-03-26T23:00:00Z
- [ ] Supplemental docs | pending@2026-03-26T23:00:00Z
- [ ] Write implement-done.md | pending@2026-03-26T23:00:00Z
