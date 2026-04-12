---
id: BL-135
title: "Add cargo-llvm-cov coverage gate to CI"
scope: LOW
target: "direct edit"
status: implemented
created: "2026-04-09"
implemented: "2026-04-12"
source: "docs/audits/web-radar-2026-04-09.md"
ring: adopt
tags: [ci, coverage, dev-tools]
---

## Context

cargo-llvm-cov is already used locally. Adding it to CI with a fail-under threshold and LCOV upload enforces the 80% coverage minimum.

## Prompt

Add a `coverage` job to `.github/workflows/ci.yml` that runs `cargo llvm-cov --workspace --lcov --output-path lcov.info` and fails if function coverage drops below 80%. Upload LCOV artifact for PR comments.

## Acceptance Criteria

- [x] CI job runs cargo-llvm-cov
- [x] Job fails if coverage < 80%
- [x] LCOV artifact uploaded

## Implementation Reference

`.github/workflows/ci.yml:165-213` — `coverage` job runs `cargo llvm-cov --workspace --exclude xtask --exclude ecc-test-support --lcov --output-path lcov.info --fail-under-functions 80` and uploads `lcov.info` as `coverage-report` artifact.
