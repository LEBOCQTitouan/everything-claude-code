# ADR 0034: Migrate from serde_yml to serde-saphyr

## Status

Accepted (2026-03-30)

## Context

`serde_yml 0.0.12` has RUSTSEC-2025-0068 — a documented segfault in its `Serializer.emitter`. The crate is archived, was flagged by the community for AI-generated unsound code, and carries supply chain risk. It is used in `ecc-domain` (the most stable crate), propagating risk to all downstream crates ([CORR-006] from full audit 2026-03-29).

Two replacement crates were evaluated:

- **serde-yaml-ng**: Near drop-in API replacement for serde-yaml, but still uses `unsafe-libyaml` under the hood
- **serde-saphyr**: Pure Rust, panic-free on malformed input, no `unsafe-libyaml`, faster in benchmarks

Our usage is minimal: only `from_str()` for YAML frontmatter parsing in backlog entries. No `Value`, `Mapping`, or serialization APIs are used.

## Decision

Use `serde-saphyr 0.0.22` as the replacement.

## Consequences

- Eliminates RUSTSEC-2025-0068 advisory
- Removes the `unsafe-libyaml` transitive dependency from the build
- Pure Rust parser is panic-free on malformed input — better safety properties
- `from_str()` API is compatible — no behavioral changes in backlog parsing
- Error messages may differ in format (acceptable — no tests assert on error text)
- If serde-saphyr is ever deprecated, serde-yaml-ng remains as a fallback
