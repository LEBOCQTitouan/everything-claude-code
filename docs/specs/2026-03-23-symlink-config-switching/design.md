# Solution: Symlink-Based Instant Config Switching (BL-058)

## Spec Reference
Concern: dev, Feature: BL-058 symlink-based config switching for ecc dev switch command

## File Changes (dependency order)
| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/config/dev_profile.rs` | create | `DevProfile` enum (Default/Dev) with Debug/Clone/PartialEq/Eq derives (NO clap — pure domain). `SymlinkOp`, `SymlinkPlan`, `MANAGED_DIRS` constant, `build_symlink_plan()` pure function. | US-001, AC-001.1–001.7 |
| 2 | `crates/ecc-domain/src/config/mod.rs` | modify | Add `pub mod dev_profile;` | US-001 |
| 3 | `crates/ecc-ports/src/fs.rs` | modify | Add `create_symlink`, `read_symlink`, `is_symlink` to `FileSystem` trait. Add `FsError::Unsupported(String)` variant. | US-002, AC-002.1–002.4 |
| 4 | `crates/ecc-test-support/src/in_memory_fs.rs` | modify | Add `symlinks: Arc<Mutex<BTreeMap<PathBuf, PathBuf>>>` field. Implement all 3 trait methods. Update `exists`, `remove_file`. Add `with_symlink` builder. | US-004, AC-004.1–004.8 |
| 5 | `crates/ecc-infra/src/os_fs.rs` | modify | Implement symlink methods with `#[cfg(unix)]`/`#[cfg(not(unix))]`. `create_symlink` removes existing file/symlink before creating. | US-003, AC-003.1–003.5 |
| 6 | `crates/ecc-app/src/dev.rs` | modify | Add `dev_switch` use case with path canonicalization, target validation, rollback tracker, dry-run. Enhance `dev_status` with profile detection. Update `dev_off` to handle symlinked state safely (remove_file, not remove_dir_all). | US-005, AC-005.1–005.14, US-006, AC-006.1–006.6 |
| 7 | `crates/ecc-cli/src/commands/dev.rs` | modify | Add `DevAction::Switch` variant. Newtype wrapper for `DevProfile` with `clap::ValueEnum` (domain stays clap-free). Wire to `dev::dev_switch`. | US-007, AC-007.1–007.7 |
| 8 | `CLAUDE.md` | modify | Update CLI Commands table with `switch`. Update test count. | US-008, AC-008.4 |

## Pass Conditions
| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | DevProfile enum has Default and Dev with correct derives | AC-001.1 | `cargo test -p ecc-domain dev_profile::tests::dev_profile_enum_variants` | pass |
| PC-002 | unit | SymlinkPlan contains Vec<SymlinkOp> with target and link | AC-001.2 | `cargo test -p ecc-domain dev_profile::tests::symlink_plan_structure` | pass |
| PC-003 | build | DevProfile usable from CLI (newtype in ecc-cli) | AC-001.3 | `cargo build -p ecc-cli` | exit 0 |
| PC-004 | unit | build_symlink_plan correct for Dev profile | AC-001.4, AC-001.6 | `cargo test -p ecc-domain dev_profile::tests::build_plan_dev_profile` | pass |
| PC-005 | unit | build_symlink_plan correct for Default profile | AC-001.4, AC-001.6 | `cargo test -p ecc-domain dev_profile::tests::build_plan_default_profile` | pass |
| PC-006 | lint | ecc-domain has zero I/O imports | AC-001.5, AC-008.3 | `! grep -rn 'use std::fs\|use std::process\|use std::net' crates/ecc-domain/src/` | exit 0 |
| PC-007 | unit | MANAGED_DIRS = agents, commands, skills, rules | AC-001.7 | `cargo test -p ecc-domain dev_profile::tests::managed_dirs_constant` | pass |
| PC-008 | build | FileSystem trait compiles with 3 new methods | AC-002.1, AC-002.2, AC-002.3 | `cargo build -p ecc-ports` | exit 0 |
| PC-009 | build | All implementors compile | AC-002.4 | `cargo build -p ecc-infra -p ecc-test-support` | exit 0 |
| PC-010 | build | OsFileSystem create_symlink uses unix symlink | AC-003.1 | `grep -q 'unix::fs::symlink' crates/ecc-infra/src/os_fs.rs && cargo build -p ecc-infra` | exit 0 |
| PC-011 | build | OsFileSystem read_symlink uses read_link | AC-003.2 | `grep -q 'read_link' crates/ecc-infra/src/os_fs.rs && cargo build -p ecc-infra` | exit 0 |
| PC-012 | build | OsFileSystem is_symlink uses symlink_metadata | AC-003.3 | `grep -q 'symlink_metadata' crates/ecc-infra/src/os_fs.rs && cargo build -p ecc-infra` | exit 0 |
| PC-013 | build | create_symlink removes existing before creating | AC-003.4 | `grep -q 'remove_file' crates/ecc-infra/src/os_fs.rs && cargo build -p ecc-infra` | exit 0 |
| PC-014 | lint | Non-Unix cfg returns Unsupported | AC-003.5 | `grep -q 'cfg(not(unix))' crates/ecc-infra/src/os_fs.rs` | exit 0 |
| PC-015 | lint | InMemoryFileSystem has symlinks BTreeMap | AC-004.1 | `grep -q 'symlinks.*BTreeMap' crates/ecc-test-support/src/in_memory_fs.rs` | exit 0 |
| PC-016 | unit | create_symlink inserts to symlinks, removes from files | AC-004.2 | `cargo test -p ecc-test-support in_memory_fs::tests::create_symlink_inserts_and_removes_file` | pass |
| PC-017 | unit | read_symlink returns target or NotFound | AC-004.3 | `cargo test -p ecc-test-support in_memory_fs::tests::read_symlink` | pass |
| PC-018 | unit | is_symlink detection | AC-004.4 | `cargo test -p ecc-test-support in_memory_fs::tests::is_symlink_detection` | pass |
| PC-019 | unit | exists includes symlinks | AC-004.5 | `cargo test -p ecc-test-support in_memory_fs::tests::exists_includes_symlinks` | pass |
| PC-020 | unit | remove_file removes symlink | AC-004.6 | `cargo test -p ecc-test-support in_memory_fs::tests::remove_file_removes_symlink` | pass |
| PC-021 | unit | with_symlink builder | AC-004.7 | `cargo test -p ecc-test-support in_memory_fs::tests::with_symlink_builder` | pass |
| PC-022 | unit | Existing InMemoryFileSystem tests pass | AC-004.8 | `cargo test -p ecc-test-support` | pass |
| PC-023 | lint | dev_switch function exists | AC-005.1 | `grep -q 'pub fn dev_switch' crates/ecc-app/src/dev.rs` | exit 0 |
| PC-024 | unit | dev_switch(Dev) creates symlinks | AC-005.2 | `cargo test -p ecc-app dev::tests::dev_switch_dev_creates_symlinks` | pass |
| PC-025 | unit | dev_switch(Default) restores copies | AC-005.3 | `cargo test -p ecc-app dev::tests::dev_switch_default_restores_copies` | pass |
| PC-026 | unit | dev_switch dry_run prints without executing | AC-005.4 | `cargo test -p ecc-app dev::tests::dev_switch_dry_run` | pass |
| PC-027 | unit | dev_switch rollback on error | AC-005.5 | `cargo test -p ecc-app dev::tests::dev_switch_rollback_on_error` | pass |
| PC-028 | unit | dev_switch validates targets within ECC_ROOT | AC-005.6 | `cargo test -p ecc-app dev::tests::dev_switch_validates_targets_within_ecc_root` | pass |
| PC-029 | unit | dev_switch tested with InMemoryFileSystem | AC-005.7 | `cargo test -p ecc-app dev::tests::dev_switch` | pass |
| PC-030 | lint | dev_switch uses MANAGED_DIRS | AC-005.8 | `grep -q 'MANAGED_DIRS' crates/ecc-app/src/dev.rs` | exit 0 |
| PC-031 | unit | dev_switch(Dev) target dirs must exist | AC-005.9 | `cargo test -p ecc-app dev::tests::dev_switch_dev_target_must_exist` | pass |
| PC-032 | unit | dev_off removes symlinks safely | AC-005.10 | `cargo test -p ecc-app dev::tests::dev_off_removes_symlinks_safely` | pass |
| PC-033 | unit | Symlinks use absolute paths | AC-005.11 | `cargo test -p ecc-app dev::tests::dev_switch_uses_absolute_paths` | pass |
| PC-034 | unit | switch dev preserves manifest | AC-005.12 | `cargo test -p ecc-app dev::tests::dev_switch_manifest_preservation` | pass |
| PC-035 | unit | Handles dangling symlinks | AC-005.13 | `cargo test -p ecc-app dev::tests::dev_switch_handles_dangling_symlinks` | pass |
| PC-036 | unit | Removes existing dirs before symlink | AC-005.14 | `cargo test -p ecc-app dev::tests::dev_switch_removes_existing_dirs` | pass |
| PC-037 | unit | dev_status reports Dev (symlinked) | AC-006.1, AC-006.2 | `cargo test -p ecc-app dev::tests::dev_status_symlinked_profile` | pass |
| PC-038 | unit | dev_status reports Default (copied) | AC-006.1 | `cargo test -p ecc-app dev::tests::dev_status_copied_profile` | pass |
| PC-039 | unit | format_status includes Profile line | AC-006.3 | `cargo test -p ecc-app dev::tests::format_status_includes_profile` | pass |
| PC-040 | unit | dev_status returns Inactive | AC-006.4 | `cargo test -p ecc-app dev::tests::dev_status_inactive_no_errors` | pass |
| PC-041 | unit | All three profile states tested | AC-006.5 | `cargo test -p ecc-app dev::tests::dev_status` | pass |
| PC-042 | unit | Mixed state reports Mixed (inconsistent) | AC-006.6 | `cargo test -p ecc-app dev::tests::dev_status_mixed_state` | pass |
| PC-043 | build | DevAction::Switch variant compiles | AC-007.1, AC-007.2, AC-007.3 | `cargo build -p ecc-cli` | exit 0 |
| PC-044 | lint | DevAction::Switch calls dev_switch | AC-007.4 | `grep -q 'DevAction::Switch' crates/ecc-cli/src/commands/dev.rs && grep -q 'dev_switch' crates/ecc-cli/src/commands/dev.rs` | exit 0 |
| PC-045 | build | Existing On/Off/Status unchanged | AC-007.5 | `cargo test -p ecc-cli` | pass |
| PC-046 | build | CLI builds with Switch variant | AC-007.6 | `cargo build -p ecc-cli` | exit 0 |
| PC-047 | unit | dev_switch error propagates | AC-007.7 | `cargo test -p ecc-app dev::tests::dev_switch_error_returns_failure` | pass |
| PC-048 | build | All tests pass | AC-008.1 | `cargo test` | pass |
| PC-049 | lint | Zero clippy warnings | AC-008.2 | `cargo clippy -- -D warnings` | exit 0 |
| PC-050 | lint | Domain zero I/O | AC-008.3 | `! grep -rn 'use std::fs\|use std::process\|use std::net' crates/ecc-domain/src/` | exit 0 |
| PC-051 | build | Full workspace builds | AC-008.1 | `cargo build` | exit 0 |

### Coverage Check
All 55 ACs covered by 51 PCs. No uncovered ACs.

### Design Clarifications (from adversarial review)

**dev_off interaction with clean_from_manifest**: When `dev_off` is called while directories are symlinked, it MUST short-circuit `clean_from_manifest` for managed directories. Instead, `dev_off` calls `remove_file` on each symlinked directory (removing the symlink itself), then proceeds with `clean_from_manifest` only for non-symlinked artifacts (e.g., settings.json entries). This avoids modifying `clean.rs` — the guard lives in `dev.rs` before the clean call.

**Rollback expected state (PC-027)**: After rollback, symlinks created before the error are removed. Directories that were removed before the error are NOT restored (filesystem removal is irreversible without backup). The test verifies: (a) no symlinks remain for successfully-created entries, (b) a log message records which directories could not be restored.

**Fragile grep PCs (PC-010–013, PC-015)**: These are implementation-hint checks that verify specific Rust API usage, not behavioral tests. They complement the build PCs. If implementation uses different import styles, the build PCs (PC-008, PC-009, PC-051) remain the true gates.

**FsError::Unsupported variant**: Covered by PC-009 (all implementors compile) and PC-014 (grep for cfg(not(unix))). The variant must exist for `#[cfg(not(unix))]` branches to compile.

### E2E Test Plan
No E2E tests needed — pure CLI dispatch, all logic tested with InMemoryFileSystem doubles.

### E2E Activation Rules
No E2E tests to activate.

## Test Strategy
TDD order:
1. **Phase 1 (PC-001–007)**: Domain types — DevProfile, SymlinkPlan, build_symlink_plan, MANAGED_DIRS
2. **Phase 2-4 (PC-008–022)**: Port + Infra + Test Support — committed together (trait extension breaks all implementors)
3. **Phase 5 (PC-037–042)**: Enhanced dev_status with profile detection
4. **Phase 6 (PC-023–036, PC-047)**: dev_switch use case — core business logic
5. **Phase 7 (PC-043–046)**: CLI wiring — DevAction::Switch with newtype wrapper for clap::ValueEnum
6. **Phase 8 (PC-048–051)**: Quality gate — cargo test, clippy, domain I/O check, CLAUDE.md

## Doc Update Plan
| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/domain/glossary.md` | Domain | Add entries | DevProfile, SymlinkPlan definitions | US-008, AC-008.1 |
| 2 | `CHANGELOG.md` | Project | Add entry | BL-058 feature: symlink switching, FileSystem extension, ADR 0016 | US-008, AC-008.2 |
| 3 | `docs/adr/0016-directory-level-symlinks.md` | Architecture | Create | Directory-level symlinks vs per-file, hooks exclusion rationale | Decision #1 |
| 4 | `CLAUDE.md` | Reference | Update | CLI Commands table with `ecc dev switch`, test count | AC-008.4 |

## SOLID Assessment
CONDITIONAL PASS — 1 HIGH fixed (clap removed from domain, newtype in CLI). 2 MEDIUM noted (ISP on FileSystem trait — defer; SRP on dev.rs — extract dev_switch logic to keep functions small).

## Robert's Oath Check
CLEAN. No oath violations. Data loss risk (dev_off symlink traversal) explicitly mitigated with is_symlink guard. 51 PCs provide thorough proof. 8 TDD phases ensure small releases. Best-effort rollback limitation honestly documented.

## Security Notes
2 HIGH addressed in design: (1) path canonicalization for ECC_ROOT before operations, (2) clean.rs symlink guard using remove_file instead of remove_dir_all. 2 MEDIUM noted: TOCTOU race (low practical risk), WalkDir follow_links (consider .follow_links(false)).

## Rollback Plan
Reverse dependency order:
1. Revert `CLAUDE.md` changes
2. Revert `CHANGELOG.md` changes
3. Revert `docs/domain/glossary.md` changes
4. Delete `docs/adr/0016-directory-level-symlinks.md`
5. Revert `crates/ecc-cli/src/commands/dev.rs`
6. Revert `crates/ecc-app/src/dev.rs`
7. Revert `crates/ecc-infra/src/os_fs.rs`
8. Revert `crates/ecc-test-support/src/in_memory_fs.rs`
9. Revert `crates/ecc-ports/src/fs.rs`
10. Revert `crates/ecc-domain/src/config/mod.rs`
11. Delete `crates/ecc-domain/src/config/dev_profile.rs`
