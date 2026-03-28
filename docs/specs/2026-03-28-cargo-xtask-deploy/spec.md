# Spec: BL-087 — Cargo xtask deploy

## Problem Statement

Deploying ECC to the local machine requires 5+ manual steps: `cargo install` for two binaries (ecc, ecc-workflow), `ecc install` for config sync, shell completion generation, and manual shell RC file editing for PATH, completions, and statusline. This friction discourages updates and causes stale installations.

## Research Summary

- Cargo xtask pattern (matklad): convention places crate at repo root `xtask/`, alias in `.cargo/config.toml`
- Managed block pattern (conda, pyenv, rustup): sentinel markers `# >>> tool >>>` / `# <<< tool <<<` for idempotent RC edits
- Shell completion paths: zsh `~/.zfunc/_cmd`, bash `~/.local/share/bash-completion/completions/cmd`, fish auto-discovers from `~/.config/fish/completions/`

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | xtask/ at repo root (not crates/) | Rust convention, semantic separation from product code | Yes |
| 2 | Both binaries (ecc + ecc-workflow) | Both needed for pipeline and hooks | No |
| 3 | Managed RC block with backup | Idempotent edits via sentinel markers, backup before first edit | No |
| 4 | Shell out to ecc install (not link) | No dependency on ECC crates — xtask is isolated | No |
| 5 | 100% test coverage on RC logic | Critical path — shell RC editing must be bulletproof | No |

## User Stories

### US-001: Scaffold xtask crate

**As a** developer, **I want** a new `xtask` crate at the repo root, **so that** I can run `cargo xtask deploy`.

#### Acceptance Criteria

- AC-001.1: `xtask/Cargo.toml` exists with `[[bin]] name = "xtask"`, depends on clap + anyhow (workspace)
- AC-001.2: `xtask/src/main.rs` exists with clap-based CLI
- AC-001.3: Workspace `Cargo.toml` includes `"xtask"` in members
- AC-001.4: `.cargo/config.toml` has `[alias] xtask = "run --package xtask --"`
- AC-001.5: `cargo xtask deploy --help` prints usage with `--dry-run` flag
- AC-001.6: `cargo build --workspace` succeeds
- AC-001.7: No dependencies on ECC product crates

#### Dependencies
- Depends on: none

### US-002: Build release binaries

**As a** deployer, **I want** deploy to build ecc and ecc-workflow in release mode, **so that** optimized binaries are available.

#### Acceptance Criteria

- AC-002.1: Runs `cargo build --release -p ecc-cli -p ecc-workflow`
- AC-002.2: Build failure aborts deploy with clear error
- AC-002.3: Dry-run prints command without executing
- AC-002.4: Summary reports build status

#### Dependencies
- Depends on: US-001

### US-003: Install binaries

**As a** deployer, **I want** built binaries copied to `~/.cargo/bin/`, **so that** they're on PATH.

#### Acceptance Criteria

- AC-003.1: Copies `target/release/ecc` and `target/release/ecc-workflow` to cargo bin dir
- AC-003.2: Detects cargo bin from `$CARGO_HOME/bin` (fallback `~/.cargo/bin/`)
- AC-003.3: Creates target directory if missing
- AC-003.4: Overwrites existing binaries (idempotent)
- AC-003.5: Dry-run prints without copying
- AC-003.6: Summary reports install paths

#### Dependencies
- Depends on: US-002

### US-004: Run ecc install

**As a** deployer, **I want** `ecc install` run automatically, **so that** config is synced.

#### Acceptance Criteria

- AC-004.1: Invokes freshly installed `ecc install`
- AC-004.2: Failure is non-fatal (warning, continues)
- AC-004.3: Dry-run prints without executing
- AC-004.4: Summary reports status

#### Dependencies
- Depends on: US-003

### US-005: Shell completions

**As a** deployer, **I want** shell completions generated and installed, **so that** tab-completion works.

#### Acceptance Criteria

- AC-005.1: Detects shell from `$SHELL` (zsh/bash/fish)
- AC-005.2: Runs `ecc completion <shell>` and captures output
- AC-005.3: Writes to correct path (zsh: `~/.zfunc/_ecc`, bash: `~/.local/share/bash-completion/completions/ecc`, fish: `~/.config/fish/completions/ecc.fish`)
- AC-005.4: Creates parent directories if missing
- AC-005.5: Unsupported shell warns and skips
- AC-005.6: Dry-run prints without writing
- AC-005.7: Overwrites existing (idempotent)

#### Dependencies
- Depends on: US-003

### US-006: PATH in RC file

**As a** deployer, **I want** `~/.cargo/bin` added to PATH in my shell RC, **so that** ecc is available in new sessions.

#### Acceptance Criteria

- AC-006.1: Detects RC file (zsh: `~/.zshrc`, bash: `~/.bashrc` or `~/.bash_profile`, fish: `~/.config/fish/config.fish`)
- AC-006.2: Skips if already present
- AC-006.3: Adds inside managed block (`# >>> ecc >>>` / `# <<< ecc <<<`)
- AC-006.4: No duplicates on re-run
- AC-006.5: Dry-run prints without writing

#### Dependencies
- Depends on: US-009

### US-007: Completion source in RC

**As a** deployer, **I want** completion loading line in my RC, **so that** completions auto-load.

#### Acceptance Criteria

- AC-007.1: Adds appropriate line (zsh: fpath+compinit, bash: source, fish: no-op)
- AC-007.2: Skips if already present
- AC-007.3: No duplicates on re-run
- AC-007.4: Dry-run prints without writing

#### Dependencies
- Depends on: US-005, US-009

### US-008: Statusline validation

**As a** deployer, **I want** statusline installation verified, **so that** I know it's working.

#### Acceptance Criteria

- AC-008.1: Runs `ecc validate statusline`
- AC-008.2: Failure is non-fatal (warning)
- AC-008.3: Dry-run prints without executing

#### Dependencies
- Depends on: US-004

### US-009: Managed RC block

**As a** deployer, **I want** RC edits in a managed block with backup, **so that** re-runs are safe.

#### Acceptance Criteria

- AC-009.1: All edits between `# >>> ecc >>>` / `# <<< ecc <<<` markers
- AC-009.2: Existing block replaced (not appended)
- AC-009.3: No write if content unchanged
- AC-009.4: Backup created before first edit (`<rcfile>.ecc-backup`)
- AC-009.5: Atomic writes (mktemp + mv)
- AC-009.6: Unit tests use string-based inputs (no real filesystem)

#### Dependencies
- Depends on: none

### US-010: Dry-run mode

**As a** cautious deployer, **I want** `--dry-run` to preview all actions without performing them.

#### Acceptance Criteria

- AC-010.1: `--dry-run` is a clap flag
- AC-010.2: Every step prints `[dry-run]` prefix
- AC-010.3: No files created/modified/deleted
- AC-010.4: Exit code 0

#### Dependencies
- Depends on: none

### US-011: Summary output

**As a** deployer, **I want** a clear summary of what was done.

#### Acceptance Criteria

- AC-011.1: Actions reported as `[installed]`, `[skipped]`, `[added]`, `[warning]`, `[dry-run]`
- AC-011.2: Summary to stdout, errors to stderr
- AC-011.3: Includes all action details
- AC-011.4: Exit 0 on success, non-zero on fatal failure

#### Dependencies
- Depends on: all

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `xtask/` (new) | Developer tooling | New crate at repo root |
| `Cargo.toml` (root) | Build | Add xtask to workspace |
| `.cargo/config.toml` (new) | Build | Cargo alias |

## Constraints

- No dependencies on ECC product crates
- POSIX shells only (no Windows)
- Idempotent RC edits
- Atomic writes for RC files
- 100% test coverage on RC block logic

## Non-Requirements

- Remote/cross-machine deployment
- Windows support
- Auto-update daemon
- Modifications to existing product crates

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| xtask binary | New entry point | Need integration tests for deploy flow |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New crate | Minor | CLAUDE.md | Add `cargo xtask deploy` |
| New pattern | Minor | Glossary | Add xtask, managed RC block |
| Architecture | Minor | ADR | ADR for xtask pattern |
| Feature | Minor | CHANGELOG | Add BL-087 entry |

## Open Questions

None — all resolved during grill-me interview.
