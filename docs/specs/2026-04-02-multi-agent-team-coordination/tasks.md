# Tasks: Multi-Agent Team Coordination (BL-104)

## Pass Conditions

- [ ] PC-001: parse valid manifest | `cargo test -p ecc-domain config::team::tests::parses_valid_manifest` | pending
- [ ] PC-002: reject missing frontmatter | `cargo test -p ecc-domain config::team::tests::rejects_missing_frontmatter` | pending
- [ ] PC-003: reject unclosed frontmatter | `cargo test -p ecc-domain config::team::tests::rejects_unclosed_frontmatter` | pending
- [ ] PC-004: reject empty agents | `cargo test -p ecc-domain config::team::tests::rejects_empty_agents` | pending
- [ ] PC-005: reject unknown strategy | `cargo test -p ecc-domain config::team::tests::rejects_unknown_strategy` | pending
- [ ] PC-006: reject duplicate agent | `cargo test -p ecc-domain config::team::tests::rejects_duplicate_agent` | pending
- [ ] PC-007: reject zero max_concurrent | `cargo test -p ecc-domain config::team::tests::rejects_zero_max_concurrent` | pending
- [ ] PC-008: valid manifest passes | `cargo test -p ecc-domain config::team::tests::valid_manifest_passes` | pending
- [ ] PC-009: allowed_tools defaults None | `cargo test -p ecc-domain config::team::tests::allowed_tools_defaults_none` | pending
- [ ] PC-010: no dir succeeds | `cargo test -p ecc-app validate::teams::tests::no_dir_succeeds` | pending
- [ ] PC-011: reject unknown agent | `cargo test -p ecc-app validate::teams::tests::rejects_unknown_agent` | pending
- [ ] PC-012: warn tool escalation | `cargo test -p ecc-app validate::teams::tests::warns_on_tool_escalation` | pending
- [ ] PC-013: valid manifest passes | `cargo test -p ecc-app validate::teams::tests::valid_manifest_passes` | pending
- [ ] PC-014: parse error with path | `cargo test -p ecc-app validate::teams::tests::reports_parse_error_with_path` | pending
- [ ] PC-015: CLI teams target | `cargo test -p ecc-cli -- teams_target` | pending
- [ ] PC-016: install teams dir | `cargo test -p ecc-app -- installs_teams` | pending
- [ ] PC-017: implement-team.md valid | `ecc validate teams` | pending
- [ ] PC-018: audit-team.md valid | `ecc validate teams` | pending
- [ ] PC-019: review-team.md valid | `ecc validate teams` | pending
- [ ] PC-020: shared-state-protocol | `ecc validate skills` | pending
- [ ] PC-021: task-handoff | `ecc validate skills` | pending
- [ ] PC-022-026: command integration | [manual review] | pending
- [ ] PC-027: clippy | `cargo clippy --workspace -- -D warnings` | pending
- [ ] PC-028: build | `cargo build --workspace` | pending
- [ ] PC-029: regression | `cargo test --workspace --exclude xtask` | pending
- [ ] PC-030: fmt | `cargo fmt --all -- --check` | pending

## Post-TDD

- [ ] Code review | pending
- [ ] Doc updates | pending
- [ ] Write implement-done.md | pending
