# Spec: Web-Based Upgrade Audit Command (/audit-web)

## Problem Statement

ECC has no mechanism to systematically scan the web for potential improvements to the current project. Upgrade discovery is ad-hoc — dependent on the user manually finding blog posts, conference talks, or new crates. There's no structured process for evaluating what the ecosystem offers against what the project currently uses, and no standardized output format for tracking upgrade opportunities over time.

## Research Summary

- ThoughtWorks' Build Your Own Radar, Zalando's tech-radar, and Qiwi's automated tech-radar generator accept JSON/CSV input and support 4 quadrants (Techniques, Tools, Platforms, Languages & Frameworks) with 4 rings (Adopt, Trial, Assess, Hold)
- GPT Researcher demonstrates the planner/executor/publisher pattern for multi-agent research: planner generates questions, executors search in parallel, publisher aggregates into structured reports
- Source triangulation (3+ independent sources per finding) is essential for reliability — single-source research agents are fragile
- Noise reduction is the key challenge: group semantically related findings, filter low-scoring items, merge duplicates across agents
- Dependency audit automation requires strong test suites as a prerequisite for trustworthy recommendations
- RAG (Retrieval-Augmented Generation) with web search is the standard mitigation for hallucination in audit recommendations
- Pitfalls: no auto-upgrades without CI gates, radar entries without ownership decay into stale lists, source attribution is mandatory

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Independent from /audit-full | External opportunity scanning vs internal code health are fundamentally different concerns; combining adds latency and breaks cross-correlation model | Yes |
| 2 | 4+1 Technology Radar quadrants | 4 standard ThoughtWorks quadrants for compatibility + 1 custom "Feature Opportunities" for competitor/user-request findings | Yes |
| 3 | Dedicated orchestrator agent | Follows /audit-full pattern with web-scout orchestrator agent for better separation of concerns | No |
| 4 | Cost consent gate after Phase 1 | Prevents surprise token burn from 8+ parallel web searches; user approves scope before Phase 2 | No |
| 5 | Polyglot from v1 | ECC supports Go, Python, TS, Java, Kotlin, C#, Shell projects; limiting to Rust-only would reduce value | No |
| 6 | Comma-separated --focus values | `--focus=deps,tools` runs only relevant agent groups; maps focus values to agent subsets | No |
| 7 | Reusable web-research skill | Extract search strategy into `skills/web-research-strategy/SKILL.md` for reuse by /spec-dev and future commands | No |
| 8 | Auto-backlog deferred to v2 | Findings reported but not auto-converted to BL-NNN entries; users run /backlog manually | No |

## User Stories

### US-001: /audit-web Command Shell

**As a** developer using ECC, **I want** a `/audit-web` slash command that orchestrates a 4-phase pipeline, **so that** I can run a single command to get a comprehensive web-based upgrade audit.

#### Acceptance Criteria

- AC-001.1: Given `commands/audit-web.md` exists, when I invoke `/audit-web`, then it executes a 4-phase pipeline (INVENTORY, LANDSCAPE SCAN, EVALUATE, SYNTHESIZE) in sequence
- AC-001.2: Given I provide `--focus=deps,tools`, when the command runs, then only dep-scanner and tool-scout agents are launched in Phase 2
- AC-001.3: Given I provide no arguments, when the command runs, then all 8 scope dimensions are scanned
- AC-001.4: Given the command file, when validated by `ecc validate`, then it passes with valid frontmatter (description, allowed-tools), mandatory workflow banner, and narrative convention reference
- AC-001.5: Given the command runs, when each phase completes, then the user sees narrative progress updates following `skills/narrative-conventions/SKILL.md`

#### Dependencies

- None

### US-002: Phase 1 — INVENTORY (Polyglot Codebase Detection)

**As a** developer, **I want** the audit to automatically detect my project's dependencies, patterns, tools, and domain context, **so that** Phase 2 search queries are grounded in what the project actually uses.

#### Acceptance Criteria

- AC-002.1: Given a Rust project with `Cargo.toml`, when Phase 1 runs, then it extracts all direct dependencies with their current versions
- AC-002.2: Given a Node.js project with `package.json`, when Phase 1 runs, then it extracts dependencies from `dependencies` and `devDependencies`
- AC-002.3: Given a Python project with `pyproject.toml` or `requirements.txt`, when Phase 1 runs, then it extracts dependencies
- AC-002.4: Given a Go project with `go.mod`, when Phase 1 runs, then it extracts module dependencies
- AC-002.5: Given the project has `CLAUDE.md` and `docs/ARCHITECTURE.md`, when Phase 1 runs, then it extracts architectural patterns (hexagonal, DDD, TDD, etc.)
- AC-002.6: Given `--focus=deps`, when Phase 1 runs, then it limits inventory output to dependency information only
- AC-002.7: Given Phase 1 completes, when the cost consent gate is displayed, then it shows: N dependencies, M patterns, K tool areas, estimated ~N*3 web searches, and asks for user confirmation before Phase 2
- AC-002.8: Given the user declines at the cost gate, when the command handles the response, then it exits gracefully with "Audit cancelled by user"
- AC-002.9: Given no recognized dependency manifest is found (no Cargo.toml, package.json, pyproject.toml, requirements.txt, or go.mod), when Phase 1 runs, then it reports "No dependency manifests detected" and skips dependency scanning
- AC-002.10: Given `CLAUDE.md` or `docs/ARCHITECTURE.md` is missing, when Phase 1 runs, then it skips pattern extraction and notes "architectural patterns: not detected"
- AC-002.11: Given more than 100 dependencies are detected, when the cost gate displays, then it groups dependencies by category and caps search queries at 50 per category to prevent excessive token burn

#### Dependencies

- None

### US-003: Phase 2 — LANDSCAPE SCAN (Parallel Web Search Agents)

**As a** developer, **I want** 8+ parallel search agents to scan the web for upgrades across different dimensions, **so that** I get broad coverage of potential improvements.

#### Acceptance Criteria

- AC-003.1: Given Phase 1 inventory and user confirmation, when Phase 2 runs, then the web-scout orchestrator launches 8 parallel Task subagents (dep-scanner, arch-scout, tool-scout, ecosystem-scout, competitor-scout, user-request-miner, blog-miner, research-scout)
- AC-003.2: Given each subagent runs, when it finds a potential upgrade, then it validates with 3+ independent sources (source triangulation)
- AC-003.3: Given each subagent, when it formulates queries, then it uses the web-research-strategy skill: short keyword queries, year-anchored, pseudo-answer generation, query variation on retry
- AC-003.4: Given `--focus=deps` was specified, when Phase 2 runs, then only the dep-scanner agent is launched
- AC-003.5: Given WebSearch is unavailable, when a subagent tries to search, then it falls back to exa-web-search MCP; if both unavailable, reports "search skipped" without failing the pipeline
- AC-003.6: Given one or more subagents fail or time out, when Phase 2 continues, then it proceeds with available results and notes which agents failed
- AC-003.7: Given each subagent completes, when findings are returned, then only condensed findings (not raw search content) are passed to Phase 3
- AC-003.8: Given 8 parallel search agents, when launched, then the orchestrator enforces a concurrency cap of 4 simultaneous agents to avoid rate limiting; remaining agents queue and start as slots free up

#### Dependencies

- Depends on US-002

### US-004: Phase 3 — EVALUATE (Scoring and Radar Classification)

**As a** developer, **I want** each finding scored and classified into Technology Radar quadrants and rings, **so that** I can prioritize which upgrades to pursue.

#### Acceptance Criteria

- AC-004.1: Given Phase 2 findings, when Phase 3 runs, then each finding is scored on 3 dimensions: strategic fit (0-5), maturity (0-5), migration effort (0-5)
- AC-004.2: Given a scored finding, when classified, then it is assigned a radar quadrant (Techniques, Tools, Platforms, Languages & Frameworks, or Feature Opportunities) and ring (Adopt, Trial, Assess, Hold)
- AC-004.3: Given a finding with fit >= 4, maturity >= 4, and effort <= 2, when classified, then it is placed in the Adopt ring
- AC-004.4: Given a finding with maturity < 2, when classified, then it is placed in the Assess or Hold ring regardless of fit
- AC-004.5: Given duplicate findings from different agents, when evaluation completes, then duplicates are merged with combined source citations

#### Dependencies

- Depends on US-003

### US-005: Phase 4 — SYNTHESIZE (Technology Radar Report)

**As a** developer, **I want** a Technology Radar report persisted and a top-10 summary displayed, **so that** I have both a permanent reference and immediate actionable insights.

#### Acceptance Criteria

- AC-005.1: Given Phase 3 evaluated findings, when Phase 4 runs, then a Markdown report is generated at `docs/audits/web-radar-YYYY-MM-DD.md`. If `docs/audits/` does not exist, Phase 4 creates the directory before writing.
- AC-005.2: Given the report, when written, then it follows Technology Radar format grouped by quadrant, then by ring, with per-finding: title, ring, fit/maturity/effort scores, rationale, and 3+ source citations
- AC-005.3: Given the report, when the terminal summary is displayed, then it shows top 10 findings ranked by impact/effort ratio
- AC-005.4: Given the report file already exists for today's date, when Phase 4 runs, then it appends a revision suffix (e.g., `web-radar-2026-03-28-r2.md`)
- AC-005.5: Given fewer than 10 findings, when the summary is displayed, then all findings are shown
- AC-005.6: Given no findings at all, when the report is generated, then it indicates "No upgrade opportunities identified — project is current"
- AC-005.7: Given the command completes, when the user sees the output, then it says "To act on findings, run `/backlog add` or `/spec` referencing this report."
- AC-005.8: Given the command, when it finishes, then it does NOT modify source code (read-only audit)

#### Dependencies

- Depends on US-004

### US-006: Web-Scout Orchestrator Agent

**As a** maintainer, **I want** a dedicated `web-scout` orchestrator agent, **so that** the Phase 2 parallel dispatch logic is encapsulated and reusable.

#### Acceptance Criteria

- AC-006.1: Given `agents/web-scout.md` exists, when it is validated by `ecc validate`, then it has valid frontmatter (name, description, tools, model)
- AC-006.2: Given the agent receives a tech stack context and scope, when it runs, then it launches up to 8 parallel Task subagents using the `web-radar-analyst` agent template
- AC-006.3: Given a `--focus` filter is provided, when the orchestrator runs, then it only launches agents matching the focus dimensions
- AC-006.4: Given each subagent returns findings, when the orchestrator aggregates, then it deduplicates and returns a unified findings list

#### Dependencies

- None

### US-007: Web-Radar-Analyst Search Agent

**As a** maintainer, **I want** a single reusable search agent that can be spawned with different category prompts, **so that** I don't need 8 separate agent files.

#### Acceptance Criteria

- AC-007.1: Given `agents/web-radar-analyst.md` exists, when validated, then it has valid frontmatter with tools: [WebSearch] and model: haiku (cost-efficient)
- AC-007.2: Given the agent receives a category, tech stack context, and query templates, when it runs, then it executes 2-3 web searches per category with source triangulation
- AC-007.3: Given search results, when the agent processes them, then it returns structured findings with: title, category, source URLs, summary, relevance score

#### Dependencies

- None

### US-008: Web Research Strategy Skill

**As a** developer building commands that need web research, **I want** a reusable `web-research-strategy` skill, **so that** search best practices are standardized across commands.

#### Acceptance Criteria

- AC-008.1: Given `skills/web-research-strategy/SKILL.md` exists, when validated, then it has valid frontmatter (name, description)
- AC-008.2: Given the skill content, when referenced by /audit-web and /spec-dev, then both commands use the same search strategy patterns
- AC-008.3: Given the skill, when it defines query templates, then they include: keyword queries, year-anchored, pseudo-answer generation, query variation on retry, source weighting (primary > secondary)

#### Dependencies

- None

### US-009: Documentation and Registration

**As a** developer browsing ECC documentation, **I want** `/audit-web` to appear in the command reference and CLAUDE.md, **so that** I can discover and understand the command.

#### Acceptance Criteria

- AC-009.1: Given the command exists, when I read `docs/commands-reference.md`, then `/audit-web` appears in the Audit Commands table
- AC-009.2: Given the command exists, when I read `CLAUDE.md`, then `/audit-web` is listed among the audit commands
- AC-009.3: Given BL-081 exists in the backlog, when the feature is implemented, then BL-081 is updated to `status: promoted`

#### Dependencies

- Depends on US-001

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `commands/audit-web.md` | Command (content) | Create — 4-phase pipeline command |
| `agents/web-scout.md` | Agent (content) | Create — orchestrator for parallel search dispatch |
| `agents/web-radar-analyst.md` | Agent (content) | Create — reusable search agent per category |
| `skills/web-research-strategy/SKILL.md` | Skill (content) | Create — reusable search strategy patterns |
| `docs/commands-reference.md` | Documentation | Update — add /audit-web row |
| `CLAUDE.md` | Documentation | Update — add to audit command list |
| `docs/backlog/BL-081-web-upgrade-audit.md` | Documentation | Update — status to promoted |
| `docs/adr/` | Documentation | Create 2 ADRs |

No Rust crate changes. No hexagonal boundary crossings.

## Constraints

- WebSearch or exa-web-search MCP must be available for Phase 2 — command must pre-flight check and warn if unavailable
- Haiku model for search agents to minimize cost
- Read-only audit — never modify source code or dependencies
- Source triangulation mandatory (3+ sources per finding)

## Non-Requirements

- Auto-backlog generation from findings (deferred to v2)
- Auto-upgrade or auto-PR creation
- Offline mode (web search is fundamental to this command)
- Performance benchmarking of search latency

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| WebSearch tool | New dependency | Command requires web search availability; graceful degradation if unavailable |
| docs/audits/ | New output | Generated report file; no existing reports affected |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New command | docs | docs/commands-reference.md | Add /audit-web row |
| New command | docs | CLAUDE.md | Add to audit command list |
| New ADR | docs | docs/adr/0020-audit-web-independent.md | Create |
| New ADR | docs | docs/adr/0021-technology-radar-format.md | Create |
| Backlog update | docs | docs/backlog/BL-081 | Status → promoted |
| CHANGELOG | docs | CHANGELOG.md | Add entry |

## Open Questions

None — all resolved during grill-me.
