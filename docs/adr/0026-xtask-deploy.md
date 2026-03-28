# ADR 0026: Cargo xtask Deploy Pattern

## Status

Accepted

## Context

Deploying ECC to a local machine requires 5+ manual steps: building two binaries (ecc, ecc-workflow), installing them, running config sync, setting up shell completions, and editing shell RC files. This friction discourages updates. The project needed an automated deploy command.

Two locations for the tooling crate were considered: `crates/xtask/` (inside the product crate structure) or `xtask/` (at repo root, following Rust ecosystem convention).

## Decision

Place the `xtask` crate at the repository root (`xtask/`), outside the `crates/` hexagonal architecture directory. The crate has zero dependencies on ECC product crates — it shells out to `cargo`, `ecc`, and standard tools via `std::process::Command`.

Key design choices:
- **Managed RC block**: Shell RC edits use `# >>> ecc >>>` / `# <<< ecc <<<` sentinel markers for idempotent updates
- **Backup before first edit**: `~/.zshrc.ecc-backup` created before first modification
- **Atomic writes**: `mktemp` + `rename` for crash-safe RC file updates
- **Dry-run mode**: `--dry-run` flag previews all actions without side effects
- **No product crate dependencies**: xtask is developer tooling, architecturally isolated from the hexagonal stack

## Consequences

- `cargo xtask deploy` automates the full local machine setup in one command
- The xtask crate is semantically separate from product code (at repo root, not in `crates/`)
- Shell RC edits are idempotent — re-running deploy produces no duplicates
- The `#[alias]` in `.cargo/config.toml` enables the `cargo xtask` convention
