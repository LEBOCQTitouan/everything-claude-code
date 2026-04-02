---
id: BL-088
title: "ecc update — self-update from GitHub Releases + cargo xtask deploy for dev"
scope: HIGH
target: "/spec dev"
status: implemented
tags: [cli, deploy, update, developer-experience, ci-cd]
created: 2026-03-28
related: [BL-087]
---

# BL-088: ecc update — Dual-Mode Deploy (Prod + Dev)

## Problem

After making code changes, developers must run 3-4 manual steps (build, copy binaries, ecc install) to see changes reflected. In production, there's no self-update mechanism — users must manually download releases via `scripts/get-ecc.sh`.

## Proposed Solution

Two complementary entry points for two audiences:

### Dev Mode: `cargo xtask deploy`
- Builds `ecc` + `ecc-workflow` from source (release or debug)
- Installs binaries to `~/.cargo/bin/`
- Runs `ecc install` to sync hooks, agents, skills, commands, rules to `~/.claude/`
- Supports `--debug` flag for faster iteration (skip release optimizations)
- Supports `--dry-run` to preview actions
- Leverages BL-087 xtask crate infrastructure

### Prod Mode: `ecc update`
- Downloads pre-built binaries from GitHub Releases (same platform detection as `scripts/get-ecc.sh`)
- Replaces installed `ecc` + `ecc-workflow` binaries (self-replacement with temp file + swap)
- Runs `ecc install` to sync config
- `ecc update` — pulls latest release
- `ecc update --version 4.3.0` — pins to specific version
- Reports: old version → new version, files synced

### CI/CD: GitHub Actions release pipeline
- On tag push (v*), build cross-platform binaries (macOS arm64/x64, Linux x64)
- Upload to GitHub Releases with checksums
- `ecc update` fetches from these releases

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | xtask vs ecc subcommand? | Both — xtask for dev, ecc update for prod | User |
| 2 | Download source for prod? | GitHub Releases (existing get-ecc.sh pattern) | Recommended |
| 3 | Version pinning? | Latest by default, --version for pinning | Recommended |

## Dependencies

- BL-087 (cargo xtask deploy) — BL-088 extends BL-087 with the prod `ecc update` path
- CI/CD release pipeline must exist for `ecc update` to have binaries to download

## Ready-to-Paste Prompt

```
/spec dev

Implement dual-mode deploy for ECC:

1. Dev: `cargo xtask deploy` — build from source, install binaries, sync config.
   Supports --debug and --dry-run. New xtask crate in workspace.

2. Prod: `ecc update` — download pre-built binaries from GitHub Releases,
   self-replace with temp+swap, run ecc install. Supports --version for pinning.
   Uses same platform detection as scripts/get-ecc.sh.

3. CI/CD: GitHub Actions release pipeline builds cross-platform binaries on
   tag push, uploads to GitHub Releases with checksums.

Dependencies: BL-087 (xtask crate). See BL-088 for full analysis.
```
