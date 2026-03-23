# Tasks: Symlink-Based Instant Config Switching (BL-058)

## Pass Conditions

- [ ] PC-001: DevProfile enum variants | `cargo test -p ecc-domain dev_profile::tests::dev_profile_enum_variants` | pending@2026-03-23T14:20:00Z
- [ ] PC-002: SymlinkPlan structure | `cargo test -p ecc-domain dev_profile::tests::symlink_plan_structure` | pending@2026-03-23T14:20:00Z
- [ ] PC-003: DevProfile usable from CLI | `cargo build -p ecc-cli` | pending@2026-03-23T14:20:00Z
- [ ] PC-004: build_symlink_plan Dev profile | `cargo test -p ecc-domain dev_profile::tests::build_plan_dev_profile` | pending@2026-03-23T14:20:00Z
- [ ] PC-005: build_symlink_plan Default profile | `cargo test -p ecc-domain dev_profile::tests::build_plan_default_profile` | pending@2026-03-23T14:20:00Z
- [ ] PC-006: Domain zero I/O imports | `! grep -rn 'use std::fs\|use std::process\|use std::net' crates/ecc-domain/src/` | pending@2026-03-23T14:20:00Z
- [ ] PC-007: MANAGED_DIRS constant | `cargo test -p ecc-domain dev_profile::tests::managed_dirs_constant` | pending@2026-03-23T14:20:00Z
- [ ] PC-008: FileSystem trait compiles | `cargo build -p ecc-ports` | pending@2026-03-23T14:20:00Z
- [ ] PC-009: All implementors compile | `cargo build -p ecc-infra -p ecc-test-support` | pending@2026-03-23T14:20:00Z
- [ ] PC-010: OsFileSystem create_symlink | `grep -q 'unix::fs::symlink' crates/ecc-infra/src/os_fs.rs && cargo build -p ecc-infra` | pending@2026-03-23T14:20:00Z
- [ ] PC-011: OsFileSystem read_symlink | `grep -q 'read_link' crates/ecc-infra/src/os_fs.rs && cargo build -p ecc-infra` | pending@2026-03-23T14:20:00Z
- [ ] PC-012: OsFileSystem is_symlink | `grep -q 'symlink_metadata' crates/ecc-infra/src/os_fs.rs && cargo build -p ecc-infra` | pending@2026-03-23T14:20:00Z
- [ ] PC-013: create_symlink removes existing | `grep -q 'remove_file' crates/ecc-infra/src/os_fs.rs && cargo build -p ecc-infra` | pending@2026-03-23T14:20:00Z
- [ ] PC-014: Non-Unix cfg Unsupported | `grep -q 'cfg(not(unix))' crates/ecc-infra/src/os_fs.rs` | pending@2026-03-23T14:20:00Z
- [ ] PC-015: InMemoryFileSystem symlinks field | `grep -q 'symlinks.*BTreeMap' crates/ecc-test-support/src/in_memory_fs.rs` | pending@2026-03-23T14:20:00Z
- [ ] PC-016: create_symlink inserts/removes | `cargo test -p ecc-test-support in_memory_fs::tests::create_symlink_inserts_and_removes_file` | pending@2026-03-23T14:20:00Z
- [ ] PC-017: read_symlink | `cargo test -p ecc-test-support in_memory_fs::tests::read_symlink` | pending@2026-03-23T14:20:00Z
- [ ] PC-018: is_symlink detection | `cargo test -p ecc-test-support in_memory_fs::tests::is_symlink_detection` | pending@2026-03-23T14:20:00Z
- [ ] PC-019: exists includes symlinks | `cargo test -p ecc-test-support in_memory_fs::tests::exists_includes_symlinks` | pending@2026-03-23T14:20:00Z
- [ ] PC-020: remove_file removes symlink | `cargo test -p ecc-test-support in_memory_fs::tests::remove_file_removes_symlink` | pending@2026-03-23T14:20:00Z
- [ ] PC-021: with_symlink builder | `cargo test -p ecc-test-support in_memory_fs::tests::with_symlink_builder` | pending@2026-03-23T14:20:00Z
- [ ] PC-022: Existing tests pass | `cargo test -p ecc-test-support` | pending@2026-03-23T14:20:00Z
- [ ] PC-023: dev_switch exists | `grep -q 'pub fn dev_switch' crates/ecc-app/src/dev.rs` | pending@2026-03-23T14:20:00Z
- [ ] PC-024: dev_switch(Dev) creates symlinks | `cargo test -p ecc-app dev::tests::dev_switch_dev_creates_symlinks` | pending@2026-03-23T14:20:00Z
- [ ] PC-025: dev_switch(Default) restores | `cargo test -p ecc-app dev::tests::dev_switch_default_restores_copies` | pending@2026-03-23T14:20:00Z
- [ ] PC-026: dev_switch dry_run | `cargo test -p ecc-app dev::tests::dev_switch_dry_run` | pending@2026-03-23T14:20:00Z
- [ ] PC-027: dev_switch rollback | `cargo test -p ecc-app dev::tests::dev_switch_rollback_on_error` | pending@2026-03-23T14:20:00Z
- [ ] PC-028: validates targets in ECC_ROOT | `cargo test -p ecc-app dev::tests::dev_switch_validates_targets_within_ecc_root` | pending@2026-03-23T14:20:00Z
- [ ] PC-029: tested with InMemoryFileSystem | `cargo test -p ecc-app dev::tests::dev_switch` | pending@2026-03-23T14:20:00Z
- [ ] PC-030: uses MANAGED_DIRS | `grep -q 'MANAGED_DIRS' crates/ecc-app/src/dev.rs` | pending@2026-03-23T14:20:00Z
- [ ] PC-031: target dirs must exist | `cargo test -p ecc-app dev::tests::dev_switch_dev_target_must_exist` | pending@2026-03-23T14:20:00Z
- [ ] PC-032: dev_off symlink safety | `cargo test -p ecc-app dev::tests::dev_off_removes_symlinks_safely` | pending@2026-03-23T14:20:00Z
- [ ] PC-033: absolute paths | `cargo test -p ecc-app dev::tests::dev_switch_uses_absolute_paths` | pending@2026-03-23T14:20:00Z
- [ ] PC-034: manifest preservation | `cargo test -p ecc-app dev::tests::dev_switch_manifest_preservation` | pending@2026-03-23T14:20:00Z
- [ ] PC-035: dangling symlinks | `cargo test -p ecc-app dev::tests::dev_switch_handles_dangling_symlinks` | pending@2026-03-23T14:20:00Z
- [ ] PC-036: removes existing dirs | `cargo test -p ecc-app dev::tests::dev_switch_removes_existing_dirs` | pending@2026-03-23T14:20:00Z
- [ ] PC-037: status Dev (symlinked) | `cargo test -p ecc-app dev::tests::dev_status_symlinked_profile` | pending@2026-03-23T14:20:00Z
- [ ] PC-038: status Default (copied) | `cargo test -p ecc-app dev::tests::dev_status_copied_profile` | pending@2026-03-23T14:20:00Z
- [ ] PC-039: format_status Profile line | `cargo test -p ecc-app dev::tests::format_status_includes_profile` | pending@2026-03-23T14:20:00Z
- [ ] PC-040: status Inactive | `cargo test -p ecc-app dev::tests::dev_status_inactive_no_errors` | pending@2026-03-23T14:20:00Z
- [ ] PC-041: all three states tested | `cargo test -p ecc-app dev::tests::dev_status` | pending@2026-03-23T14:20:00Z
- [ ] PC-042: mixed state | `cargo test -p ecc-app dev::tests::dev_status_mixed_state` | pending@2026-03-23T14:20:00Z
- [ ] PC-043: DevAction::Switch compiles | `cargo build -p ecc-cli` | pending@2026-03-23T14:20:00Z
- [ ] PC-044: Switch calls dev_switch | `grep -q 'DevAction::Switch' crates/ecc-cli/src/commands/dev.rs && grep -q 'dev_switch' crates/ecc-cli/src/commands/dev.rs` | pending@2026-03-23T14:20:00Z
- [ ] PC-045: existing actions unchanged | `cargo test -p ecc-cli` | pending@2026-03-23T14:20:00Z
- [ ] PC-046: CLI builds with Switch | `cargo build -p ecc-cli` | pending@2026-03-23T14:20:00Z
- [ ] PC-047: error propagation | `cargo test -p ecc-app dev::tests::dev_switch_error_returns_failure` | pending@2026-03-23T14:20:00Z
- [ ] PC-048: all tests pass | `cargo test` | pending@2026-03-23T14:20:00Z
- [ ] PC-049: zero clippy warnings | `cargo clippy -- -D warnings` | pending@2026-03-23T14:20:00Z
- [ ] PC-050: domain zero I/O | `! grep -rn 'use std::fs\|use std::process\|use std::net' crates/ecc-domain/src/` | pending@2026-03-23T14:20:00Z
- [ ] PC-051: workspace builds | `cargo build` | pending@2026-03-23T14:20:00Z

## Post-TDD

- [ ] E2E tests | pending@2026-03-23T14:20:00Z
- [ ] Code review | pending@2026-03-23T14:20:00Z
- [ ] Doc updates | pending@2026-03-23T14:20:00Z
- [ ] Write implement-done.md | pending@2026-03-23T14:20:00Z
