---
id: BL-110
title: "Add cargo-semver-checks to CI pipeline"
scope: LOW
target: "direct edit"
status: implemented
created: "2026-03-31"
source: "docs/audits/web-radar-2026-03-31.md"
ring: adopt
tags: [ci, tooling, semver]
---

## Context

cargo-semver-checks grew from 121 to 242 lints in 2025, detecting breaking API changes before release. The project uses conventional commits and has an active release pipeline.

## Prompt

Add `cargo-semver-checks` to the CI workflow (`.github/workflows/ci.yml`). Install it as a cargo subcommand and run `cargo semver-checks` as a step in the `validate` job, after `cargo clippy`. This is a LOW scope change — one step addition to the existing CI configuration. No code changes needed.

## Acceptance Criteria

- [ ] `cargo semver-checks` runs in CI validate job
- [ ] Breaking API changes are caught before merge
- [ ] CI passes with current codebase (no false positives)
