# Tasks: Deterministic Task Synchronization (BL-075 + BL-072)

## Pass Conditions

- [ ] PC-001: TaskStatus FSM valid transitions | `cargo test --lib -p ecc-domain task::status::tests -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-002: TaskStatus FSM rejection cases | `cargo test --lib -p ecc-domain task::status::tests::rejects -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-003: Parser well-formed input | `cargo test --lib -p ecc-domain task::parser::tests -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-004: Parser multi-segment trails | `cargo test --lib -p ecc-domain task::parser::tests::multi_segment -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-005: Parser PostTdd entries | `cargo test --lib -p ecc-domain task::parser::tests::post_tdd -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-006: Parser malformed input errors | `cargo test --lib -p ecc-domain task::parser::tests::malformed -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-007: Parser old-format pipe separator | `cargo test --lib -p ecc-domain task::parser::tests::old_format -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-008: Parser empty tasks.md | `cargo test --lib -p ecc-domain task::parser::tests::empty -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-009: Updater append trail segment | `cargo test --lib -p ecc-domain task::updater::tests::append_trail -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-010: Updater done checkbox flip | `cargo test --lib -p ecc-domain task::updater::tests::done_checkbox -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-011: Updater reject invalid transition | `cargo test --lib -p ecc-domain task::updater::tests::reject_invalid -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-012: Updater entry not found | `cargo test --lib -p ecc-domain task::updater::tests::not_found -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-013: Updater PostTdd update by string ID | `cargo test --lib -p ecc-domain task::updater::tests::post_tdd_update -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-014: Updater PostTdd pending->done | `cargo test --lib -p ecc-domain task::updater::tests::post_tdd_done -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-015: Renderer generate from PCs | `cargo test --lib -p ecc-domain task::renderer::tests -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-016: Renderer dependency order preserved | `cargo test --lib -p ecc-domain task::renderer::tests::order -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-017: Sync subcommand valid output | `cargo test -p ecc-workflow tasks::tests::sync -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-018: Sync missing path | `cargo test -p ecc-workflow tasks::tests::sync_missing -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-019: Sync malformed input | `cargo test -p ecc-workflow tasks::tests::sync_malformed -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-020: Sync path traversal | `cargo test -p ecc-workflow tasks::tests::sync_traversal -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-021: Update subcommand atomic write | `cargo test -p ecc-workflow tasks::tests::update_atomic -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-022: Update path traversal | `cargo test -p ecc-workflow tasks::tests::update_traversal -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-023: Init subcommand generate from design | `cargo test -p ecc-workflow tasks::tests::init_generate -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-024: Init existing output blocked | `cargo test -p ecc-workflow tasks::tests::init_exists -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-025: Init overwrite with --force | `cargo test -p ecc-workflow tasks::tests::init_force -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-026: Init no PC table | `cargo test -p ecc-workflow tasks::tests::init_no_pcs -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-027: Init duplicate PCs | `cargo test -p ecc-workflow tasks::tests::init_dup_pcs -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-028: TaskStatus serde serialization | `cargo test --lib -p ecc-domain task::status::tests::serde_format -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-029: Updater same-state rejection | `cargo test --lib -p ecc-domain task::updater::tests::same_state -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-030: Parser/updater error_detail | `cargo test --lib -p ecc-domain task::parser::tests::failed_detail -- --nocapture` | pending@2026-03-29T13:59:20Z
- [ ] PC-031: Clippy lint zero warnings | `cargo clippy --workspace -- -D warnings` | pending@2026-03-29T13:59:20Z
- [ ] PC-032: Release build succeeds | `cargo build --release` | pending@2026-03-29T13:59:20Z

## Post-TDD

- [ ] E2E tests | pending@2026-03-29T13:59:20Z
- [ ] Code review | pending@2026-03-29T13:59:20Z
- [ ] Doc updates | pending@2026-03-29T13:59:20Z
- [ ] Supplemental docs | pending@2026-03-29T13:59:20Z
- [ ] Write implement-done.md | pending@2026-03-29T13:59:20Z
