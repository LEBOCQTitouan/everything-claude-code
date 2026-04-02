# Tasks: BL-088 — ecc update Dual-Mode Deploy

Spec: [spec.md](./spec.md) | Design: [design.md](./design.md)

## Pass Conditions

| ID | Description | Status | Trail |
|----|-------------|--------|-------|
| PC-001 | Platform/Architecture enums in ecc-domain | done | green@2026-03-31T22:30:00Z |
| PC-002 | ArtifactName::resolve(MacOS, Arm64) → ecc-darwin-arm64 | done | green@2026-03-31T22:30:00Z |
| PC-003 | All 5 release targets resolve correctly | done | green@2026-03-31T22:30:00Z |
| PC-004 | Unknown platform → UnsupportedPlatform error | done | green@2026-03-31T22:30:00Z |
| PC-005 | Enum-based resolve, string API removed | done | green@2026-03-31T22:30:00Z |
| PC-006 | New error variants (PermissionDenied, UpdateLocked, etc.) | done | green@2026-03-31T22:30:00Z |
| PC-007 | ecc-ports re-exports Architecture/Platform from domain | done | green@2026-03-31T22:45:00Z |
| PC-008 | Environment trait includes current_exe() | done | green@2026-03-31T22:45:00Z |
| PC-009 | MockEnvironment with_current_exe builder | done | green@2026-03-31T22:45:00Z |
| PC-010 | TarballExtractor port + MockExtractor | done | green@2026-03-31T22:45:00Z |
| PC-045 | ReleaseClient + MockReleaseClient compile with new methods | done | green@2026-03-31T22:45:00Z |
| PC-011 | rollback_swapped restores binaries from .bak | done | green@2026-03-31T23:00:00Z |
| PC-012 | rollback failure returns both errors | done | green@2026-03-31T23:00:00Z |
| PC-013 | cleanup_backups removes .bak files | done | green@2026-03-31T23:00:00Z |
| PC-014 | Windows swap rename-to-.old | done | green@2026-03-31T23:00:00Z |
| PC-015 | Orchestrator uses ctx.env.current_exe() | done | green@2026-03-31T23:20:00Z |
| PC-016 | Permission check before download | done | green@2026-03-31T23:20:00Z |
| PC-017 | Flock acquisition, UpdateLocked on conflict | done | green@2026-03-31T23:20:00Z |
| PC-018 | Lock released on success and failure | done | green@2026-03-31T23:20:00Z |
| PC-019 | Orchestrator wires extraction | done | green@2026-03-31T23:20:00Z |
| PC-020 | Corrupt archive → SwapFailed | done | green@2026-03-31T23:20:00Z |
| PC-021 | Zip-slip path traversal rejected | done | green@2026-03-31T23:20:00Z |
| PC-022 | ConfigSyncFailed triggers rollback | done | green@2026-03-31T23:20:00Z |
| PC-023 | Cosign NotInstalled aborts update | done | green@2026-03-31T23:20:00Z |
| PC-024 | Cosign Failed → SecurityVerificationFailed | done | green@2026-03-31T23:20:00Z |
| PC-025 | GithubReleaseClient latest_version parses API | done | green@2026-04-01T00:00:00Z |
| PC-026 | get_version queries correct tag | done | green@2026-04-01T00:00:00Z |
| PC-027 | Streaming download with progress | done | green@2026-04-01T00:00:00Z |
| PC-028 | Checksum verification SHA256 | done | green@2026-04-01T00:00:00Z |
| PC-029 | Network error handling | done | green@2026-04-01T00:00:00Z |
| PC-030 | Rate limiting (HTTP 403) | done | green@2026-04-01T00:00:00Z |
| PC-031 | GITHUB_TOKEN auth header | done | green@2026-04-01T00:00:00Z |
| PC-032 | FlateExtractor extracts valid tarball | done | green@2026-04-01T00:00:00Z |
| PC-033 | Windows .exe preservation | done | green@2026-04-01T00:00:00Z |
| PC-034 | Cosign bundle verification | done | green@2026-04-01T00:00:00Z |
| PC-035 | Certificate identity compile-time constant | done | green@2026-04-01T00:00:00Z |
| PC-036 | OsEnvironment current_exe | done | green@2026-04-01T00:00:00Z |
| PC-037 | xtask builds three binaries | done | green@2026-04-01T00:30:00Z |
| PC-038 | xtask installs three binaries | done | green@2026-04-01T00:30:00Z |
| PC-039 | xtask dry-run lists three | done | green@2026-04-01T00:30:00Z |
| PC-040 | dry-run integration (mocked) | done | green@2026-04-01T00:30:00Z |
| PC-041 | Full orchestrator integration | done | green@2026-04-01T00:30:00Z |
| PC-042 | release.yml cosign bundle grep | done | green@2026-04-01T00:30:00Z |
| PC-043 | cargo clippy -- -D warnings | done | green@2026-04-01T01:00:00Z |
| PC-044 | cargo build --release | done | green@2026-04-01T01:00:00Z |

## Post-TDD

| Phase | Status | Trail |
|-------|--------|-------|
| E2E tests | done@2026-04-01T01:10:00Z | No E2E tests required (all #[ignore]) |
| Code review | done@2026-04-01T01:15:00Z | 2 CRITICAL + 4 HIGH fixed in 6 commits |
| Doc updates | done@2026-04-01T01:25:00Z | CLAUDE.md, CHANGELOG, 2 ADRs, backlog |
| Supplemental docs | done@2026-04-01T01:30:00Z | MODULE-SUMMARIES + diagrams dispatched |
| Write implement-done.md | done@2026-04-01T01:35:00Z | .claude/workflow/implement-done.md |
