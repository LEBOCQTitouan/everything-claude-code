# AAIF Alignment Gap Analysis

**Date**: 2026-04-12
**Reference**: AGENTS.md spec pinned to 2025-12 AAIF launch (https://github.com/agentsmd/agents.md)
**Scope**: ECC agent and skill frontmatter vs AAIF AGENTS.md standard

## Executive Summary

ECC's agent frontmatter is **additively aligned** with AAIF. The 4 core AAIF fields (`name`, `description`, `tools`, `model`) are already present in all ECC agents with compatible semantics. ECC-specific fields (`effort`, `skills`, `memory`, `tracking`, `patterns`) are additive extensions with no naming conflicts. The single semantic gap — `model` value format (short aliases vs full model IDs) — is resolved at the hook layer.

## Field-by-Field Mapping Table

### Agent Frontmatter

| ECC Field | AAIF Equivalent | Status | Notes |
|-----------|----------------|--------|-------|
| `name` | `name` | aligned | Identical semantics — human-readable agent identifier |
| `description` | `description` | aligned | Identical semantics — one-line purpose statement |
| `tools` | `tools` | aligned | String array of tool names. ECC uses Claude Code tool names; AAIF is tool-agnostic |
| `model` | `model` | gap | ECC uses short aliases (`opus`, `sonnet`, `haiku`) resolved by SubagentStart hook; AAIF expects full model IDs (`claude-sonnet-4-6`). The hook is the ACL — no agent file changes needed |
| `effort` | — | extension | additive: no AAIF equivalent, no naming conflict, safe to keep. Controls thinking budget tier (low/medium/high/max) |
| `skills` | — | extension | additive: no AAIF equivalent, no naming conflict, safe to keep. References composable skill files |
| `memory` | — | extension | additive: no AAIF equivalent, no naming conflict, safe to keep. Memory persistence type (e.g., "project") |
| `tracking` | — | extension | additive: no AAIF equivalent, no naming conflict, safe to keep. Progress tracking mode (e.g., "todowrite") |
| `patterns` | — | extension | additive: no AAIF equivalent, no naming conflict, safe to keep. Pattern category references |

### Skill Frontmatter

| ECC Field | AAIF Equivalent | Status | Notes |
|-----------|----------------|--------|-------|
| `name` | — | extension | additive: AAIF has no composable skill concept. ECC-proprietary |
| `description` | — | extension | additive: AAIF has no skill metadata. ECC-proprietary |
| `origin` | — | extension | additive: always "ECC". ECC-proprietary |

### Summary

- **Aligned fields**: 3 (name, description, tools)
- **Gap fields**: 1 (model — semantic format difference, resolved by hook)
- **Extension fields**: 8 (effort, skills, memory, tracking, patterns, skill name, skill description, skill origin)
- **Conflict fields**: 0

## Filesystem Layout

### AGENTS.md Convention
AGENTS.md is a **single file per project** — a "README for agents" placed at the repository root (e.g., `AGENTS.md` or `.agents/AGENTS.md`). It describes project-level conventions: setup commands, testing workflows, coding style, PR guidelines. It is **not** a per-agent definition file.

### ECC Convention
ECC uses a **multi-file per-agent** structure:
- Agents: `agents/<name>.md` — one file per agent (70+ files)
- Skills: `skills/<name>/SKILL.md` — one directory per skill (100+ directories)
- Commands: `commands/<name>.md` — one file per command (33 files)
- Hooks: `hooks/hooks.json` — single JSON configuration

### Divergence Assessment
These are **different abstraction levels**, not competing conventions:
- AGENTS.md = project-level guidance for AI agents (analogous to ECC's `CLAUDE.md`)
- ECC agents/ = per-agent behavioral definitions (no AAIF equivalent)

ECC's `CLAUDE.md` is the functional equivalent of AGENTS.md. The per-agent directory structure is a proprietary extension that AAIF does not address.

**Interoperability impact**: An AAIF-compliant tool looking for `AGENTS.md` would find `CLAUDE.md` serves the same purpose. An AAIF tool looking for per-agent definitions would find no standard to compare against — ECC's format is beyond AAIF's current scope.

## Gap Detail: Model Value Format

**ECC**: `model: sonnet` (short alias resolved by `SubagentStart` hook in `hooks/hooks.json`)
**AAIF**: `model: claude-sonnet-4-6` (full model identifier)

**Resolution**: The SubagentStart hook already performs alias expansion at runtime. This is the correct ACL boundary — the domain layer (agent files) stays clean with human-readable aliases, while the adapter layer (hook) translates to full model IDs for the API. No agent file changes are needed.

**Risk**: External AAIF-compliant tooling consuming raw ECC frontmatter will see short aliases, not full IDs. Callers must either use the ECC hook pipeline or apply the alias map from `crates/ecc-domain/src/config/validate.rs` (VALID_MODELS constant).

## Validation Code Delta

If full AAIF conformance were pursued in the future (not recommended), these Rust files would need changes:

| File | Current Role | Change Needed |
|------|-------------|---------------|
| `crates/ecc-domain/src/config/validate.rs` | Defines VALID_MODELS (`haiku`, `sonnet`, `opus`) | Add full model ID alternatives or alias map |
| `crates/ecc-app/src/validate/agents.rs` | Validates agent frontmatter fields | Accept both alias and full ID in `model` field |
| `crates/ecc-app/src/validate/skills.rs` | Validates skill frontmatter fields | No change needed — skills have no AAIF equivalent |
| `crates/ecc-app/src/validate/conventions.rs` | Cross-validates naming and tool references | Update model validation if alias format changes |

**Estimated effort**: ~50 lines of Rust across 2-3 files. Low risk, mechanical change. Not recommended — the hook ACL approach is cleaner.

## Recommendation

**Additive alignment**. ECC is already compatible with AAIF's 4 core fields. The 8 extension fields are non-conflicting. The model alias gap is resolved at the hook layer. No file changes required. Document this stance in ADR-0062.
