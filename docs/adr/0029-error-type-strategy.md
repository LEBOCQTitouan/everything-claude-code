# ADR 0029: thiserror Enums Per Module, anyhow Only in Binaries

## Status

Accepted (2026-03-28)

## Context

The audit identified 3 competing error strategies in ecc-app: `Result<T, String>` (14 functions), `anyhow::Result` (3 functions), and typed error enums (54 functions). The `anyhow` dependency in a library crate violates the project's Rust coding rules ("Use `anyhow` only in binary crates").

## Decision

Standardize on **thiserror-derived error enums per module** in all library crates:

- `ClawError` for claw/ module (8 functions migrated)
- `MergeError` for merge/ module (4 functions migrated)
- `ConfigAppError` for config/ module (3 functions migrated)
- `InstallError` for install/ module (1 function migrated)
- `PathValidationError` for hook/helpers (1 function migrated)

Binary crates (`ecc-cli`, `ecc-workflow`) keep `anyhow` for ad-hoc error wrapping at the entry point.

## Consequences

- Callers can match on error variants for targeted user messages
- CLI layer maps each variant to operation name + remediation hint
- `Result<T, String>` eliminated from ecc-app public API
- `anyhow` partially removed from ecc-app (worktree.rs still uses it — tracked for future cleanup)
- All future ecc-app modules must define typed error enums
