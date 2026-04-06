---
id: BL-100
title: "sccache + mold build acceleration for dev environment"
scope: LOW
target: "direct edit"
status: implemented
tags: [tooling, build, performance, dev-experience]
created: 2026-03-29
related: [BL-087]
source: "docs/audits/web-radar-2026-03-29-r2.md"
---

# BL-100: sccache + mold Build Acceleration

## Problem

Rust compilation times grow with crate count. ECC has 9 crates; incremental builds are fast but clean builds and test compilation can be slow.

## Proposed Solution

Add sccache (compilation caching, 11-14% speedup on test builds) and document mold linker setup for Linux. Optionally trial Cranelift backend for dev builds (30% faster compilation, trades runtime performance).

## Ready-to-Paste Prompt

```
Add sccache and mold linker configuration to the dev environment:

1. Add .cargo/config.toml with sccache as build.rustc-wrapper
2. Document mold linker setup for Linux in docs/getting-started.md
3. Add optional Cranelift backend configuration (commented out, with docs)
4. Update cargo xtask deploy to optionally install sccache

Keep changes to dev tooling only — no CI changes. Source: web-radar-2026-03-29-r2.md
```
