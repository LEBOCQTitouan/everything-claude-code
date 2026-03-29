# Spec: Adversarial challenge phase for audit commands (BL-083)

## Problem Statement

Audit commands produce findings from a single perspective without independent verification. False positives persist, genuine issues may be under-weighted, and findings lack validation against current best practices. The spec/design pipeline already uses adversarial agents (spec-adversary, solution-adversary) with proven quality improvement — audit commands lack this pattern. Adding an interleaved adversarial challenge phase to all 10 `/audit-*` commands would catch false positives, validate findings against web best practices, and surface disagreements for user resolution.

## Research Summary

- Adversarial review pattern already proven in ECC spec/design pipeline (spec-adversary scores 7 dimensions)
- Clean context for adversary is critical — context pollution research shows same-agent second passes produce weaker challenges
- "Red team" pattern in security auditing: independent verification by a separate team is standard practice
- CodeRabbit uses multi-pass review with an independent synthesis step
- BL-036 (numeric quality scores) already establishes scoring patterns for adversary output quality

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | New `audit-challenger` agent at Sonnet tier | Checklist verification + web search, not deep adversarial reasoning. Per ADR-0030 three-tier policy. | No |
| 2 | Interleaved after each domain audit, not batched | BL-083 Q3: per-domain challenge catches issues in context before moving on | No |
| 3 | Always-on, no opt-in flag | BL-083 Q8: accept cost, never sample or skip | No |
| 4 | User resolves disagreements | BL-083 Q9: show both perspectives with recommendation, user makes final call | No |
| 5 | Retry once on low-quality output | BL-083 Q6: stricter prompt on retry; if still low quality, surface warning alongside content | No |
| 6 | Agent frontmatter: skills=["clean-craft"], memory=project | Per ECC adversary conventions (rules/ecc/development.md) | No |
| 7 | Agent tools: Read, Grep, Glob, Bash, WebSearch | Read-only analysis + web search. No Write or Edit. | No |
| 8 | Graceful degradation on agent failure | If adversary fails to spawn or errors, warn and proceed with unchallenged findings. Follows ECC pattern. | No |
| 9 | audit-full.md also needs modification | audit-full spawns agents directly, not via individual command files. Must add adversary dispatches there too. | No |

## User Stories

### US-001: Audit-challenger agent

**As a** developer running audits, **I want** a separate agent to challenge each audit's findings independently, **so that** false positives are caught and findings are validated against current best practices.

#### Acceptance Criteria

- AC-001.1: Given the audit-challenger agent file, when frontmatter is read, then model is `sonnet`, skills includes `clean-craft`, memory is `project`, tools do NOT include Write or Edit
- AC-001.2: Given audit-challenger runs after a domain audit, when it finds no issues to challenge, then it emits an explicit "clean bill of health" statement with rationale
- AC-001.3: Given audit-challenger output does not contain structured per-finding verdicts (each finding must have: finding ID, verdict {confirmed|refuted|amended}, and rationale), when the command detects missing structure, then it retries once with a stricter prompt; if second attempt still lacks structure, a "Low-quality adversary output" warning is surfaced alongside the raw content
- AC-001.4: Given audit and adversary disagree on a finding, when results are displayed, then both the original finding and the adversary's challenge are shown side by side with the adversary's recommendation, and the user is prompted for final decision
- AC-001.5: Given the audit-challenger agent fails to spawn or returns an error, when the command handles the failure, then it emits a warning "Adversary challenge skipped: <reason>" and proceeds with unchallenged findings (graceful degradation)

#### Dependencies

- Depends on: none

### US-002: Integrate adversary into all audit commands

**As a** developer, **I want** every `/audit-*` command to include an adversarial challenge phase after the analysis stage, **so that** all audits benefit from independent verification.

#### Acceptance Criteria

- AC-002.1: Given each of the 10 domain audit commands (audit-archi, audit-code, audit-convention, audit-doc, audit-errors, audit-evolution, audit-observability, audit-security, audit-test, audit-web), when the command definition is read, then it includes an adversary phase that launches the `audit-challenger` agent after the analysis phase completes
- AC-002.2: Given `/audit-full` delegates to the `audit-orchestrator` agent which spawns domain agents directly, when the orchestrator agent definition is read, then it includes adversary challenge dispatches after each domain agent completes
- AC-002.3: Given all modified audit command files, when `ecc validate commands` runs, then it passes with zero errors
- AC-002.4: Given the new audit-challenger agent file, when `ecc validate agents` runs, then it passes with zero errors
- AC-002.5: Given all modified files, when `ecc validate conventions` runs, then it passes with zero errors

#### Dependencies

- Depends on: US-001

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `agents/audit-challenger.md` | Config (new) | New adversary agent for audit challenge |
| `commands/audit-archi.md` | Config | Add adversary phase after analysis |
| `commands/audit-code.md` | Config | Add adversary phase after analysis |
| `commands/audit-convention.md` | Config | Add adversary phase after analysis |
| `commands/audit-doc.md` | Config | Add adversary phase after analysis |
| `commands/audit-errors.md` | Config | Add adversary phase after analysis |
| `commands/audit-evolution.md` | Config | Add adversary phase after analysis |
| `commands/audit-observability.md` | Config | Add adversary phase after analysis |
| `commands/audit-security.md` | Config | Add adversary phase after analysis |
| `commands/audit-test.md` | Config | Add adversary phase after analysis |
| `commands/audit-web.md` | Config | Add adversary phase after analysis |
| `commands/audit-full.md` | Config | Add adversary challenges after each domain agent dispatch |
| `agents/audit-orchestrator.md` | Config | Add adversary challenge dispatches after each domain agent in Phase 2 |
| `docs/commands-reference.md` | Docs | Update audit descriptions |
| `CHANGELOG.md` | Docs | Entry |

## Constraints

- Agent must NOT have Write or Edit tools (read-only analysis per ECC conventions)
- Agent must have `skills: ["clean-craft"]` and `memory: project` per adversary conventions
- Adversary runs in clean context (separate Task agent spawn, not same-agent second pass)
- No changes to domain-specific logic of any existing audit command
- No changes to /review or /verify commands
- Markdown-only changes — zero Rust code

## Non-Requirements

- Opt-in flag or `--no-adversary` bypass
- Sampling or partial-pass logic for cost reduction
- Modifying existing audit agent internals
- Changing /review or /verify commands
- Rust code changes

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | Markdown config only | No Rust code boundaries affected |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New agent | Docs | docs/commands-reference.md | Note adversary challenge phase in audit command descriptions |
| Changelog | Project | CHANGELOG.md | Add entry under ### Added |

## Open Questions

None — all resolved during BL-083 challenge log (10 questions) + grill-me interview (3 questions).
