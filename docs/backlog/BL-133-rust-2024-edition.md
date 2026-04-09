---
id: BL-133
title: "Migrate workspace to Rust 2024 edition"
scope: MEDIUM
target: "/spec-dev"
status: open
created: "2026-04-09"
source: "docs/audits/web-radar-2026-04-09.md"
ring: adopt
tags: [ecosystem, edition, toolchain]
---

## Context

Rust 2024 edition is stable since Rust 1.85.0 (Feb 2025). Per-crate incremental migration supported — mixed editions interoperate. `cargo fix --edition` automates most changes.

## Prompt

Migrate the 9-crate ECC workspace from Rust 2021 to Rust 2024 edition. Use `cargo fix --edition` per crate, then verify all tests pass. Handle any edition-specific changes (lifetime elision rules, `unsafe_op_in_unsafe_fn` default, `gen` keyword reservation). Incremental: migrate one crate at a time starting from leaf crates (ecc-domain, ecc-ports).

## Acceptance Criteria

- [ ] All 9 crates declare `edition = "2024"` in Cargo.toml
- [ ] `cargo fix --edition` applied per crate
- [ ] All tests pass
- [ ] cargo clippy passes
