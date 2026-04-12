# Spec: BL-147 AGENTS.md / AAIF Standard Alignment Audit

## Problem Statement

The Linux Foundation's AAIF brings together MCP, Goose, and AGENTS.md under neutral governance. ECC's 70+ agents and 100+ skills use a similar but unaudited markdown format. Without formal alignment analysis, ECC risks diverging from an emerging industry standard, reducing interoperability and discoverability. The audit determines where ECC already aligns, where it diverges, and what the alignment stance should be.

## Research Summary

- AGENTS.md is a "README for agents" — simple markdown with optional YAML metadata for project-level AI guidance (setup, testing, coding style)
- AAIF (Dec 2025) unites MCP (protocol), AGENTS.md (documentation), Goose (execution) as complementary layers
- AGENTS.md core fields: name, description, tools, model — all already present in ECC agent frontmatter
- 60,000+ projects adopted AGENTS.md by end 2025; MCP has 10,000+ published servers
- AGENTS.md is single-file project-level; ECC's agents/ is per-agent multi-file — different abstraction levels
- Radical simplicity philosophy: no custom syntax, tool-agnostic, plain markdown

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Additive alignment stance | Core fields compatible, extensions non-conflicting, model alias resolved by hook | Yes |
| 2 | Date-pin AGENTS.md to 2025-12 AAIF launch | Spec evolving; stable reference needed | No |
| 3 | ECC-only comparison | Competitors covered in BL-140 | No |
| 4 | Extensions framing | ECC fields are additive, not conflicting | No |
| 5 | Include validation code delta appendix | Light estimate for future alignment | No |
| 6 | Include filesystem layout analysis | Discovery paths are interop concern | No |

## User Stories

### US-001: Gap Analysis Document
**As a** developer maintaining ECC, **I want** a structured gap analysis comparing ECC agent/skill format to AGENTS.md, **so that** alignment decisions are evidence-based.

#### Acceptance Criteria
- AC-001.1: Given the document, when inspected, then it contains a markdown table with columns: ECC Field, AAIF Equivalent, Status (aligned/extension/gap), Notes — covering all 9 ECC agent frontmatter fields (name, description, model, tools, effort, skills, memory, tracking, patterns) and all 3 skill fields (name, description, origin)
- AC-001.2: Given the document, then it has a "## Filesystem Layout" section comparing ECC discovery paths (agents/*.md, skills/name/SKILL.md, commands/*.md) against AGENTS.md single-file convention
- AC-001.3: Given the mapping table, then fields with Status="extension" include a Notes cell stating "additive: no AAIF equivalent, no naming conflict, safe to keep"
- AC-001.4: Given the document, then it has a "## Validation Code Delta" appendix listing affected Rust files (validate/agents.rs, validate/skills.rs, config/validate.rs) with 1-sentence description of what would change per file if full alignment were pursued
- AC-001.5: Given the document, when persisted, then it is at docs/research/aaif-alignment-gap-analysis.md
- AC-001.6: Given the document, then the header includes a permalink URL to the AGENTS.md spec snapshot (github.com/agentsmd/agents.md at a specific commit or tag from 2025-12)
- AC-001.7: Given the mapping table and the prose sections, then every field listed in the table is referenced in at least one prose section (cross-consistency)

#### Dependencies
- Depends on: none

### US-002: ADR on Alignment Stance
**As a** developer, **I want** an ADR documenting the decision to adopt additive alignment, **so that** the rationale is preserved.

#### Acceptance Criteria
- AC-002.1: Given the ADR, when inspected, then it follows Status/Context/Decision/Consequences format
- AC-002.2: Given the ADR, when inspected, then the Decision section picks "additive alignment"
- AC-002.3: Given the ADR, when inspected, then Alternatives Considered includes "full conformance" and "ignore AAIF"
- AC-002.4: Given the ADR, when persisted, then it uses the next available ADR number (0062)

#### Dependencies
- Depends on: US-001

### US-003: Documentation Updates
**As a** developer, **I want** CHANGELOG and CLAUDE.md updated, **so that** the audit is discoverable.

#### Acceptance Criteria
- AC-003.1: Given CHANGELOG.md, when inspected, then BL-147 entry exists under Unreleased
- AC-003.2: Given CLAUDE.md, when inspected, then glossary contains "additive alignment" definition
- AC-003.3: Given BL-147 backlog item, when inspected, then status is "implemented"

#### Dependencies
- Depends on: US-001, US-002

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| docs/research/aaif-alignment-gap-analysis.md | Docs | Create |
| docs/adr/0062-aaif-alignment-stance.md | Docs | Create |
| CLAUDE.md | Docs | Modify — glossary entry |
| CHANGELOG.md | Docs | Modify — BL-147 entry |

## Constraints

- Pure documentation — NO Rust code changes
- NO agent/skill file modifications (follow-up BL)
- AGENTS.md reference pinned to 2025-12 snapshot

## Non-Requirements

- Modifying any existing agent or skill file
- Runtime behavioral changes
- Giving up Claude-specific capabilities
- Competitor comparison (BL-140)
- Actual implementation of alignment changes

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | None | Pure documentation |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Gap analysis | docs/research/ | aaif-alignment-gap-analysis.md | Create |
| ADR | docs/adr/ | 0062-aaif-alignment-stance.md | Create |
| Glossary | CLAUDE.md | Glossary line | Add "additive alignment" |
| Entry | CHANGELOG.md | Unreleased | Add BL-147 entry |

## Open Questions

None — all resolved during grill-me interview.
