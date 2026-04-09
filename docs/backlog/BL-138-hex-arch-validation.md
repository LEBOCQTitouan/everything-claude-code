---
id: BL-138
title: "Evaluate hex crate for compile-time architecture boundary enforcement"
scope: MEDIUM
target: "/spec-dev"
status: open
created: "2026-04-09"
source: "docs/audits/web-radar-2026-04-09.md"
ring: assess
tags: [architecture, hexagonal, validation]
---

## Context

The hex crate (2025) provides reusable types, traits, and architectural validation for hexagonal/ports-and-adapters in Rust. Could enforce port boundaries as a compile-time or lint-time gate instead of convention-only enforcement.

## Prompt

Evaluate the hex/hexser crate for enforcing hexagonal architecture boundaries in ECC. Compare with current convention-based enforcement (ecc-domain zero-I/O hook, dependency direction in crates/CLAUDE.md). Assess: does hex add value beyond what we enforce via Cargo.toml dependency rules? Is it mature enough for production use?

## Acceptance Criteria

- [ ] Evaluation report comparing hex vs current enforcement
- [ ] Decision: adopt, defer, or reject
- [ ] If adopt: integration plan with affected crates
