# Tasks: BL-088 — ecc update Dual-Mode Deploy

Spec: [spec.md](./spec.md) | Design: [design.md](./design.md)

## Pass Conditions

| ID | Description | Status | Trail |
|----|-------------|--------|-------|
| PC-001 | Platform/Architecture enums in ecc-domain | pending | |
| PC-002 | ArtifactName::resolve(MacOS, Arm64) → ecc-darwin-arm64 | pending | |
| PC-003 | All 5 release targets resolve correctly | pending | |
| PC-004 | Unknown platform → UnsupportedPlatform error | pending | |
| PC-005 | Enum-based resolve, string API removed | pending | |
| PC-006 | New error variants (PermissionDenied, UpdateLocked, etc.) | pending | |
| PC-007 | ecc-ports re-exports Architecture/Platform from domain | pending | |
| PC-008 | Environment trait includes current_exe() | pending | |
| PC-009 | MockEnvironment with_current_exe builder | pending | |
| PC-010 | TarballExtractor port + MockExtractor | pending | |
| PC-045 | ReleaseClient + MockReleaseClient compile with new methods | pending | |
| PC-011 | rollback_swapped restores binaries from .bak | pending | |
| PC-012 | rollback failure returns both errors | pending | |
| PC-013 | cleanup_backups removes .bak files | pending | |
| PC-014 | Windows swap rename-to-.old | pending | |
| PC-015 | Orchestrator uses ctx.env.current_exe() | pending | |
| PC-016 | Permission check before download | pending | |
| PC-017 | Flock acquisition, UpdateLocked on conflict | pending | |
| PC-018 | Lock released on success and failure | pending | |
| PC-019 | Orchestrator wires extraction | pending | |
| PC-020 | Corrupt archive → SwapFailed | pending | |
| PC-021 | Zip-slip path traversal rejected | pending | |
| PC-022 | ConfigSyncFailed triggers rollback | pending | |
| PC-023 | Cosign NotInstalled aborts update | pending | |
| PC-024 | Cosign Failed → SecurityVerificationFailed | pending | |
| PC-025 | GithubReleaseClient latest_version parses API | pending | |
| PC-026 | get_version queries correct tag | pending | |
| PC-027 | Streaming download with progress | pending | |
| PC-028 | Checksum verification SHA256 | pending | |
| PC-029 | Network error handling | pending | |
| PC-030 | Rate limiting (HTTP 403) | pending | |
| PC-031 | GITHUB_TOKEN auth header | pending | |
| PC-032 | FlateExtractor extracts valid tarball | pending | |
| PC-033 | Windows .exe preservation | pending | |
| PC-034 | Cosign bundle verification | pending | |
| PC-035 | Certificate identity compile-time constant | pending | |
| PC-036 | OsEnvironment current_exe | pending | |
| PC-037 | xtask builds three binaries | pending | |
| PC-038 | xtask installs three binaries | pending | |
| PC-039 | xtask dry-run lists three | pending | |
| PC-040 | dry-run integration (mocked) | pending | |
| PC-041 | Full orchestrator integration | pending | |
| PC-042 | release.yml cosign bundle grep | pending | |
| PC-043 | cargo clippy -- -D warnings | pending | |
| PC-044 | cargo build --release | pending | |

## Post-TDD

| Phase | Status | Trail |
|-------|--------|-------|
| E2E tests | pending | |
| Code review | pending | |
| Doc updates | pending | |
| Supplemental docs | pending | |
| Write implement-done.md | pending | |
