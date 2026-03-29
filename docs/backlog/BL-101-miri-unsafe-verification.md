---
id: BL-101
title: "Miri unsafe code verification for ecc-flock"
scope: LOW
target: "direct edit"
status: open
tags: [tooling, safety, testing, unsafe]
created: 2026-03-29
related: []
source: "docs/audits/web-radar-2026-03-29-r2.md"
---

# BL-101: Miri Unsafe Code Verification

## Problem

ecc-flock uses libc and POSIX flock which involve unsafe boundaries. Miri can detect undefined behavior in unsafe code at test time.

## Proposed Solution

Add a Miri test target for ecc-flock crate. Run `cargo +nightly miri test -p ecc-flock` to verify no UB in the flock implementation.

## Ready-to-Paste Prompt

```
Add Miri verification for ecc-flock unsafe code:

1. Add a CI-optional Miri test step for ecc-flock: cargo +nightly miri test -p ecc-flock
2. Document in CLAUDE.md under "Running Tests"
3. Fix any UB that Miri detects

Only target ecc-flock — the rest of the codebase is pure safe Rust.
Source: web-radar-2026-03-29-r2.md
```
