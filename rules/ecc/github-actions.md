# ECC GitHub Actions Conventions

Canonical reference for the 4 ECC workflows. Follow these conventions when editing `.github/workflows/`.

## Workflows

### ci.yml — CI

- **Triggers**: `pull_request` on `[main]`, `merge_group`
- **Jobs**: `validate` (checkout, rust-toolchain, cache, cargo test, cargo clippy, cargo fmt --check, cargo semver-checks, ecc validate agents/commands/skills/hooks/rules, **ecc validate claude-md markers --strict** — flags stale `TEMPORARY (BL-NNN)` warnings whose backlog file is missing on disk; emergency brake via `ECC_CLAUDE_MD_MARKERS_DISABLED=1` env var)
- **Concurrency**: `${{ github.workflow }}-${{ github.ref }}`, cancel-in-progress
- **Permissions**: `contents: read`

### release.yml — Release (cargo-dist managed)

- **Triggers**: `push tags: ['v*']`
- **Jobs**: `plan` (version/tag validation), `build-local` (cross-compile matrix: aarch64-apple-darwin, x86_64-apple-darwin, x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu, x86_64-pc-windows-msvc), `host` (tarball packaging, checksum, gh release create), `cosign-sign` (non-blocking Sigstore signing), `announce`
- **Config**: `dist.toml` at workspace root — targets, binaries, includes, checksums, installers
- **Caching**: `Swatinem/rust-cache@v2` per target
- **Permissions**: `contents: write`
- **Custom jobs**: `cosign-sign` declared in `dist.toml` `[plan.jobs]` for preservation across `cargo dist generate-ci`

### cd.yml — CD

- **Triggers**: `push` on `[main]`
- **Jobs**: `auto-tag` (gate check: skip `[skip cd]`, skip bot actors, require release label; then version bump, tag, push)
- **Concurrency**: `cd-main`, no cancel-in-progress (serialize)
- **Permissions**: `contents: write`

### maintenance.yml — Scheduled Maintenance

- **Triggers**: `schedule` (weekly Monday 9am UTC: `0 9 * * 1`), `workflow_dispatch`
- **Jobs**: `stale` (mark/close stale issues and PRs via `actions/stale`)
- **Permissions**: `contents: read`, `issues: write`, `pull-requests: write`

## Conventions

- Always set `permissions` to least privilege — never use default (full write)
- Use `concurrency` groups to prevent duplicate runs
- Pin action versions (`@v5`, `@v4`) for reproducibility — never use `@latest`
- Required status checks reference **job names** (e.g., `validate`), not workflow filenames
- Protect secrets: never echo, use environment-level secrets for deploy credentials
- Cross-compilation matrix must cover all 5 targets listed in release.yml
- Tag-based releases use `v*` pattern — semver tags only
- CD pipeline guards: `[skip cd]` in commit message, bot actor check, release label requirement

For general CI/CD patterns and Claude Code workflow templates, see `skills/ci-cd-workflows/SKILL.md`.
