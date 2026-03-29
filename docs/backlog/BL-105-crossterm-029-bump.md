---
id: BL-105
title: "Bump crossterm 0.28 → 0.29.0"
scope: LOW
target: "direct edit"
status: open
tags: [dependencies, crossterm, terminal]
created: 2026-03-29
related: []
source: "docs/audits/web-radar-2026-03-29-r2.md"
---

# BL-105: Bump crossterm to 0.29.0

## Problem

crossterm 0.29.0 was released 2025-04-05. ECC uses 0.28. No breaking changes between versions.

## Proposed Solution

Update crossterm version in Cargo.toml workspace dependencies from 0.28 to 0.29 and verify all tests pass.

## Ready-to-Paste Prompt

```
Bump crossterm from 0.28 to 0.29 in Cargo.toml workspace dependencies.
Run cargo test to verify no regressions. No breaking changes expected.
Source: web-radar-2026-03-29-r2.md
```
