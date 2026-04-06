# Spec: sccache + mold Build Acceleration

## Problem Statement

Rust compilation times grow with crate count. ECC has 9 crates; clean builds and test compilation can be slow. The 2026-03-29 web radar identified sccache (11-14% speedup on test builds) and mold linker as recommended build acceleration tools.

## Research Summary

- sccache: Mozilla's compilation caching tool, 11-14% speedup on repeated builds
- mold: Fast linker for Linux, significantly faster than default ld
- Cranelift: Alternative codegen backend, 30% faster compilation but trades runtime performance
- All are well-established, production-ready tools

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Per-user sccache config, not project config.toml | Avoids breaking builds for users without sccache | No |
| 2 | mold via target-specific config.toml sections | Linux-only, uses [target.x86_64-unknown-linux-gnu] so macOS unaffected | No |
| 3 | Cranelift commented out | 30% faster compile but slower runtime; document trade-off, let users opt in | No |
| 4 | Dev-only, no CI changes | CI has its own caching strategy | No |

## User Stories

### US-001: Build Acceleration Setup

**As a** developer, **I want** documented build acceleration tools, **so that** I can reduce Rust compilation times during development.

#### Acceptance Criteria

- AC-001.1: Given docs/getting-started.md, when a developer reads it, then it contains a "## Build Acceleration" section with sccache setup instructions
- AC-001.2: Given the sccache section, when the developer follows it, then it includes: install command, how to set RUSTC_WRAPPER, and expected speedup (11-14%)
- AC-001.3: Given .cargo/config.toml, when mold linker config is added, then it uses target-specific sections ([target.x86_64-unknown-linux-gnu] and [target.aarch64-unknown-linux-gnu]) so macOS builds are unaffected
- AC-001.4: Given the getting-started.md, when Cranelift is documented, then it is shown as commented-out config with a note explaining the trade-off (30% faster compile, slower runtime)
- AC-001.5: Given cargo build is run, when .cargo/config.toml has mold config, then it succeeds on macOS (mold sections ignored because target doesn't match)
- AC-001.6: Given CLAUDE.md Running Tests section, when updated, then it includes a note about optional sccache with a pointer to getting-started.md

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change Type |
|--------|-------|-------------|
| `.cargo/config.toml` | Config | Add mold linker target-specific sections |
| `docs/getting-started.md` | Docs | Add Build Acceleration section |
| `CLAUDE.md` | Docs | Add sccache note to Running Tests |

## Constraints

- .cargo/config.toml must not break macOS builds
- sccache must be optional (per-user config)
- No CI pipeline changes
- Cranelift must be commented out by default

## Non-Requirements

- CI pipeline changes
- Mandatory sccache installation
- Cranelift as default backend
- cargo xtask deploy auto-install of sccache
- Remote sccache cache sharing

## E2E Boundaries Affected

None -- config and documentation changes only.

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New section | docs/getting-started.md | Build Acceleration | Add setup instructions |
| Note | CLAUDE.md | Running Tests | Add sccache reference |
| Changelog | CHANGELOG.md | Add entry | all |

## Rollback Plan

1. Remove mold sections from .cargo/config.toml
2. Remove Build Acceleration section from getting-started.md
3. Remove sccache note from CLAUDE.md

## Open Questions

None -- all questions resolved during grill-me interview.
