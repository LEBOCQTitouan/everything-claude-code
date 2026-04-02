---
id: BL-116
title: "Add cargo-mutants mutation testing"
scope: MEDIUM
target: "/spec-dev"
status: open
created: "2026-03-31"
source: "docs/audits/web-radar-2026-03-31.md"
ring: trial
tags: [testing, quality, tooling]
---

## Context

cargo-mutants detects untested code paths by injecting mutations and verifying test failures. Supports sharding for distributed testing and reflink copy-on-write. Integrates with cargo-nextest. Complements the 80%+ coverage target by measuring test quality, not just quantity.

## Prompt

Integrate `cargo-mutants` into the ECC development workflow. (1) Add as a dev tool (cargo install). (2) Run an initial mutation testing pass against the workspace to establish a baseline. (3) Add a `/verify` step or standalone command for mutation testing. (4) Document which crates/modules have the weakest mutation scores. Focus on ecc-domain and ecc-app first as they contain core business logic.

## Acceptance Criteria

- [ ] cargo-mutants installed and configured for the workspace
- [ ] Baseline mutation testing report generated
- [ ] Integration with cargo-nextest for faster mutation runs
- [ ] Weak spots identified and documented
