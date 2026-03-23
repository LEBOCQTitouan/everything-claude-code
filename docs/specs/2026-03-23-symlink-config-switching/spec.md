# Spec: Symlink-Based Instant Config Switching (BL-058)

## Problem Statement

The current `ecc dev on` command performs a full clean + reinstall cycle to switch between release and development configurations. This is slow and writes many files. Developers iterating on ECC agents, commands, skills, and rules need instant switching. Unix symlinks provide zero-copy, zero-file-edit instant switching by pointing `~/.claude/` asset directories to the local ECC checkout (`ECC_ROOT`).

## Research Summary

- **GNU Stow pattern**: Industry-standard symlink farm manager. Symlinks entire directories when single ownership applies (our case). `--restow` pattern (unstow + stow) for clean transitions.
- **Rust `std::os::unix::fs::symlink`**: Standard Unix symlink creation. `std::fs::read_link` for reading targets. `std::fs::symlink_metadata` for detection without following.
- **Directory-level symlinks preferred**: When a single tool owns all files in a directory (ECC owns agents/, commands/, etc.), symlinking at directory level is simpler and more efficient than per-file symlinks.
- **Best-effort rollback**: True atomic rollback impossible with filesystem symlinks. Two-phase approach: build plan first, execute with tracked state for reversal on error.
- **Platform isolation**: Port trait stays unconditional; `OsFileSystem` uses `#[cfg(unix)]` with fallback error on non-Unix.
- **Symlink target validation**: Validate targets are within ECC_ROOT to prevent symlink-following attacks.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Directory-level symlinks (agents/, commands/, skills/, rules/) not per-file | Single ownership (ECC owns all files). Simpler, faster, matches GNU Stow pattern. | Yes |
| 2 | Exclude hooks/ directory from symlink plan | Hooks are registered in settings.json, not a filesystem directory. Symlinking hooks/ would be meaningless. | No |
| 3 | Runtime profile detection via is_symlink, no manifest schema change | Simpler, no backward-compatibility concern, detection is instant. | No |
| 4 | `switch default` = remove symlinks + reinstall copies (reuse dev_on logic) | Consistent with existing behavior. User gets working config in either state. | No |
| 5 | Best-effort rollback on error, not atomic | True FS atomicity impossible. Track operations and reverse on failure. Document limitation. | No |
| 6 | Unix-only v1, `FsError::Unsupported` on non-Unix | Windows junction points are complex. Scoping to Unix keeps v1 simple. | No |
| 7 | Symlink target validation — targets must be within ECC_ROOT | Prevents symlink-following attacks where targets point outside the ECC checkout. | No |
| 8 | Post-install symlink swap (Option A from architect) not modify install pipeline | Keeps install pipeline unchanged. Lower risk, simpler implementation. | No |
| 9 | Manifest preserved during `switch dev`, regenerated on `switch default` | Manifest records installed file state. During dev mode, manifest is stale but harmless. `switch default` runs `dev_on` which reinstalls and regenerates. Avoids manifest schema changes. | No |

## User Stories

### US-001: Domain — DevProfile enum and SymlinkPlan value type

**As a** developer working on ECC config switching, **I want** pure domain types representing the dev profile and symlink operations, **so that** business logic for profile switching is testable without I/O.

#### Acceptance Criteria

- AC-001.1: Given `ecc-domain`, when `DevProfile` enum is defined, then it has variants `Default` and `Dev` with `Debug, Clone, PartialEq, Eq` derives
- AC-001.2: Given `ecc-domain`, when `SymlinkPlan` is defined, then it contains a `Vec<SymlinkOp>` where each `SymlinkOp` holds `target: PathBuf` and `link: PathBuf`
- AC-001.3: Given `DevProfile`, when parsed from CLI, then it implements `clap::ValueEnum`
- AC-001.4: Given a function `build_symlink_plan(ecc_root, claude_dir, managed_dirs)`, when called, then it returns a `SymlinkPlan` computed without I/O
- AC-001.5: Given `ecc-domain` crate, when checked for I/O imports, then zero `std::fs`, `std::process`, `std::net` imports exist
- AC-001.6: Given unit tests for plan building, when run, then both `Dev` and `Default` profiles produce correct plans
- AC-001.7: Given the managed directories list, when defined, then it is a domain constant: `MANAGED_DIRS = ["agents", "commands", "skills", "rules"]`

#### Dependencies

- Depends on: none

### US-002: Port — Symlink operations on FileSystem trait

**As a** developer extending the filesystem abstraction, **I want** `create_symlink`, `read_symlink`, and `is_symlink` operations on the `FileSystem` trait, **so that** symlink operations are abstracted behind the port boundary.

#### Acceptance Criteria

- AC-002.1: Given `FileSystem` trait, when extended, then `create_symlink(target: &Path, link: &Path) -> Result<(), FsError>` exists
- AC-002.2: Given `FileSystem` trait, when extended, then `read_symlink(link: &Path) -> Result<PathBuf, FsError>` exists
- AC-002.3: Given `FileSystem` trait, when extended, then `is_symlink(path: &Path) -> bool` exists
- AC-002.4: Given all `FileSystem` implementors, when trait is extended, then all compile successfully

#### Dependencies

- Depends on: none

### US-003: Infra — OsFileSystem symlink implementation

**As a** developer running ECC on Unix, **I want** `OsFileSystem` to implement symlink operations using `std::os::unix::fs::symlink`, **so that** real symlinks are created on disk.

#### Acceptance Criteria

- AC-003.1: Given `OsFileSystem::create_symlink`, when called on Unix, then it calls `std::os::unix::fs::symlink(target, link)` with proper error mapping
- AC-003.2: Given `OsFileSystem::read_symlink`, when called, then it calls `std::fs::read_link(link)` with proper error mapping
- AC-003.3: Given `OsFileSystem::is_symlink`, when called, then it uses `std::fs::symlink_metadata` and checks `file_type().is_symlink()`
- AC-003.4: Given `create_symlink` with existing file or symlink at link path, when called, then it removes the existing file/symlink before creating. Callers are responsible for removing existing directories before calling `create_symlink`.
- AC-003.5: Given non-Unix platform, when symlink methods are called, then `FsError::Unsupported` is returned (via `#[cfg(unix)]` / `#[cfg(not(unix))]`)

#### Dependencies

- Depends on: US-002

### US-004: Test Support — InMemoryFileSystem symlink stub

**As a** test author, **I want** `InMemoryFileSystem` to simulate symlinks via an internal map, **so that** I can write unit tests for symlink-based switching without touching the real filesystem.

#### Acceptance Criteria

- AC-004.1: Given `InMemoryFileSystem`, when constructed, then a `symlinks: Arc<Mutex<BTreeMap<PathBuf, PathBuf>>>` field tracks link-to-target mappings
- AC-004.2: Given `create_symlink(target, link)`, when called, then it inserts into symlinks map and removes from files map if present
- AC-004.3: Given `read_symlink(link)`, when called on a symlink, then it returns the target; on a non-symlink, returns `Err(NotFound)`
- AC-004.4: Given `is_symlink(path)`, when called, then returns `true` if path is in symlinks map
- AC-004.5: Given `exists(path)`, when called on a symlink, then returns `true`
- AC-004.6: Given `remove_file(path)`, when called on a symlink, then removes from symlinks map
- AC-004.7: Given builder pattern, when `with_symlink(link, target)` is called, then symlink is set up for testing
- AC-004.8: Given existing `InMemoryFileSystem` tests, when run after changes, then all pass

#### Dependencies

- Depends on: US-002

### US-005: App — `dev_switch` use case

**As a** developer switching between release and local config, **I want** a `dev_switch(profile, ecc_root, claude_dir, dry_run)` use case, **so that** `ecc dev switch dev` creates symlinks and `ecc dev switch default` restores copies.

#### Acceptance Criteria

- AC-005.1: Given `dev_switch` function, when it exists, then it lives in `ecc-app/src/dev.rs` or a new `dev_switch.rs` module
- AC-005.2: Given `Dev` profile, when `dev_switch` runs, then it removes existing files/dirs for each managed category and creates symlinks from `claude_dir/<category>` to `ecc_root/<category>`
- AC-005.3: Given `Default` profile, when `dev_switch` runs, then it removes symlinks and runs `install_global` to restore copied files
- AC-005.4: Given `dry_run=true`, when `dev_switch` runs, then it prints the planned operations without executing them
- AC-005.5: Given error mid-execution, when `dev_switch` fails, then best-effort rollback restores any already-processed items. The rollback tracker is a `Vec<CompletedOp>` recording each operation performed.
- AC-005.6: Given symlink targets, when `dev_switch` validates them, then it confirms all targets are within ECC_ROOT (security validation)
- AC-005.7: Given unit tests, when run, then all logic is tested with `InMemoryFileSystem`
- AC-005.8: Given managed paths, when `dev_switch` discovers them, then it uses the domain constant `MANAGED_DIRS` for the directory list and optionally reads the manifest to verify installation state (missing manifest = not installed, error)
- AC-005.9: Given `dev_switch(Dev)`, when validating, then each symlink target directory must exist within ECC_ROOT on disk (not just "within" — actually present)
- AC-005.10: Given `dev off` is called while in symlinked state, when it runs, then it detects symlinks and removes them (via `remove_file` on the symlink itself) without traversing into the linked directory. This prevents accidental deletion of source files in ECC_ROOT.
- AC-005.11: Given symlink creation, when targets are specified, then absolute paths are used
- AC-005.12: Given `switch dev`, when the manifest exists, then the manifest is preserved as-is (not deleted or modified). `switch default` reuses `dev_on` which reinstalls and regenerates the manifest.
- AC-005.13: Given a dangling symlink at a managed directory path, when `dev_switch` runs, then it removes the dangling symlink before proceeding (handles both valid and dangling symlinks)
- AC-005.14: Given `dev_switch` directory removal, when an existing directory (not symlink) is at the link path, then `dev_switch` calls `remove_dir_all` on it before creating the symlink. The rollback tracker records the removed directory.

#### Dependencies

- Depends on: US-001, US-002, US-004

### US-006: App — Enhanced `dev_status` with profile detection

**As a** user checking my ECC installation, **I want** `ecc dev status` to show whether my config is symlinked or copied, **so that** I can verify which profile is active.

#### Acceptance Criteria

- AC-006.1: Given `DevStatus`, when profile detection runs, then it reports `Dev (symlinked)`, `Default (copied)`, or `Inactive`
- AC-006.2: Given profile detection, when checking, then it uses `is_symlink` on managed directories to determine profile
- AC-006.3: Given `format_status` output, when displayed, then it includes a `Profile:` line
- AC-006.4: Given ECC is not installed, when status runs, then it returns `Inactive` without errors
- AC-006.5: Given unit tests, when run, then all three profile states are covered
- AC-006.6: Given mixed state (some dirs symlinked, some copied), when `dev_status` runs, then it reports `Mixed (inconsistent)` with a warning listing which directories are symlinked and which are copied

#### Dependencies

- Depends on: US-002, US-004

### US-007: CLI — `DevAction::Switch` subcommand

**As a** CLI user, **I want** `ecc dev switch dev|default [--dry-run]` to invoke the switch use case, **so that** I can toggle between symlinked and copied config from the terminal.

#### Acceptance Criteria

- AC-007.1: Given `DevAction` enum, when extended, then `Switch { profile: DevProfile, dry_run: bool }` variant exists
- AC-007.2: Given `profile` argument, when parsed, then `clap::ValueEnum` is used on `DevProfile`
- AC-007.3: Given `--dry-run` flag, when omitted, then defaults to `false`
- AC-007.4: Given `DevAction::Switch`, when matched in `run()`, then it calls `dev::dev_switch`
- AC-007.5: Given existing `On`, `Off`, `Status` actions, when `Switch` is added, then they remain unchanged
- AC-007.6: Given `ecc dev switch dev --dry-run`, when run, then it prints planned operations and exits 0
- AC-007.7: Given `ecc dev switch dev` with errors, when run, then it exits non-zero

#### Dependencies

- Depends on: US-005

### US-008: Quality gate

**As a** maintainer, **I want** all new code to pass `cargo test` and `cargo clippy -- -D warnings`, **so that** the codebase remains clean.

#### Acceptance Criteria

- AC-008.1: Given `cargo test`, when run, then all tests pass including new ones
- AC-008.2: Given `cargo clippy -- -D warnings`, when run, then zero warnings
- AC-008.3: Given `ecc-domain`, when checked, then zero I/O imports
- AC-008.4: Given CLAUDE.md, when checked, then test count is updated

#### Dependencies

- Depends on: US-001, US-002, US-003, US-004, US-005, US-006, US-007

## Affected Modules

| Module | Layer | Nature of Change |
|--------|-------|-----------------|
| `crates/ecc-domain/src/config/dev_profile.rs` | Domain (new) | `DevProfile` enum, `SymlinkPlan` value type, `build_symlink_plan` |
| `crates/ecc-ports/src/fs.rs` | Port | Extend `FileSystem` trait with 3 symlink methods |
| `crates/ecc-infra/src/os_fs.rs` | Infra | Implement symlink methods via `std::os::unix::fs` |
| `crates/ecc-test-support/src/in_memory_fs.rs` | Test Support | Add symlink tracking and follow-through |
| `crates/ecc-app/src/dev.rs` | App | `dev_switch` use case, enhanced `dev_status` |
| `crates/ecc-cli/src/commands/dev.rs` | CLI | `DevAction::Switch` variant |
| `docs/domain/glossary.md` | Docs | DevProfile, SymlinkPlan entries |
| `CHANGELOG.md` | Docs | BL-058 entry |
| `docs/adr/0016-directory-level-symlinks.md` | Docs | ADR for directory-level symlink decision |

## Constraints

- `ecc-domain` must have zero I/O imports
- `cargo test` must pass (all ~1185+ tests)
- `cargo clippy -- -D warnings` must pass
- Existing `on`/`off`/`status` actions must be unchanged
- Unix-only for v1 (`#[cfg(unix)]`)
- Managed directories: `agents/`, `commands/`, `skills/`, `rules/` (NOT `hooks/`)
- Symlink targets must be validated as within ECC_ROOT

## Non-Requirements

- Windows junction points or symlink support
- Modifying `ecc install` to support symlinks
- Automatic profile detection on shell startup
- Changes to existing `on`/`off`/`status` actions
- Hooks directory symlinking (hooks are in settings.json)
- Manifest schema changes (profile detection is runtime-only)
- Logging or doc comment additions beyond what's directly needed

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| FileSystem port | 3 new methods | All existing call sites unaffected. New call sites only in dev_switch. |
| CLI → App | New DevAction variant | Thin dispatch layer, low risk |
| App → Domain | New types consumed | No existing domain types modified |
| Infra → OS | Unix symlink syscalls | Platform-specific, gated by `#[cfg(unix)]` |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New domain concepts | Domain | `docs/domain/glossary.md` | Add DevProfile, SymlinkPlan entries |
| Feature entry | Project | `CHANGELOG.md` | Add BL-058 entry |
| New ADR | Architecture | `docs/adr/0016-directory-level-symlinks.md` | Directory-level symlink decision |
| CLI reference | Reference | `CLAUDE.md` | Update CLI Commands table with `switch` |

## Open Questions

None — all resolved during grill-me interview.
