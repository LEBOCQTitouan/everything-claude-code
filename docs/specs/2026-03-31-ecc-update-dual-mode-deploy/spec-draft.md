# Spec: BL-088 — ecc update Dual-Mode Deploy

## Problem Statement

ECC has a partially-implemented self-update mechanism (`ecc update`) that cannot function in production. The `GithubReleaseClient` adapter is a stub returning errors on all methods, downloaded tarballs are never extracted, and Apple Silicon users hit an `UnsupportedPlatform` error due to a label mismatch between `Architecture::Arm64.as_label()` ("aarch64") and `ArtifactName::resolve` expectations ("arm64"). Developers using `cargo xtask deploy` also miss the `ecc-flock` binary. This spec completes the feature end-to-end, including Windows support, mandatory keyless Sigstore cosign verification, and rollback on failure.

## Research Summary

- **`self_update` crate** is prior art but bundles its own HTTP/archive/replace logic — conflicts with ECC's existing orchestrator. Reference its patterns but don't adopt wholesale.
- **`ureq` over `reqwest`** for sync CLI: no tokio dependency, smaller binary, no unsafe code. Best fit for CLI without async runtime.
- **Atomic binary replacement**: rustup's pattern — write to temp file in same directory, chmod, then rename() (atomic on POSIX). On Windows, rename running binary to `.old` first.
- **`flate2` + `tar`** for tarball extraction with mandatory zip-slip prevention (canonicalize paths, assert within target directory).
- **Checksum verification is mandatory** before extraction. Cosign signature verification provides defense in depth.
- **`semver` crate** for version comparison (already used in ECC).
- **Pitfalls**: GitHub API rate limiting (support GITHUB_TOKEN), temp file cleanup on failure, check write permissions early, preserve Unix file permissions after extraction.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Use `ureq` as HTTP client | Sync, no tokio, small binary, no unsafe. CLI has no async runtime. | Yes |
| 2 | Mandatory keyless Sigstore cosign verification | Defense in depth — checksum + Sigstore OIDC-based signature. Verify via --certificate-identity + --certificate-oidc-issuer. No key to manage or rotate. | Yes |
| 3 | Move Architecture/Platform to ecc-domain | Domain concepts, not port concepts. Re-export from ecc-ports for compat. | No |
| 4 | Include Windows self-update | release.yml already builds Windows. Parity across platforms. | No |
| 5 | ureq streaming downloads | Never buffer entire tarball in memory. Stream to temp file. | No |
| 6 | flock-based concurrent update lock | Prevent installation corruption from parallel ecc update runs. | No |

## User Stories

### US-001: Wire GithubReleaseClient to GitHub Releases API

**As a** production user, **I want** `ecc update` to query and download releases from GitHub, **so that** I can self-update without manual downloads.

#### Acceptance Criteria

- AC-001.1: Given `ecc update` is run, when the `GithubReleaseClient` queries latest version, then it returns the latest stable release version from the GitHub API
- AC-001.2: Given `ecc update --version 4.3.0`, when the version exists, then it returns the correct `ReleaseInfo` for that version
- AC-001.3: Given a tarball download, when streaming to disk, then progress callbacks are invoked and no full-tarball buffer exists in memory
- AC-001.4: Given a downloaded tarball, when checksum verification runs, then SHA256 is computed locally and compared against `checksums-sha256.txt` from the release
- AC-001.5: Given no network connectivity, when any API call is made, then a clear `NetworkError` is returned with retry guidance
- AC-001.6: Given GitHub API rate limiting (403), when detected, then `RateLimited` error is returned with reset time
- AC-001.7: Given `GITHUB_TOKEN` in environment, when API calls are made, then the token is included as Authorization header
- AC-001.8: Given `ecc update` starts, when the install directory is not writable, then a clear `PermissionDenied` error is returned before any download begins
- AC-001.9: Given the orchestrator determines the install directory, when it resolves the current executable path, then it uses the `Environment` port trait (not direct `std::env::current_exe()` in the app layer)

#### Dependencies

- Depends on: none

### US-002: Implement Tarball Extraction

**As a** production user, **I want** downloaded tarballs to be properly extracted, **so that** the binary swap operates on real files.

#### Acceptance Criteria

- AC-002.1: Given a `.tar.gz` file, when extraction runs, then it decompresses into a temp directory with `bin/ecc` and `bin/ecc-workflow` (ecc-flock is dev-only, not in release tarballs)
- AC-002.2: Given the extraction directory, when sequential swap runs, then binaries from `bin/` are swapped into the install directory
- AC-002.3: Given a corrupt archive, when extraction fails, then a clear `SwapFailed` error is returned with no partial state
- AC-002.4: Given extraction entries with `../` paths, when zip-slip is detected, then extraction is aborted with a security error

#### Dependencies

- Depends on: none

### US-003: Fix Platform-Architecture Mapping

**As a** macOS ARM64 user, **I want** `ecc update` to correctly resolve my platform, **so that** it does not fail with `UnsupportedPlatform`.

#### Acceptance Criteria

- AC-003.1: Given `Architecture` and `Platform` enums, when referenced from ecc-domain, then they are defined in `ecc-domain` and re-exported from `ecc-ports`
- AC-003.2: Given `ArtifactName::resolve(Platform::MacOS, Architecture::Arm64)`, when called, then it returns `ecc-darwin-arm64`
- AC-003.3: Given all 5 release targets from release.yml, when `ArtifactName::resolve` is called for each, then each resolves to the correct artifact name
- AC-003.4: Given `Architecture::Unknown`, when resolve is called, then a clear `UnsupportedPlatform` error is returned
- AC-003.5: Given `ArtifactName::resolve` accepts `Platform` and `Architecture` enums, when the old string-based API is removed, then all callers are migrated and `Architecture::as_label()` is aligned with artifact naming or removed

#### Dependencies

- Depends on: none

### US-004: Add Concurrent Update Lock

**As a** user who may accidentally run `ecc update` twice, **I want** concurrent updates prevented, **so that** I don't corrupt my installation.

#### Acceptance Criteria

- AC-004.1: Given `ecc update` starts, when it acquires an exclusive flock, then the update proceeds
- AC-004.2: Given another instance holds the lock, when a new instance starts, then it exits with "Another update is in progress"
- AC-004.3: Given the update completes or fails, when the process exits, then the lock is released
- AC-004.4: Given a stale lock from a crashed process, when a new update starts, then the lock is acquired (OS flock auto-releases)

#### Dependencies

- Depends on: none

### US-005: Add Rollback on Post-Swap Failure

**As a** user, **I want** binaries rolled back if `ecc install` fails after swap, **so that** I'm not left with updated binaries but broken config.

#### Acceptance Criteria

- AC-005.1: Given binary swap succeeds, when `ecc install` fails, then all swapped binaries are restored from `.bak` backups
- AC-005.2: Given rollback succeeds, when the error is reported, then it includes "rolled back to previous version"
- AC-005.3: Given rollback also fails, when the error is reported, then it includes both failures and backup paths for manual recovery
- AC-005.4: Given a successful update, when everything completes, then `.bak` files are cleaned up
- AC-005.5: Given binary swap succeeds and `ecc install` fails, when the orchestrator handles `ConfigSyncFailed`, then it invokes `rollback_swapped` to restore all swapped binaries from `.bak` backups before returning the error

#### Dependencies

- Depends on: none

### US-006: Complete xtask deploy with all binaries

**As a** developer, **I want** `cargo xtask deploy` to build and install all binaries including ecc-flock, **so that** my local environment is complete.

#### Acceptance Criteria

- AC-006.1: Given `cargo xtask deploy`, when build runs, then it builds ecc-cli, ecc-workflow, and ecc-flock
- AC-006.2: Given install step, when copying binaries, then all three are installed to `~/.cargo/bin/`
- AC-006.3: Given `--dry-run`, when summary prints, then it lists all three binaries

#### Dependencies

- Depends on: none

### US-007: Mandatory Keyless Sigstore Cosign Verification

**As a** security-conscious user, **I want** cosign signature verification to be mandatory using keyless Sigstore verification, **so that** I can trust the downloaded binary was signed by the official GitHub Actions workflow.

#### Acceptance Criteria

- AC-007.1: Given a downloaded tarball, when cosign verification runs, then it downloads the `.bundle` file from the release and verifies via `cosign verify-blob --certificate-identity=<workflow-identity> --certificate-oidc-issuer=https://token.actions.githubusercontent.com`
- AC-007.2: Given cosign verification fails, when the result is `CosignResult::Failed`, then the update is aborted with a security error
- AC-007.3: Given cosign is not installed, when verification is attempted, then the update is aborted with guidance to install cosign
- AC-007.4: Given the certificate identity, when it is embedded in the binary, then it is stored as a compile-time constant matching the release workflow path (e.g., `https://github.com/<owner>/<repo>/.github/workflows/release.yml@refs/tags/v*`)
- AC-007.5: Given the release pipeline, when cosign signs artifacts, then it uses `cosign sign-blob --bundle "${asset}.bundle"` to produce a single Sigstore bundle file (containing both signature and certificate) alongside each tarball — replacing the current separate `--output-signature` + `--output-certificate` approach
- AC-007.6: Given cosign verification is mandatory, when the orchestrator encounters `CosignResult::NotInstalled`, then the update is aborted (not treated as a warning) — the orchestrator must be updated to enforce this

#### Dependencies

- Depends on: US-001

### US-008: Windows Self-Update Support

**As a** Windows user, **I want** `ecc update` to work on Windows, **so that** I can self-update like Unix users.

#### Acceptance Criteria

- AC-008.1: Given `ArtifactName::resolve(Platform::Windows, Architecture::Amd64)`, when called, then it returns `ecc-win32-x64`
- AC-008.2: Given Windows binary replacement, when the running binary cannot be overwritten, then the old binary is renamed to `.old` before the new one is placed
- AC-008.3: Given Windows extraction, when `.tar.gz` is extracted, then `.exe` extensions are preserved on binaries

#### Dependencies

- Depends on: US-003

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `ecc-domain/src/update/` | Domain | Add `Architecture`, `Platform` enums; change `ArtifactName::resolve` to accept enums; add Windows variant |
| `ecc-ports/src/env.rs` | Ports | Re-export `Architecture`, `Platform` from domain; remove local definitions |
| `ecc-ports/src/release.rs` | Ports | No change (trait is already defined) |
| `ecc-app/src/update/orchestrator.rs` | Application | Wire real extraction, fix `current_exe` bypass, add flock, add rollback |
| `ecc-app/src/update/swap.rs` | Application | Add `rollback_swapped` function, Windows swap path |
| `ecc-infra/src/github_release.rs` | Infrastructure | Replace stub with real ureq-based implementation |
| `ecc-infra/Cargo.toml` | Infrastructure | Add `ureq`, `flate2`, `tar` dependencies |
| `ecc-cli/src/commands/update.rs` | CLI | Minor — inject real adapter |
| `ecc-test-support/` | Test | Update `MockEnvironment` for domain enums |
| `xtask/src/deploy.rs` | Dev tooling | Add ecc-flock to build/install list |
| `.github/workflows/release.yml` | CI | Update cosign signing to use `--bundle` flag for Sigstore bundle format |

## Constraints

- `ecc-domain` must remain zero-I/O (no `std::fs`, `std::net`, `std::process`)
- Dependency direction must remain inward (domain → ports → app → infra → CLI)
- All 5 release targets must have matching `ArtifactName` variants
- Cosign certificate identity must be pinned at compile time (keyless Sigstore model)
- ecc-flock is dev-only — not included in release tarballs
- Streaming downloads — no full-tarball memory buffering
- Zip-slip prevention in tarball extraction

## Non-Requirements

- Delta/patch updates (full tarball replacement only)
- Rollback to arbitrary previous versions (only immediate failure rollback)
- Auto-update background checking
- Async HTTP (ureq is sync)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| ReleaseClient / GithubReleaseClient | Stub → real implementation | Network calls to GitHub API; needs `#[ignore]` integration tests + CI dry-run test |
| Environment / OsEnvironment | Enum relocation | Import path change; re-export maintains compat |
| FileSystem / RealFileSystem | Tarball extraction | Real filesystem I/O for extraction + swap |
| ShellExecutor / RealExecutor | ecc install call | Post-swap config sync; rollback on failure |
| TerminalIO / RealTerminal | Progress display | Download progress indicator |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New CLI command docs | CLAUDE.md | `ecc update` usage | Add to CLI Commands section |
| Architecture change | ARCHITECTURE.md | Update module descriptions | Note ureq dependency in ecc-infra |
| ADR | docs/adr/ | ADR-NNNN: ureq over reqwest | Create |
| ADR | docs/adr/ | ADR-NNNN: mandatory keyless Sigstore cosign verification | Create |
| Backlog status | docs/backlog/ | BL-088 | Update to implemented |

## Open Questions

None — all resolved during grill-me interview.
