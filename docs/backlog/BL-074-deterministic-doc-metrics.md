---
id: BL-074
title: Deterministic doc metrics — staleness detection, coverage calculation, severity counting
status: implemented
scope: LOW
target: /spec dev
created: 2026-03-26
tags: [deterministic, documentation, metrics, rust-cli]
related: [BL-056]
---

# BL-074: Deterministic Documentation Metrics

## Problem

Doc-reporter and audit-full agents perform mechanical calculations via LLM:
1. **Staleness detection** — compare git log timestamps between code and docs
2. **Coverage calculation** — `documented / total * 100`
3. **Severity counting** — count CRITICAL/HIGH/MEDIUM/LOW findings across domains

These are arithmetic and git operations — zero LLM judgment needed.

## Proposed Solution

### `ecc analyze doc-staleness [--threshold-days 30]`
- For each doc file, get last modified date via `git log -1 --format=%ai`
- For related code files, get last modified date
- Flag docs where code changed >N days after doc
- Output stale docs sorted by staleness

### `ecc analyze doc-coverage`
- Count public functions/types in Rust crates (pub items in .rs files)
- Count documented items (items with `///` doc comments)
- Calculate per-crate and overall coverage percentage

### `ecc analyze audit-summary <audit-report-path>`
- Parse markdown tables from audit report
- Count rows by severity column
- Aggregate per-domain and total
- Output summary statistics

## Impact

- **Speed**: < 500ms for full doc analysis vs 30-60s LLM agent
- **Accuracy**: Exact counts, not LLM approximations
- **CI-ready**: Can gate PRs on doc coverage thresholds
