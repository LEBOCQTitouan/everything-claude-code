# Design: BL-088 — ecc update Dual-Mode Deploy

Spec: [spec.md](./spec.md)

---

## 1. File Changes Table (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/update/platform.rs` | **Create** | Define `Architecture` and `Platform` enums in the domain layer (zero-I/O). Include `current()` compile-time detection, `ArtifactName`-aligned labels. | AC-003.1 |
| 2 | `crates/ecc-domain/src/update/artifact.rs` | **Modify** | Change `ArtifactName::resolve` to accept `Platform` and `Architecture` enums instead of `&str`. Add Windows variant (`ecc-win32-x64`). Remove string-based API. Add `tarball_filename()` and `extension()` helpers. | AC-003.2, AC-003.3, AC-003.4, AC-003.5, AC-008.1 |
| 3 | `crates/ecc-domain/src/update/error.rs` | **Modify** | Update `UnsupportedPlatform` to hold `Platform` + `Architecture` enums instead of `String`. Add `SecurityVerificationFailed` variant. Add `UpdateLocked` variant. Add `PermissionDenied` variant. Add `RollbackFailed` composite variant. | AC-001.8, AC-004.2, AC-005.3, AC-007.2 |
| 4 | `crates/ecc-domain/src/update/mod.rs` | **Modify** | Add `pub mod platform;` and re-export `Architecture`, `Platform`. | AC-003.1 |
| 5 | `crates/ecc-ports/src/env.rs` | **Modify** | Remove `Architecture` and `Platform` enum definitions. Re-export from `ecc_domain::update::platform`. Add `current_exe() -> Option<PathBuf>` to `Environment` trait. | AC-003.1, AC-001.9 |
| 6 | `crates/ecc-ports/src/release.rs` | **Modify** | Update `verify_cosign` signature to accept `bundle_path: &Path` parameter (the `.bundle` file is downloaded separately). Add `download_file` method for downloading checksum files and `.bundle` files. | AC-007.1, AC-007.5 |
| 7 | `crates/ecc-app/src/update/extract.rs` | **Create** | Tarball extraction module: `extract_tarball(fs, tarball_path, dest_dir) -> Result<(), UpdateError>`. Delegates to a port trait `TarballExtractor` for the actual `flate2+tar` work. Validates zip-slip prevention in domain logic. | AC-002.1, AC-002.3, AC-002.4 |
| 8 | `crates/ecc-app/src/update/swap.rs` | **Modify** | Add `rollback_swapped(fs, swapped: &[(PathBuf, PathBuf)]) -> Result<(), UpdateError>` function. Add Windows-specific swap path (`rename_to_old` before placing new). Add `.bak` cleanup function `cleanup_backups`. | AC-005.1, AC-005.2, AC-005.3, AC-005.4, AC-008.2 |
| 9 | `crates/ecc-app/src/update/orchestrator.rs` | **Modify** | (a) Replace `std::env::current_exe()` with `ctx.env.current_exe()`. (b) Add permission check before download. (c) Wire extraction via port. (d) Add flock acquisition around update body. (e) Wire rollback on `ConfigSyncFailed`. (f) Make cosign `NotInstalled` abort (not warning). (g) Accept `Platform`/`Architecture` enums directly from `ArtifactName::resolve`. | AC-001.8, AC-001.9, AC-002.2, AC-004.1-4.4, AC-005.1-5.5, AC-007.6 |
| 10 | `crates/ecc-app/src/update/context.rs` | **Modify** | Add `lock: &'a dyn FileLock` field. Add `extractor: &'a dyn TarballExtractor` field. | AC-004.1, AC-002.1 |
| 11 | `crates/ecc-app/src/update/mod.rs` | **Modify** | Add `pub mod extract;` | -- |
| 12 | `crates/ecc-ports/src/extract.rs` | **Create** | Define `TarballExtractor` port trait: `extract(tarball: &Path, dest: &Path) -> Result<(), ExtractError>`. | AC-002.1 |
| 13 | `crates/ecc-ports/src/lib.rs` | **Modify** | Add `pub mod extract;` | -- |
| 14 | `crates/ecc-infra/src/github_release.rs` | **Modify** | Replace stub with real `ureq`-based implementation. Implement `latest_version` (GET `/repos/{owner}/{repo}/releases/latest`), `get_version` (GET `/repos/{owner}/{repo}/releases/tags/v{version}`), `download_tarball` (streaming GET to file), `verify_checksum` (download `checksums-sha256.txt`, compute SHA256, compare), `verify_cosign` (download `.bundle`, shell out to `cosign verify-blob`). | AC-001.1-001.7, AC-007.1, AC-007.4 |
| 15 | `crates/ecc-infra/src/tarball_extractor.rs` | **Create** | `FlateExtractor` implementing `TarballExtractor` using `flate2::read::GzDecoder` + `tar::Archive`. Zip-slip prevention via `canonicalize` + prefix check. Windows `.exe` extension preservation. | AC-002.1, AC-002.3, AC-002.4, AC-008.3 |
| 16 | `crates/ecc-infra/src/os_env.rs` | **Modify** | Implement `current_exe()` via `std::env::current_exe().ok()`. Update import paths for domain-based `Architecture`/`Platform`. | AC-001.9 |
| 17 | `crates/ecc-infra/src/lib.rs` | **Modify** | Add `pub mod tarball_extractor;` | -- |
| 18 | `crates/ecc-infra/Cargo.toml` | **Modify** | Add `ureq = "3"`, `flate2 = "1"`, `tar = "0.4"`, `sha2 = "0.10"` dependencies. | -- |
| 19 | `crates/ecc-test-support/src/mock_env.rs` | **Modify** | Add `current_exe` field and `with_current_exe` builder. Update to import `Architecture`/`Platform` from `ecc_domain` (via `ecc_ports` re-export). | AC-001.9 |
| 19b | `crates/ecc-test-support/src/mock_release_client.rs` | **Modify** | Add `download_file` implementation and update `verify_cosign` mock to accept `bundle_path: &Path`. Sequence in Phase 2 alongside port changes. | AC-007.1, AC-007.5 |
| 20 | `crates/ecc-test-support/src/mock_extractor.rs` | **Create** | `MockExtractor` implementing `TarballExtractor` for tests. Simulates extraction by writing expected files to dest dir. | AC-002.1 |
| 21 | `crates/ecc-test-support/src/lib.rs` | **Modify** | Add `pub mod mock_extractor;` and re-export. | -- |
| 22 | `crates/ecc-cli/src/commands/update.rs` | **Modify** | Inject `FlateExtractor` and `FlockLock` into `UpdateContext`. | AC-004.1, AC-002.1 |
| 23 | `xtask/src/deploy.rs` | **Modify** | Add `ecc-flock` to build packages list (`-p ecc-flock`), install list, and dry-run summary. | AC-006.1, AC-006.2, AC-006.3 |
| 24 | `.github/workflows/release.yml` | **Modify** | Change cosign signing from `--output-signature "${asset}.sig" --output-certificate "${asset}.bundle"` to `--bundle "${asset}.bundle"` (single Sigstore bundle file). | AC-007.5 |
| 25 | `Cargo.toml` (workspace) | **Modify** | Add `ureq`, `flate2`, `tar`, `sha2` to `[workspace.dependencies]`. | -- |

---

## 2. Pass Conditions Table

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | Unit | `Platform` and `Architecture` enums defined in `ecc-domain`, with `current()` and display | AC-003.1 | `cargo test -p ecc-domain platform` | PASS |
| PC-002 | Unit | `ArtifactName::resolve(Platform::MacOS, Architecture::Arm64)` returns `ecc-darwin-arm64` | AC-003.2 | `cargo test -p ecc-domain resolves_macos_arm64` | PASS |
| PC-003 | Unit | All 5 release targets resolve correctly via `ArtifactName::resolve` | AC-003.3, AC-008.1 | `cargo test -p ecc-domain artifact` | PASS |
| PC-004 | Unit | `ArtifactName::resolve(Platform::Unknown, Architecture::Amd64)` returns `UnsupportedPlatform` | AC-003.4 | `cargo test -p ecc-domain rejects_unsupported` | PASS |
| PC-005 | Unit | `ArtifactName::resolve` signature accepts enums, string-based overload removed | AC-003.5 | `cargo test -p ecc-domain artifact` | PASS |
| PC-006 | Unit | `UpdateError::UnsupportedPlatform` holds enum variants, `PermissionDenied`/`UpdateLocked`/`SecurityVerificationFailed`/`RollbackFailed` variants exist | AC-001.8, AC-004.2, AC-005.3, AC-007.2 | `cargo test -p ecc-domain error_display` | PASS |
| PC-007 | Unit | `ecc-ports` re-exports `Architecture` and `Platform` from domain | AC-003.1 | `cargo test -p ecc-ports` | PASS |
| PC-008 | Unit | `Environment` trait includes `current_exe() -> Option<PathBuf>` | AC-001.9 | `cargo build -p ecc-ports` | exit 0 |
| PC-009 | Unit | `MockEnvironment` supports `with_current_exe` builder and returns it from `current_exe()` | AC-001.9 | `cargo test -p ecc-test-support mock_env` | PASS |
| PC-010 | Unit | `TarballExtractor` port trait compiles, `MockExtractor` implements it | AC-002.1 | `cargo test -p ecc-test-support mock_extractor` | PASS |
| PC-011 | Unit | `rollback_swapped` restores all binaries from `.bak` backups | AC-005.1 | `cargo test -p ecc-app rollback_swapped_restores` | PASS |
| PC-012 | Unit | `rollback_swapped` failure returns both original and rollback errors | AC-005.3 | `cargo test -p ecc-app rollback_both_failures` | PASS |
| PC-013 | Unit | `cleanup_backups` removes `.bak` files after successful update | AC-005.4 | `cargo test -p ecc-app cleanup_backups` | PASS |
| PC-014 | Unit | Windows swap: renames running binary to `.old` before placing new | AC-008.2 | `cargo test -p ecc-app windows_swap` | PASS |
| PC-015 | Unit | Orchestrator uses `ctx.env.current_exe()` instead of `std::env::current_exe()` | AC-001.9 | `cargo test -p ecc-app full_upgrade_flow` | PASS |
| PC-016 | Unit | Orchestrator checks install dir writability before download, returns `PermissionDenied` | AC-001.8 | `cargo test -p ecc-app permission_denied_before_download` | PASS |
| PC-017 | Unit | Orchestrator acquires flock, returns `UpdateLocked` when lock unavailable | AC-004.1, AC-004.2 | `cargo test -p ecc-app update_locked` | PASS |
| PC-018 | Unit | Lock released on success and on failure (RAII guard drop) | AC-004.3 | `cargo test -p ecc-app lock_released` | PASS |
| PC-019 | Unit | Orchestrator wires extraction from tarball to temp dir | AC-002.1, AC-002.2 | `cargo test -p ecc-app full_upgrade_flow` | PASS |
| PC-020 | Unit | Corrupt archive returns `SwapFailed` with no partial state | AC-002.3 | `cargo test -p ecc-app corrupt_archive` | PASS |
| PC-021 | Unit | Zip-slip path traversal rejected during extraction | AC-002.4 | `cargo test -p ecc-infra zip_slip_prevention` | PASS |
| PC-022 | Unit | Orchestrator on `ConfigSyncFailed` invokes `rollback_swapped`, message contains "rolled back" | AC-005.1, AC-005.2, AC-005.5 | `cargo test -p ecc-app config_sync_triggers_rollback` | PASS |
| PC-023 | Unit | Cosign `NotInstalled` aborts update (not treated as warning) | AC-007.6 | `cargo test -p ecc-app cosign_not_installed_aborts` | PASS |
| PC-024 | Unit | Cosign `Failed` aborts with `SecurityVerificationFailed` | AC-007.2 | `cargo test -p ecc-app cosign_failed_aborts` | PASS |
| PC-025 | Unit | `GithubReleaseClient::latest_version` parses GitHub API JSON, returns `ReleaseInfo` | AC-001.1 | `cargo test -p ecc-infra parse_latest_release` | PASS |
| PC-026 | Unit | `GithubReleaseClient::get_version("4.3.0")` queries correct tag URL | AC-001.2 | `cargo test -p ecc-infra get_specific_version` | PASS |
| PC-027 | Unit | Download streams to disk, invokes progress callbacks (no full-tarball buffer) | AC-001.3 | `cargo test -p ecc-infra streaming_download` | PASS |
| PC-028 | Unit | Checksum verification: computes SHA256, compares against `checksums-sha256.txt` | AC-001.4 | `cargo test -p ecc-infra checksum_verification` | PASS |
| PC-029 | Unit | Network error returns clear `NetworkError` with retry guidance | AC-001.5 | `cargo test -p ecc-infra network_error` | PASS |
| PC-030 | Unit | HTTP 403 returns `RateLimited` with reset time | AC-001.6 | `cargo test -p ecc-infra rate_limited` | PASS |
| PC-031 | Unit | `GITHUB_TOKEN` included as Authorization header when present | AC-001.7 | `cargo test -p ecc-infra github_token_auth` | PASS |
| PC-032 | Unit | `FlateExtractor` extracts `bin/ecc` and `bin/ecc-workflow` from valid tarball | AC-002.1 | `cargo test -p ecc-infra extract_valid_tarball` | PASS |
| PC-033 | Unit | Windows extraction preserves `.exe` extension | AC-008.3 | `cargo test -p ecc-infra windows_exe_preserved` | PASS |
| PC-034 | Unit | Cosign verification downloads `.bundle`, runs `cosign verify-blob --bundle` with certificate identity + OIDC issuer | AC-007.1, AC-007.3, AC-007.4 | `cargo test -p ecc-infra cosign_verify_bundle` | PASS |
| PC-035 | Unit | Certificate identity is compile-time constant matching workflow path | AC-007.4 | `cargo test -p ecc-infra certificate_identity_constant` | PASS |
| PC-036 | Unit | `OsEnvironment::current_exe()` returns path | AC-001.9 | `cargo test -p ecc-infra os_env_current_exe` | PASS |
| PC-037 | Unit | xtask deploy builds ecc-cli, ecc-workflow, ecc-flock | AC-006.1 | `cargo test -p xtask deploy_builds_three` | PASS |
| PC-038 | Unit | xtask deploy installs all three to `~/.cargo/bin/` | AC-006.2 | `cargo test -p xtask deploy_installs_three` | PASS |
| PC-039 | Unit | xtask `--dry-run` lists all three binaries | AC-006.3 | `cargo test -p xtask deploy_dry_run_lists_three` | PASS |
| PC-040 | Integration | `ecc update --dry-run` prints plan without errors (no network, mocked) | AC-001.1 | `cargo test -p ecc-app dry_run_no_writes` | PASS |
| PC-041 | Integration | Full orchestrator flow with mock adapters: download, extract, swap, install, verify | AC-001.1-001.9, AC-002.1-002.2 | `cargo test -p ecc-app full_upgrade_flow` | PASS |
| PC-042 | CI | release.yml uses `cosign sign-blob --bundle` format | AC-007.5 | `grep -q 'cosign sign-blob.*--bundle' .github/workflows/release.yml` | exit 0 |
| PC-043 | Lint | Clippy passes with zero warnings | -- | `cargo clippy -- -D warnings` | exit 0 |
| PC-044 | Build | Release build succeeds | -- | `cargo build --release` | exit 0 |
| PC-045 | Build | `ReleaseClient` trait compiles with `download_file` + updated `verify_cosign`; `MockReleaseClient` implements both | AC-007.1, AC-007.5 | `cargo build -p ecc-ports && cargo build -p ecc-test-support` | exit 0 |

---

## 3. TDD Order

### Phase 1: Domain Enums + Artifact Resolution (Layers: Entity)

**PCs: 001-006**

1. Create `crates/ecc-domain/src/update/platform.rs` with `Architecture` and `Platform` enums (moved from `ecc-ports/src/env.rs`).
2. Modify `ArtifactName::resolve` to accept enum parameters. Add Windows variant.
3. Update `UpdateError::UnsupportedPlatform` to hold enum variants. Add new error variants.
4. Update existing tests in `artifact.rs` and `error.rs`.

**Why first**: All other layers depend on these domain types. Zero-I/O, no dependencies to satisfy.

### Phase 2: Port Trait Changes (Layers: Entity, Adapter)

**PCs: 007-010**

1. Modify `ecc-ports/src/env.rs`: remove enum definitions, re-export from domain. Add `current_exe()` to `Environment` trait.
2. Create `ecc-ports/src/extract.rs` with `TarballExtractor` port trait.
3. Update `MockEnvironment` with `current_exe` support.
4. Create `MockExtractor` in `ecc-test-support`.
5. Update all callers that previously imported `Architecture`/`Platform` from ports (imports remain valid via re-export).

**Why second**: Port traits must be defined before app layer can reference them.

### Phase 3: Swap + Rollback (Layers: UseCase)

**PCs: 011-014**

1. Add `rollback_swapped` function to `swap.rs`.
2. Add `cleanup_backups` function.
3. Add Windows swap path (rename-to-`.old` pattern).
4. Write unit tests using `InMemoryFileSystem`.

**Why third**: Rollback is a self-contained app-layer concern that the orchestrator will consume in Phase 4.

### Phase 4: Orchestrator Wiring (Layers: UseCase)

**PCs: 015-024**

1. Replace `std::env::current_exe()` with `ctx.env.current_exe()`.
2. Add permission check before download.
3. Wire flock acquisition (add `lock` to `UpdateContext`).
4. Wire extraction via `TarballExtractor` port.
5. Wire rollback on `ConfigSyncFailed`.
6. Make cosign `NotInstalled` abort.
7. Update all orchestrator tests.

**Why fourth**: Depends on Phases 1-3 for enums, port traits, and rollback function.

### Phase 5: Infrastructure — GithubReleaseClient (Layers: Adapter, Framework)

**PCs: 025-036**

1. Add `ureq`, `flate2`, `tar`, `sha2` to workspace dependencies.
2. Implement `GithubReleaseClient` with real `ureq` HTTP calls.
3. Implement `FlateExtractor` with `flate2`+`tar`.
4. Implement cosign verification via `cosign verify-blob --bundle`.
5. Update `OsEnvironment::current_exe()`.
6. Unit tests use mock HTTP responses (not real network).

**Why fifth**: Infrastructure is the outermost layer; depends on all port trait shapes being finalized.

### Phase 6: CLI + xtask + CI (Layers: Framework)

**PCs: 037-042**

1. Update `ecc-cli/src/commands/update.rs` to inject new dependencies.
2. Update `xtask/src/deploy.rs` with `ecc-flock`.
3. Update `.github/workflows/release.yml` cosign signing.

**Why last**: Thin wiring layer that depends on everything above.

### Phase 7: Final Gates

**PCs: 043-044**

1. `cargo clippy -- -D warnings`
2. `cargo build --release`

---

## 4. Key Design Decisions

### 4.1 Architecture/Platform Enum Migration

**Current state**: `Architecture` and `Platform` are defined in `crates/ecc-ports/src/env.rs` (lines 20-78). The `Environment` trait (same file, line 4) returns them. `ArtifactName::resolve` in `crates/ecc-domain/src/update/artifact.rs` (line 17) accepts `&str` parameters, creating a string mismatch where `Architecture::Arm64.as_label()` returns `"aarch64"` but the match arm expects `"arm64"`.

**New state**: Create `crates/ecc-domain/src/update/platform.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture { Amd64, Arm64, Unknown }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform { MacOS, Linux, Windows, Unknown }
```

In `crates/ecc-ports/src/env.rs`, replace the enum bodies with:

```rust
pub use ecc_domain::update::platform::{Architecture, Platform};
```

This preserves all existing `use ecc_ports::env::{Architecture, Platform}` imports across the codebase (no breaking change to external callers). The `as_label()` method is removed; `ArtifactName::resolve` handles the mapping internally.

`ArtifactName::resolve` becomes:

```rust
pub fn resolve(platform: Platform, arch: Architecture) -> Result<Self, UpdateError> {
    let name = match (platform, arch) {
        (Platform::MacOS, Architecture::Arm64) => "ecc-darwin-arm64",
        (Platform::MacOS, Architecture::Amd64) => "ecc-darwin-x64",
        (Platform::Linux, Architecture::Amd64) => "ecc-linux-x64",
        (Platform::Linux, Architecture::Arm64) => "ecc-linux-arm64",
        (Platform::Windows, Architecture::Amd64) => "ecc-win32-x64",
        _ => return Err(UpdateError::UnsupportedPlatform { platform, arch }),
    };
    Ok(Self(name.to_string()))
}
```

### 4.2 ureq Integration (Streaming Pattern)

`GithubReleaseClient` uses `ureq` v3 (sync, no tokio). Key patterns:

**API calls** (JSON):
```rust
fn latest_version(&self, include_prerelease: bool) -> Result<ReleaseInfo, BoxError> {
    let url = format!("https://api.github.com/repos/{}/{}/releases/latest",
                       self.repo_owner, self.repo_name);
    let mut request = ureq::get(&url).set("Accept", "application/vnd.github+json");
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        request = request.set("Authorization", &format!("Bearer {token}"));
    }
    let response = request.call()?;
    // Parse JSON, extract tag_name, body
}
```

**Streaming download** (AC-001.3): Read in 8KB chunks to a `BufWriter<File>`, calling `on_progress` after each chunk. Never buffer the entire tarball:

```rust
fn download_tarball(&self, ..., on_progress: &dyn Fn(u64, u64)) -> Result<(), BoxError> {
    let response = ureq::get(&url).call()?;
    let total = response.header("Content-Length").and_then(|h| h.parse().ok()).unwrap_or(0);
    let mut reader = response.into_reader();
    let mut writer = BufWriter::new(File::create(dest)?);
    let mut downloaded: u64 = 0;
    let mut buf = [0u8; 8192];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 { break; }
        writer.write_all(&buf[..n])?;
        downloaded += n as u64;
        on_progress(downloaded, total);
    }
    Ok(())
}
```

**Error mapping**: HTTP 403 with `X-RateLimit-Reset` header maps to `UpdateError::RateLimited`. Connection failures map to `UpdateError::NetworkError`. HTTP 404 maps to `UpdateError::VersionNotFound`.

### 4.3 Tarball Extraction (flate2 + tar, zip-slip prevention)

Create `crates/ecc-infra/src/tarball_extractor.rs` implementing the `TarballExtractor` port trait.

**Port trait** (`crates/ecc-ports/src/extract.rs`):
```rust
pub trait TarballExtractor: Send + Sync {
    fn extract(&self, tarball: &Path, dest: &Path) -> Result<Vec<PathBuf>, ExtractError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ExtractError {
    #[error("corrupt archive: {0}")]
    CorruptArchive(String),
    #[error("path traversal detected: {0}")]
    ZipSlip(String),
    #[error("I/O error: {0}")]
    Io(String),
}
```

**Zip-slip prevention**: Use lexical path normalization (no filesystem access) to prevent path traversal. `canonicalize()` must NOT be used on the entry path because the target file doesn't exist yet:

```rust
fn normalize_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => { result.pop(); }
            std::path::Component::CurDir => {}
            other => result.push(other),
        }
    }
    result
}

// Usage in extraction loop:
for entry in archive.entries()? {
    let entry = entry?;
    let path = entry.path()?;
    let full_path = normalize_path(&dest.join(&path));
    if !full_path.starts_with(dest) {
        return Err(ExtractError::ZipSlip(path.display().to_string()));
    }
    // Create parent dirs, then extract
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    entry.unpack(&full_path)?;
}
```

**Windows `.exe` preservation** (AC-008.3): The tar archive from the Windows build already contains `ecc.exe` — `tar` extraction preserves filenames as-is. No special handling needed beyond not stripping extensions.

### 4.4 Rollback Wiring in the Orchestrator

In `orchestrator.rs`, after `sequential_swap` succeeds (step 11, line 196), the `ecc install` call (step 13, line 202) may fail. Current code returns `ConfigSyncFailed` without rollback.

**New flow**:

```rust
// 11. Swap binaries (returns swapped pairs for potential rollback)
let swapped = swap::sequential_swap(ctx.fs, &extraction_dir, &install_dir, &binaries)?;

// 12. Update shims
let _shim_count = swap::update_shims(ctx.fs, &extraction_dir, &install_dir)?;

// 13. Run ecc install (config sync)
let install_result = ctx.shell.run_command("ecc", &["install"]);
match install_result {
    Ok(output) if output.success() => { /* success path */ }
    Ok(output) => {
        // Config sync failed — rollback
        match swap::rollback_swapped(ctx.fs, &swapped) {
            Ok(()) => {
                return Err(UpdateError::ConfigSyncFailed {
                    reason: format!(
                        "ecc install failed (exit {}): {}. Rolled back to previous version.",
                        output.exit_code, output.stderr
                    ),
                });
            }
            Err(rollback_err) => {
                return Err(UpdateError::RollbackFailed {
                    original: format!("ecc install failed: {}", output.stderr),
                    rollback: rollback_err.to_string(),
                    backup_paths: swapped.iter().map(|(_, b)| b.clone()).collect(),
                });
            }
        }
    }
    Err(e) => { /* same rollback pattern */ }
}

// 14. Clean up backups on success
swap::cleanup_backups(ctx.fs, &swapped);
```

The `rollback_swapped` function:

```rust
pub fn rollback_swapped(
    fs: &dyn FileSystem,
    swapped: &[(PathBuf, PathBuf)],
) -> Result<(), UpdateError> {
    for (target, backup) in swapped {
        restore_backup(fs, backup, target)?;
    }
    Ok(())
}
```

### 4.5 Keyless Sigstore Cosign Verification

**Compile-time constant** (in `crates/ecc-infra/src/github_release.rs`):

```rust
const COSIGN_CERTIFICATE_IDENTITY: &str =
    "https://github.com/LEBOCQTitouan/everything-claude-code/.github/workflows/release.yml@refs/tags/*";
const COSIGN_OIDC_ISSUER: &str = "https://token.actions.githubusercontent.com";
```

**Verification flow**:

1. Download `{artifact}.tar.gz.bundle` from the release to a temp path.
2. Shell out to: `cosign verify-blob --bundle {bundle_path} --certificate-identity-regexp {identity} --certificate-oidc-issuer {issuer} {tarball_path}`
   > **Note**: Uses `--certificate-identity-regexp` (not `--certificate-identity`) because the tag portion of the identity varies per release (e.g., `@refs/tags/v1.0.0` vs `@refs/tags/v1.1.0`). The regexp anchors the workflow path while allowing any tag. This is intentional and more secure than exact-match for a versioned identity.
3. Map exit code: 0 = `Verified`, non-zero = `Failed`, command-not-found = `NotInstalled`.

**Orchestrator enforcement** (AC-007.6): The current orchestrator (line 166-178) treats `CosignResult::NotInstalled` as a warning. This changes to:

```rust
match cosign {
    CosignResult::Verified => { /* print verified message */ }
    CosignResult::NotInstalled => {
        let _ = ctx.fs.remove_dir_all(&temp_dir);
        return Err(UpdateError::CosignUnavailable);
    }
    CosignResult::Failed => {
        let _ = ctx.fs.remove_dir_all(&temp_dir);
        return Err(UpdateError::SecurityVerificationFailed {
            reason: "cosign signature verification failed".to_string(),
        });
    }
}
```

**Release pipeline change** (AC-007.5): In `.github/workflows/release.yml`, line 228-233, change:

```yaml
# Old
cosign sign-blob "$asset" \
  --yes \
  --output-signature "${asset}.sig" \
  --output-certificate "${asset}.bundle" || true

# New
cosign sign-blob "$asset" \
  --yes \
  --bundle "${asset}.bundle" || true
```

### 4.6 Concurrent Update Lock

The orchestrator acquires an exclusive flock at the start of `run_update` using the existing `FileLock` port trait (`crates/ecc-ports/src/lock.rs`). The lock name is `"ecc-update"`. The lock guard is held for the duration of the update and released automatically via RAII drop.

```rust
// At the start of run_update:
let lock_guard = ctx.lock.acquire_with_timeout(
    &install_dir, // repo_root is install_dir for update locks
    "ecc-update",
    Duration::from_secs(5),
).map_err(|e| match e {
    LockError::Timeout(_) => UpdateError::UpdateLocked {
        reason: "Another update is in progress".to_string(),
    },
    other => UpdateError::SwapFailed {
        reason: format!("failed to acquire update lock: {other}"),
    },
})?;
// lock_guard lives until end of function scope (RAII)
```

The `InMemoryLock` test double already exists in `ecc-test-support` for testing.

### 4.7 Environment `current_exe()` Addition

Add to the `Environment` trait in `crates/ecc-ports/src/env.rs`:

```rust
/// Return the path of the currently running executable.
fn current_exe(&self) -> Option<PathBuf>;
```

`OsEnvironment` implements via `std::env::current_exe().ok()`. `MockEnvironment` stores an `Option<PathBuf>` with a `with_current_exe` builder, defaulting to `Some(PathBuf::from("/usr/local/bin/ecc"))`.

The orchestrator's line 181 (`std::env::current_exe()`) changes to:

```rust
let install_dir = ctx.env.current_exe()
    .and_then(|p| p.parent().map(|p| p.to_path_buf()))
    .unwrap_or_else(|| PathBuf::from("/usr/local/bin"));
```

---

## 5. Spec Reference

Concern: dev, Feature: BL-088: ecc update dual-mode deploy

## 6. Coverage Check

Every AC-NNN.N from the spec appears in at least one PC's "Verifies AC" column.

| AC | Covering PCs |
|----|-------------|
| AC-001.1 | PC-025, PC-040, PC-041 |
| AC-001.2 | PC-026 |
| AC-001.3 | PC-027 |
| AC-001.4 | PC-028 |
| AC-001.5 | PC-029 |
| AC-001.6 | PC-030 |
| AC-001.7 | PC-031 |
| AC-001.8 | PC-006, PC-016 |
| AC-001.9 | PC-008, PC-009, PC-015, PC-036 |
| AC-002.1 | PC-010, PC-019, PC-032 |
| AC-002.2 | PC-019 |
| AC-002.3 | PC-020 |
| AC-002.4 | PC-021 |
| AC-003.1 | PC-001, PC-007 |
| AC-003.2 | PC-002 |
| AC-003.3 | PC-003 |
| AC-003.4 | PC-004 |
| AC-003.5 | PC-005 |
| AC-004.1 | PC-017 |
| AC-004.2 | PC-017 |
| AC-004.3 | PC-018 |
| AC-004.4 | *(POSIX flock(2) kernel semantics — lock auto-released on process death; guaranteed by OS, not unit-testable)* |
| AC-005.1 | PC-011, PC-022 |
| AC-005.2 | PC-022 |
| AC-005.3 | PC-012 |
| AC-005.4 | PC-013 |
| AC-005.5 | PC-022 |
| AC-006.1 | PC-037 |
| AC-006.2 | PC-038 |
| AC-006.3 | PC-039 |
| AC-007.1 | PC-034 |
| AC-007.2 | PC-024 |
| AC-007.3 | PC-034 |
| AC-007.4 | PC-035 |
| AC-007.5 | PC-042 |
| AC-007.6 | PC-023 |
| AC-008.1 | PC-003 |
| AC-008.2 | PC-014 |
| AC-008.3 | PC-033 |

**All 39 ACs covered.** Zero uncovered.

## 7. E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | GitHub API | GithubReleaseClient | ReleaseClient | `ecc update --dry-run` against real GitHub API | ignored | GithubReleaseClient modified |
| 2 | Tarball extraction | FlateExtractor | TarballExtractor | Extract real `.tar.gz`, verify files | ignored | FlateExtractor modified |
| 3 | Binary swap | RealFileSystem | FileSystem | Swap binaries in temp dir, verify perms | ignored | swap.rs modified |
| 4 | Config sync | RealExecutor | ShellExecutor | `ecc install` after swap, rollback on failure | ignored | orchestrator config sync modified |
| 5 | Cosign verification | cosign CLI | ShellExecutor | Verify Sigstore-signed blob with real cosign | ignored | cosign verification modified |

### E2E Activation Rules

All 5 boundaries modified in this implementation. Tests 1-5 should be un-ignored during integration testing. Test 1 runs in CI as dry-run.

## 8. Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `CLAUDE.md` | Project | Update | Add `ecc update` to CLI Commands | US-001 |
| 2 | `docs/ARCHITECTURE.md` | Project | Update | Note ureq, flate2, tar, sha2 deps; TarballExtractor port | US-001, US-002 |
| 3 | `docs/adr/ADR-NNNN-ureq-http-client.md` | Project | Create | ADR: ureq over reqwest/self_update | Decision #1 |
| 4 | `docs/adr/ADR-NNNN-keyless-sigstore-cosign.md` | Project | Create | ADR: mandatory keyless Sigstore verification | Decision #2 |
| 5 | `CHANGELOG.md` | Project | Update | feat: ecc update self-update from GitHub Releases | US-001 |
| 6 | `docs/backlog/BL-088-ecc-update-command.md` | Backlog | Update | Status: implemented | BL-088 |
| 7 | `docs/MODULE-SUMMARIES.md` | Project | Update | New module entries | US-002, US-003 |

## 9. SOLID Assessment

**uncle-bob verdict**: Pass with findings

- **CRITICAL (FIXED)**: Zip-slip `canonicalize()` bug — corrected to lexical path normalization
- **MEDIUM**: `ReleaseClient` ISP violation (6 methods mixing query/download/verify) — accepted as pragmatic tradeoff, documented
- **MEDIUM**: `Box<dyn Error>` in port API — **deferred** to follow-up. The existing `BoxError` pattern is used consistently across all port traits in the codebase. Defining `ReleaseError` would be a separate refactor touching all 5 existing methods. The `download_file` addition follows the existing `BoxError` convention for consistency.
- **LOW**: `GITHUB_TOKEN` via `std::env::var` in infra — acceptable, deferrable

## 10. Robert's Oath Check

**robert verdict**: CLEAN with 2 watch items

1. Zip-slip guard (FIXED in design)
2. AC-004.4 traceability: guaranteed by POSIX flock(2) kernel semantics (noted in coverage table)

## 11. Security Notes

**security-reviewer verdict**: 3 MEDIUMs (all addressable in implementation)

1. **Zip-slip** (FIXED): Lexical path normalization, no `canonicalize()` on non-existent paths
2. **Cosign shell-out**: Must use `Command::arg()` (discrete positional args), never shell string interpolation. Paths from local temp files, not raw API filenames.
3. **API tag_name validation**: Validate `tag_name` matches `^v\d+\.\d+\.\d+` before URL interpolation
4. **LOW**: Sanitize ureq error strings to strip HTTP headers before propagation
5. **LOW**: Consider returning error if `current_exe()` returns None instead of fallback

## 12. Rollback Plan

Reverse dependency order of File Changes — if implementation fails, undo in this order:

1. `.github/workflows/release.yml` — revert cosign signing change
2. `xtask/src/deploy.rs` — remove ecc-flock from build list
3. `crates/ecc-cli/src/commands/update.rs` — remove new injections
4. `crates/ecc-test-support/` — remove MockExtractor, revert MockEnvironment
5. `crates/ecc-infra/` — remove tarball_extractor.rs, revert github_release.rs, os_env.rs, Cargo.toml
6. `crates/ecc-app/src/update/` — remove extract.rs, revert orchestrator.rs, swap.rs, context.rs
7. `crates/ecc-ports/` — remove extract.rs, revert env.rs
8. `crates/ecc-domain/src/update/` — remove platform.rs, revert artifact.rs, error.rs, mod.rs
9. `Cargo.toml` — remove workspace dependency additions

---

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID (uncle-bob) | PASS with findings | 1 CRITICAL (fixed), 2 MEDIUM (1 accepted, 1 deferred), 1 LOW |
| Robert | CLEAN | 2 watch items |
| Security | 3 MEDIUMs (implementation-level) | 3 MEDIUM, 2 LOW |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Completeness | 94 | PASS | All 39 ACs covered; all 45 PCs present |
| Correctness | 91 | PASS | Enum migration preserves imports; zip-slip uses lexical normalization |
| Testability | 92 | PASS | TDD phases correctly ordered; mock seam for every I/O port |
| Security | 90 | PASS | certificate-identity-regexp justified; cosign NotInstalled aborts |
| Dependency Order | 93 | PASS | File changes and TDD phases consistent |
| Deferred Items | 88 | PASS | ReleaseError deferral acknowledged; GITHUB_TOKEN env-var acceptable |
| CI/CD Alignment | 90 | PASS | release.yml --bundle change specified; PC-042 CI check |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `ecc-domain/src/update/platform.rs` | Create | AC-003.1 |
| 2 | `ecc-domain/src/update/artifact.rs` | Modify | AC-003.2-5, AC-008.1 |
| 3 | `ecc-domain/src/update/error.rs` | Modify | AC-001.8, AC-004.2, AC-005.3, AC-007.2 |
| 4 | `ecc-domain/src/update/mod.rs` | Modify | AC-003.1 |
| 5 | `ecc-ports/src/env.rs` | Modify | AC-003.1, AC-001.9 |
| 6 | `ecc-ports/src/release.rs` | Modify | AC-007.1, AC-007.5 |
| 7 | `ecc-app/src/update/extract.rs` | Create | AC-002.1, AC-002.3-4 |
| 8 | `ecc-app/src/update/swap.rs` | Modify | AC-005.*, AC-008.2 |
| 9 | `ecc-app/src/update/orchestrator.rs` | Modify | AC-001.8-9, AC-002.2, AC-004.*, AC-005.*, AC-007.6 |
| 10 | `ecc-app/src/update/context.rs` | Modify | AC-004.1, AC-002.1 |
| 11-13 | Ports + app mod.rs | Modify | -- |
| 14 | `ecc-infra/src/github_release.rs` | Modify | AC-001.*, AC-007.* |
| 15 | `ecc-infra/src/tarball_extractor.rs` | Create | AC-002.*, AC-008.3 |
| 16-18 | Infra env, lib, Cargo.toml | Modify | AC-001.9 |
| 19-21 | Test support mocks | Create/Modify | AC-001.9, AC-002.1, AC-007.* |
| 22 | `ecc-cli/src/commands/update.rs` | Modify | AC-004.1, AC-002.1 |
| 23 | `xtask/src/deploy.rs` | Modify | AC-006.* |
| 24 | `.github/workflows/release.yml` | Modify | AC-007.5 |
| 25 | `Cargo.toml` (workspace) | Modify | -- |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-31-ecc-update-dual-mode-deploy/design.md | Full design |
| docs/specs/2026-03-31-ecc-update-dual-mode-deploy/spec.md | Full spec (from prior phase) |
| .claude/workflow/campaign.md | Campaign manifest |
