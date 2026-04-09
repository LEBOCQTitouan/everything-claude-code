---
id: BL-135
title: "Add cargo-llvm-cov coverage gate to CI"
scope: LOW
target: "direct edit"
status: open
created: "2026-04-09"
source: "docs/audits/web-radar-2026-04-09.md"
ring: adopt
tags: [ci, coverage, dev-tools]
---

## Context

cargo-llvm-cov is already used locally. Adding it to CI with a fail-under threshold and LCOV upload enforces the 80% coverage minimum.

## Prompt

Add a `coverage` job to `.github/workflows/ci.yml` that runs `cargo llvm-cov --workspace --lcov --output-path lcov.info` and fails if function coverage drops below 80%. Upload LCOV artifact for PR comments.

## Acceptance Criteria

- [ ] CI job runs cargo-llvm-cov
- [ ] Job fails if coverage < 80%
- [ ] LCOV artifact uploaded
