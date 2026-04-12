# ADR-0062: AAIF Alignment Stance — Additive Alignment

## Status

Accepted

## Context

The Linux Foundation's Agentic AI Foundation (AAIF, Dec 2025) establishes AGENTS.md as the standard for project-level AI agent guidance, alongside MCP (protocol) and Goose (execution). ECC's 70+ agents and 100+ skills use markdown files with YAML frontmatter that predates this standard. BL-147 audited the gap between ECC's format and AAIF to determine alignment posture.

The gap analysis (docs/research/aaif-alignment-gap-analysis.md) found:
- 3 fields fully aligned: `name`, `description`, `tools`
- 1 semantic gap: `model` uses short aliases vs AAIF full IDs (resolved by SubagentStart hook)
- 8 extension fields with no AAIF equivalent and no naming conflicts
- 0 conflicting fields

Three alignment stances were evaluated.

## Decision

**Additive alignment.** The additive alignment stance means ECC adopts AAIF core fields as-is — no renames, no removals. ECC-specific extensions (`effort`, `skills`, `memory`, `tracking`, `patterns`) are treated as a published language layer on top of AAIF. No agent or skill files are modified.

The `model` semantic gap is resolved at the hook layer (SubagentStart), which already performs alias expansion. This is the correct ACL boundary — the domain stays clean, the adapter translates.

## Consequences

**Positive**
- Zero disruption: existing agents and skills require no edits
- Forward-compatible: if AAIF adds new fields, ECC can adopt them additively
- Hook layer remains the single translation point for runtime concerns
- ECC's `CLAUDE.md` serves the same functional role as AGENTS.md

**Negative**
- External AAIF tooling consuming raw ECC frontmatter sees short model aliases, not full IDs
- Skills remain ECC-proprietary — no portability to non-ECC AAIF runtimes
- Per-agent file structure (agents/*.md) has no AAIF equivalent — beyond standard scope

## Alternatives Considered

**Full conformance** — rename `model` values to full IDs, add `system_prompt` key, restructure to single AGENTS.md file. Rejected: requires touching every agent file, breaks `ecc validate agents` schema, adds noise with no runtime benefit since the hook already resolves aliases. Would also lose the per-agent modularity that enables parallel development.

**Ignore AAIF** — make no analysis, no documentation, no alignment claim. Rejected: ECC is a community-facing tool — alignment with the emerging standard improves interoperability and discoverability. The cost of documenting the stance is minimal.
