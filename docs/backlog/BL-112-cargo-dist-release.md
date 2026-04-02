---
id: BL-112
title: "Evaluate cargo-dist to replace custom release.yml"
scope: MEDIUM
target: "/spec-refactor"
status: open
created: "2026-03-31"
source: "docs/audits/web-radar-2026-03-31.md"
ring: adopt
tags: [ci, release, tooling]
---

## Context

cargo-dist provides declarative plan/build/publish/announce stages for binary distribution. The project currently has a hand-rolled release.yml with cross-compilation matrix, tarball packaging, checksums, and gh release create. cargo-dist could simplify and standardize this.

## Prompt

Evaluate replacing the custom `.github/workflows/release.yml` with cargo-dist. Compare: (1) current release.yml capabilities (5-target cross-compile, cosign signing, tarball+checksum), (2) cargo-dist equivalents, (3) migration effort. If viable, refactor the release pipeline to use cargo-dist while preserving cosign signing and all 5 compilation targets (aarch64-apple-darwin, x86_64-apple-darwin, x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu, x86_64-pc-windows-msvc).

## Acceptance Criteria

- [ ] Comparison document of current vs cargo-dist capabilities
- [ ] Decision: adopt cargo-dist or keep custom release.yml
- [ ] If adopted: all 5 targets build successfully
- [ ] If adopted: cosign signing preserved
- [ ] If adopted: checksum generation preserved
