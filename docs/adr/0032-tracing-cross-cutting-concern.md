# ADR 0032: Tracing as Cross-Cutting Concern — Layer Restrictions

## Status

Accepted (2026-03-30)

## Context

BL-091 replaces `log`/`env_logger` with `tracing`/`tracing-subscriber` for structured, tiered diagnostics. The `tracing` crate is a cross-cutting concern that could leak into any layer. The hexagonal architecture requires clear rules about where tracing is permitted.

The key question: should the domain and port layers be allowed to use `tracing` macros, or should they remain infrastructure-free?

## Decision

**Tracing is forbidden in `ecc-domain` and `ecc-ports`.** Permitted in `ecc-app`, `ecc-infra`, `ecc-cli`, and `ecc-workflow`.

| Layer | Tracing Allowed? | Rationale |
|-------|-----------------|-----------|
| `ecc-domain` | NO | Pure business logic. Zero infrastructure dependencies. |
| `ecc-ports` | NO | Trait definitions only. No implementations. |
| `ecc-app` | YES (facade only) | Application orchestration. Natural instrumentation point. |
| `ecc-infra` | YES | Infrastructure adapters. |
| `ecc-cli` | YES (facade + subscriber) | Subscriber initialization lives here. |
| `ecc-workflow` | YES (facade + subscriber) | Binary crate, same as ecc-cli. |

The `tracing` crate's macros are no-ops when no subscriber is installed, making the facade dependency lightweight. The subscriber (the actual output adapter) is wired exclusively in binary `main.rs` files.

## Consequences

- Domain hook enforcement (`grep -rn "tracing" crates/ecc-domain/src/`) checks for tracing imports alongside existing I/O import checks
- The `LogLevel` value object lives in `ecc-domain` but has zero tracing dependency — it's a pure enum
- `ConfigStore` port uses `RawEccConfig` (strings) to avoid domain dependency in ports; conversion to `LogLevel` happens in `ecc-app`
- Future observability features (BL-092 structured logs) follow the same layer restrictions
