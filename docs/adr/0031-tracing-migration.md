# ADR 0031: Migrate from log+env_logger to tracing+tracing-subscriber

## Status

Accepted

## Context

The ECC CLI and workflow binaries used `log` crate v0.4 + `env_logger` v0.11 for diagnostics. This provided only warn-level output with a binary --verbose toggle. Users had no way to see hook decisions, state transitions, or component status without reading files manually. The `log` crate lacks structured spans and per-module filtering.

## Decision

Migrate all ECC binaries and crates from `log`+`env_logger` to `tracing`+`tracing-subscriber`:

1. **Tiered verbosity**: -q (error), default (warn), -v (info), -vv (debug), -vvv (trace) via `clap::ArgAction::Count`
2. **ECC_LOG env var**: Custom env var for ECC-specific filtering, follows `RUST_LOG` conventions (per-module: `ECC_LOG=ecc_domain=trace`). `RUST_LOG` supported as deprecated fallback with stderr warning.
3. **ecc-workflow silent default**: Tracing subscriber only initialized when `ECC_LOG` is explicitly set — protects JSON hook output from stderr pollution.
4. **Config persistence**: `~/.ecc/config.toml` with `log-level` key. Precedence: CLI flag > ECC_LOG > RUST_LOG > config > default(warn).
5. **ecc-domain stays pure**: No tracing dependency. The `tracing-log` bridge forwards tracing events to the `log` facade for test capture via `testing_logger`.
6. **All output to stderr**: Zero model cost — diagnostics never enter Claude's context window.

## Consequences

- ~60 `log::` call sites migrated to `tracing::` across 4 crates
- New dependencies: tracing 0.1, tracing-subscriber 0.3 (with env-filter feature)
- `env_logger` removed from ecc-cli and ecc-workflow
- `log` retained as dev-dependency for `testing_logger` test assertions
- New `ecc status` and `ecc config` CLI subcommands
- Config file `~/.ecc/config.toml` created on first `ecc config set`
- BL-092 (structured log management) can build on this tracing foundation
