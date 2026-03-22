# Spec: Create design-an-interface skill + agent (BL-014)

## Problem Statement

When designing module interfaces or port contracts, developers typically go with the first design that comes to mind. John Ousterhout's "Design It Twice" principle (from *A Philosophy of Software Design*) shows that comparing multiple radically different designs side-by-side produces better interfaces. Currently, ECC has no structured way to explore interface alternatives in parallel — developers must manually ideate and compare. A dedicated skill+agent pair would automate this exploration by spawning parallel sub-agents with radically different constraints, enforce radical divergence via a review step, and persist the comparison for team review.

## Research Summary

- **"Design It Twice"** (John Ousterhout, *A Philosophy of Software Design* Ch.11): Core inspiration — never go with the first design; compare at least two alternatives side-by-side. The second design often proves superior. Final design may combine ideas from both.
- **Claude Code parallel sub-agent patterns**: Well-established via Task tool. Heterogeneous model routing (opus orchestrates, sonnet executes). Map-reduce patterns used for competing hypotheses in code review.
- **Multi-perspective review**: Anthropic's own engineering uses sub-agents to independently check different aspects, with the orchestrator synthesizing results.
- **ECC skill/agent separation**: Skills are passive knowledge docs (<500 words, no tools/model). Agents handle behavior (spawning, orchestration). Must be split per ECC conventions.
- **API-first design methodology**: Modern best practice is designing APIs before implementation, enabling consumers and producers to collaborate on definitions. This skill formalizes that practice.
- **Divergence enforcement**: Prompt-only constraints are soft. An explicit review step where the orchestrator checks for convergence is the practical enforcement mechanism.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Split into skill (methodology) + agent (orchestration) | Skills are passive knowledge; agents handle behavior. Matches ECC conventions. | No |
| 2 | Output to `docs/designs/{module}-interface-{date}.md` | New directory for standalone design explorations, distinct from spec-pipeline artifacts | Yes |
| 3 | Fixed 4 constraints + optional user constraint | Balances reusability with simplicity | No |
| 4 | Agent-chosen paradigm for constraint 4 | Maximizes radical difference anti-pattern | No |
| 5 | Auto-detect project language from codebase | Scan for Cargo.toml, package.json, go.mod; fallback to pseudo-code | No |
| 6 | Standalone + /design integration | Triggered conversationally or invoked within /design Phase 1 | No |
| 7 | Divergence review step after sub-agents complete | Orchestrator checks for convergence; re-spawns if too similar | No |

## User Stories

### US-001: Skill File with Methodology & Triggers

**As a** developer, **I want** a `design-an-interface` skill that documents the interface exploration methodology and trigger phrases, **so that** I can activate it conversationally.

#### Acceptance Criteria

- AC-001.1: Given `skills/design-an-interface/SKILL.md` exists, when `ecc validate skills` runs, then validation passes with no errors.
- AC-001.2: Given the skill content, when word count is checked, then it is under 500 words.
- AC-001.3: Given the skill frontmatter, when inspected, then it contains `name: design-an-interface`, `description`, and `origin: ECC`.
- AC-001.4: Given the skill content, when read, then trigger phrases are documented: "design an interface", "design it twice", "explore interface options", "compare API shapes", "what should the port look like".
- AC-001.5: Given the skill content, when read, then the 4 default constraints are described: (1) minimize method count (1-3 max), (2) maximize flexibility, (3) optimize for most common case, (4) radically different paradigm chosen by agent. The 5 evaluation dimensions are described: interface simplicity, general-purpose vs specialized, implementation efficiency, depth, ease of correct use vs ease of misuse.
- AC-001.6: Given the skill content, when read, then the anti-patterns section includes: no similar designs, no skipping comparison, no implementation.
- AC-001.7: Given the skill content, when read, then it references the `interface-designer` agent for orchestration.

#### Dependencies

- Depends on: none

### US-002: Agent Orchestration with Parallel Sub-Agents

**As a** developer, **I want** an `interface-designer` agent that spawns 3+ parallel sub-agents with radically different constraints, **so that** I get genuinely diverse interface options.

#### Acceptance Criteria

- AC-002.1: Given `agents/interface-designer.md` exists, when inspected, then frontmatter has: `name: interface-designer`, `description`, `model: opus`, `tools: [Read, Grep, Glob, Agent, Write, TodoWrite, TodoRead, AskUserQuestion]`, `skills: [design-an-interface]`.
- AC-002.2: Given a target module/port, when the agent executes, then it spawns at least 3 Tasks referencing the `architect-module` agent in parallel.
- AC-002.3: Given the 4 sub-agents, when they execute, then each has a unique constraint: (1) minimize method count (1-3 max), (2) maximize flexibility — support many use cases, (3) optimize for the most common case, (4) radically different paradigm chosen by the agent for maximum divergence.
- AC-002.4: Given a user-supplied 5th constraint, when provided, then a 5th sub-agent is spawned with that constraint.
- AC-002.5: Given each sub-agent completes, when output is collected, then it includes: interface signature (in detected language), usage example, what it hides internally, and tradeoffs.
- AC-002.6: Given the agent starts, when it detects the project language, then it checks for Cargo.toml (→ Rust), package.json (→ TypeScript), go.mod (→ Go), with fallback to pseudo-code.
- AC-002.7: Given no target module/port is specified, when the agent starts, then it prompts the user to specify one.
- AC-002.8: Given the agent frontmatter, when inspected, then tools include: `[Read, Grep, Glob, Agent, Write, TodoWrite, TodoRead, AskUserQuestion]`.
- AC-002.9: Given the agent has 5+ workflow steps, when it executes, then it uses TodoWrite for progress tracking with graceful degradation ("If TodoWrite is unavailable, proceed without tracking").
- AC-002.10: Given multiple language markers are detected (e.g., both Cargo.toml and package.json), when the agent starts, then it asks the user which language to use for interface signatures.

#### Dependencies

- Depends on: US-001

### US-003: Divergence Review

**As a** developer, **I want** the agent to check that sub-agent designs are genuinely different, **so that** I don't get superficial variations.

#### Acceptance Criteria

- AC-003.1: Given all sub-agents complete, when the orchestrator reviews designs, then it checks for convergence defined as: two designs share the same structural pattern AND >50% method name overlap.
- AC-003.2: Given designs are too similar (convergence detected), when the divergence check fails, then the converging agent is re-spawned with an additional constraint: "Your design MUST use a fundamentally different structural pattern than Design X" (max 1 retry per agent).
- AC-003.3: Given a sub-agent fails or times out, when the comparison phase begins, then it proceeds with available designs and notes the gap.
- AC-003.4: Given a retried sub-agent still produces a converging design, when the retry completes, then the agent proceeds with available distinct designs (minimum 2 required to continue).

#### Dependencies

- Depends on: US-002

### US-004: Structured Comparison Matrix

**As a** developer, **I want** all designs compared on explicit dimensions, **so that** I can make an informed decision based on concrete tradeoffs.

#### Acceptance Criteria

- AC-004.1: Given all designs are available, when comparison runs, then designs are compared on these 5 dimensions: (1) interface simplicity, (2) general-purpose vs specialized, (3) implementation efficiency, (4) depth (small interface hiding significant complexity = good), (5) ease of correct use vs ease of misuse.
- AC-004.2: Given the comparison output, when presented, then it uses a structured table/matrix format.
- AC-004.3: Given the anti-pattern rule, when comparison would be skipped, then the agent blocks progression and requires the comparison step.

#### Dependencies

- Depends on: US-003

### US-005: User Synthesis & Design Selection

**As a** developer, **I want** to select a primary design and optionally incorporate elements from others, **so that** the final output reflects my actual needs.

#### Acceptance Criteria

- AC-005.1: Given the comparison is complete, when synthesis begins, then the agent asks via AskUserQuestion which design fits the user's primary use case.
- AC-005.2: Given the user selects a primary design, when prompted, then the agent asks whether elements from other designs should be incorporated.
- AC-005.3: Given AskUserQuestion is unavailable, when synthesis begins, then the agent presents all designs inline and asks conversationally (graceful degradation).
- AC-005.4: Given the user wants none of the designs, when selected, then the agent suggests re-running with different constraints or different module decomposition.

#### Dependencies

- Depends on: US-004

### US-006: Persisted Output Document

**As a** developer, **I want** the interface design output saved to `docs/designs/`, **so that** it's preserved for future reference and team discussion.

#### Acceptance Criteria

- AC-006.1: Given synthesis is complete, when output is written, then it is saved to `docs/designs/{module}-interface-{date}.md` where module is kebab-case and date is YYYY-MM-DD.
- AC-006.2: Given the output document, when read, then it contains: all individual designs (signature, usage example, internals hidden, tradeoffs), comparison matrix, user selection, synthesis rationale.
- AC-006.3: Given `docs/designs/` does not exist, when the agent writes output, then it creates the directory.
- AC-006.4: Given a file already exists at the target path, when writing, then the agent appends a numeric suffix in format `-N` before the extension (e.g., `{module}-interface-{date}-1.md`).

#### Dependencies

- Depends on: US-005

### US-007: /design Pipeline Integration

**As a** developer, **I want** the interface designer callable from within `/design`, **so that** I can explore interfaces during the design phase.

#### Acceptance Criteria

- AC-007.1: Given `/design` command file, when read, then Phase 1 mentions the `interface-designer` agent as an optional exploration tool.
- AC-007.2: Given the agent is invoked from `/design`, when output is written, then the output path follows the spec directory convention (`docs/specs/{slug}/`) instead of `docs/designs/`.

#### Dependencies

- Depends on: US-002

### US-008: Documentation & Discoverability

**As a** developer, **I want** the skill and agent documented, **so that** I can find them when I need interface exploration.

#### Acceptance Criteria

- AC-008.1: Given the glossary, when read, then it includes an "Interface Designer" term definition.
- AC-008.2: Given CHANGELOG.md, when read, then it includes a BL-014 feature entry.
- AC-008.3: Given `docs/adr/`, when read, then an ADR for the `docs/designs/` output convention exists.

#### Dependencies

- Depends on: US-001, US-002

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `skills/design-an-interface/SKILL.md` | Content (skill) | New file — methodology docs |
| `agents/interface-designer.md` | Content (agent) | New file — orchestration |
| `commands/design.md` | Driving adapter (command) | Modify — mention interface-designer as optional |
| `docs/domain/glossary.md` | Documentation | Modify — add Interface Designer term |
| `CHANGELOG.md` | Documentation | Modify — add BL-014 entry |
| `docs/adr/` | Documentation | New ADR — docs/designs/ convention |

No Rust crate changes. No hook changes. No state schema changes.

## Constraints

- Skill must be under 500 words (ECC convention)
- Agent frontmatter must have: name, description, model, tools, skills
- Agent must include TodoWrite with graceful degradation (ECC convention for 4+ step workflows)
- No implementation code — design exploration only
- Graceful degradation for AskUserQuestion
- Sub-agents reference architect-module agent with hexagonal boundary constraints
- Output directory is `docs/designs/` (standalone) or `docs/specs/{slug}/` (from /design pipeline)

## Non-Requirements

- Implementing the designed interfaces — this is purely about interface shape
- Supporting more than 5 sub-agents per run
- Auto-running after /spec completes
- Version control of design iterations (just the final output)
- Runtime validation of sub-agent output format
- Modifying the phase-gate hook for `docs/designs/`

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | — | No E2E boundaries crossed — pure content-layer addition |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Domain term | domain | docs/domain/glossary.md | Add Interface Designer term |
| Feature | project | CHANGELOG.md | Add BL-014 entry |
| Convention | architecture | docs/adr/ | ADR for docs/designs/ directory convention |

## Open Questions

None — all resolved during grill-me interview and adversarial review.
