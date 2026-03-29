---
id: BL-099
title: "Migrate serde_yml to serde-yaml-ng — maintenance risk mitigation"
scope: MEDIUM
target: "/spec-refactor"
status: open
tags: [dependencies, yaml, serde, security, maintenance]
created: 2026-03-29
related: []
source: "docs/audits/web-radar-2026-03-29-r2.md"
---

# BL-099: Migrate serde_yml to serde-yaml-ng

## Problem

serde_yml 0.0.12 has documented quality concerns: community reports AI-generated code with unsound implementations and broken documentation. The crate is a fork of the archived serde-yaml, but the fork quality is questionable.

## Proposed Solution

Migrate from serde_yml to serde-yaml-ng (actively maintained fork) or serde-saphyr (direct YAML parsing). Both are drop-in replacements for serde-yaml API.

## Ready-to-Paste Prompt

```
/spec-refactor Migrate from serde_yml 0.0.12 to serde-yaml-ng across the workspace.

Motivation: serde_yml has documented quality concerns (AI-generated code, unsound
implementations). serde-yaml-ng is the community-recommended maintained fork.

Scope:
- Replace serde_yml dependency in Cargo.toml workspace deps
- Update all import paths (serde_yml:: → serde_yaml_ng:: or equivalent)
- Verify all YAML parsing tests pass with the new crate
- Check for any API differences between serde_yml and serde-yaml-ng

This should be a straightforward dependency swap with no behavioral changes.
Source: docs/audits/web-radar-2026-03-29-r2.md (Hold — serde_yml maintenance risk)
```
