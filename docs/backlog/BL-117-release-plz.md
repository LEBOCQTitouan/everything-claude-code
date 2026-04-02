---
id: BL-117
title: "Evaluate release-plz for automated semver and changelog"
scope: MEDIUM
target: "/spec-dev"
status: open
created: "2026-03-31"
source: "docs/audits/web-radar-2026-03-31.md"
ring: trial
tags: [ci, release, automation]
---

## Context

release-plz automates semver bumping from Conventional Commits, opens changelog PRs, and integrates with cargo-semver-checks. Complements cargo-dist (release-plz handles versioning, cargo-dist handles binary packaging). The project already uses conventional commits.

## Prompt

Evaluate `release-plz` for automating version bumps and changelog generation. Compare against the current manual versioning in workspace Cargo.toml. Assess: (1) integration with conventional commit workflow, (2) workspace-aware version management across 9 crates, (3) compatibility with existing CD pipeline (auto-tag on main push), (4) changelog generation quality. If viable, add a GitHub Actions workflow for release-plz.

## Acceptance Criteria

- [ ] Evaluation document comparing current workflow vs release-plz
- [ ] Decision: adopt or defer
- [ ] If adopted: release-plz GitHub Action configured
- [ ] If adopted: changelog generation tested
