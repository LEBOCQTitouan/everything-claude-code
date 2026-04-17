---
id: BL-111
title: "Enable GitHub Merge Queue for CI load reduction"
scope: LOW
target: "direct edit"
status: implemented
created: "2026-03-31"
source: "docs/audits/web-radar-2026-03-31.md"
ring: adopt
tags: [ci, github, workflow]
---

## Context

GitHub Merge Queue is GA — batches and orders PR merges, reducing redundant CI runs. The Rust infrastructure team has adopted this pattern.

## Prompt

Enable GitHub Merge Queue in repo settings and add the `merge_group` trigger event to `.github/workflows/ci.yml`. This is a LOW scope change — one trigger addition plus repo settings toggle.

## Acceptance Criteria

- [ ] `merge_group` event added to ci.yml triggers
- [ ] Merge Queue enabled in repository branch protection rules
- [ ] PRs are batched and tested together before merge
