# Design: Web-Based Upgrade Audit Command (/audit-web) — BL-081

## Overview

Content-only implementation of a new `/audit-web` slash command with its orchestrator agent, search agent, reusable skill, two ADRs, and documentation updates. No Rust code. All deliverables are Markdown files with YAML frontmatter.

## File Changes

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `commands/audit-web.md` | CREATE | 4-phase pipeline command shell with frontmatter, mandatory workflow banner, narrative reference, arguments, TodoWrite tracking, and full phase descriptions | US-001, US-002, US-003, US-004, US-005 |
| 2 | `agents/web-scout.md` | CREATE | Orchestrator agent that receives inventory context and dispatches up to 8 parallel Task subagents with concurrency cap, focus filtering, and deduplication | US-006 |
| 3 | `agents/web-radar-analyst.md` | CREATE | Reusable search agent template spawned per category; haiku model, WebSearch tool, source triangulation, structured finding output | US-007 |
| 4 | `skills/web-research-strategy/SKILL.md` | CREATE | Reusable skill defining keyword query patterns, year-anchoring, pseudo-answer generation, query variation, source weighting | US-008 |
| 5 | `docs/adr/0020-audit-web-independent.md` | CREATE | ADR for Decision 1: /audit-web independent from /audit-full | Spec Decision 1 |
| 6 | `docs/adr/0021-technology-radar-format.md` | CREATE | ADR for Decision 2: 4+1 Technology Radar quadrants with rings | Spec Decision 2 |
| 7 | `docs/commands-reference.md` | UPDATE | Add `/audit-web` row to Audit Commands table | US-009, AC-009.1 |
| 8 | `CLAUDE.md` | UPDATE | Add `/audit-web` to audit command list in Slash Commands section | US-009, AC-009.2 |
| 9 | `docs/backlog/BL-081-web-upgrade-audit.md` | UPDATE | Change `status: open` to `status: promoted` | US-009, AC-009.3 |
| 10 | `CHANGELOG.md` | UPDATE | Add BL-081 entry under Features | Spec Doc Impact |

## Pass Conditions

Each PC verifies one or more ACs. Assertions use `test -f` (file exists), `grep -q` (content present), and `ecc validate` (frontmatter valid).

### Phase 1: Command File (`commands/audit-web.md`)

| PC | Assertion | ACs Covered |
|----|-----------|-------------|
| PC-001 | `test -f commands/audit-web.md` | AC-001.1 |
| PC-002 | `grep -q '^description:' commands/audit-web.md` | AC-001.4 |
| PC-003 | `grep -q '^allowed-tools:' commands/audit-web.md` | AC-001.4 |
| PC-004 | `grep -q 'MANDATORY WORKFLOW' commands/audit-web.md` | AC-001.4 |
| PC-005 | `grep -q 'narrative-conventions' commands/audit-web.md` | AC-001.5 |
| PC-006 | `grep -q 'INVENTORY' commands/audit-web.md && grep -q 'LANDSCAPE SCAN' commands/audit-web.md && grep -q 'EVALUATE' commands/audit-web.md && grep -q 'SYNTHESIZE' commands/audit-web.md` | AC-001.1 |
| PC-007 | `grep -q '\-\-focus' commands/audit-web.md` | AC-001.2, AC-001.3 |
| PC-008 | `grep -q 'Cargo.toml' commands/audit-web.md` | AC-002.1 |
| PC-009 | `grep -q 'package.json' commands/audit-web.md` | AC-002.2 |
| PC-010 | `grep -q 'pyproject.toml' commands/audit-web.md && grep -q 'requirements.txt' commands/audit-web.md` | AC-002.3 |
| PC-011 | `grep -q 'go.mod' commands/audit-web.md` | AC-002.4 |
| PC-012 | `grep -q 'CLAUDE.md' commands/audit-web.md && grep -q 'ARCHITECTURE.md' commands/audit-web.md` | AC-002.5 |
| PC-013 | `grep -q 'cost' commands/audit-web.md` (cost consent gate) | AC-002.7, AC-002.8 |
| PC-014 | `grep -q 'No dependency manifests' commands/audit-web.md` | AC-002.9 |
| PC-015 | `grep -q 'not detected' commands/audit-web.md` | AC-002.10 |
| PC-016 | `grep -q '50' commands/audit-web.md && grep -q '100' commands/audit-web.md` (cap at 50 per category when >100) | AC-002.11 |
| PC-017 | `grep -q 'web-scout' commands/audit-web.md` (delegates to orchestrator) | AC-003.1 |
| PC-018 | `grep -q 'dep-scanner' commands/audit-web.md && grep -q 'arch-scout' commands/audit-web.md && grep -q 'tool-scout' commands/audit-web.md` | AC-003.1 |
| PC-019 | `grep -q 'triangulation\|3.*sources\|three.*sources' commands/audit-web.md` | AC-003.2 |
| PC-020 | `grep -q 'web-research-strategy' commands/audit-web.md` | AC-003.3 |
| PC-021 | `grep -q 'fallback\|exa-web-search' commands/audit-web.md` | AC-003.5 |
| PC-022 | `grep -q 'fail\|timeout\|proceed' commands/audit-web.md` (agent failure handling) | AC-003.6 |
| PC-023 | `grep -q 'condensed' commands/audit-web.md` | AC-003.7 |
| PC-024 | `grep -q 'concurrency.*4\|4.*simultaneous\|cap.*4' commands/audit-web.md` | AC-003.8 |
| PC-025 | `grep -q 'strategic fit' commands/audit-web.md && grep -q 'maturity' commands/audit-web.md && grep -q 'migration effort' commands/audit-web.md` | AC-004.1 |
| PC-026 | `grep -q 'Techniques' commands/audit-web.md && grep -q 'Feature Opportunities' commands/audit-web.md && grep -q 'Adopt' commands/audit-web.md && grep -q 'Hold' commands/audit-web.md` | AC-004.2 |
| PC-027 | `grep -q 'fit.*4.*maturity.*4.*effort.*2\|Adopt' commands/audit-web.md` (Adopt ring rules) | AC-004.3 |
| PC-028 | `grep -q 'maturity.*<.*2\|maturity below 2' commands/audit-web.md` (low maturity → Assess/Hold) | AC-004.4 |
| PC-029 | `grep -q 'dedup\|duplicate\|merge' commands/audit-web.md` | AC-004.5 |
| PC-030 | `grep -q 'web-radar-.*\.md' commands/audit-web.md` | AC-005.1 |
| PC-031 | `grep -q 'docs/audits' commands/audit-web.md` | AC-005.1 |
| PC-032 | `grep -q 'top 10\|top-10\|Top 10' commands/audit-web.md` | AC-005.3 |
| PC-033 | `grep -q 'revision\|r2' commands/audit-web.md` | AC-005.4 |
| PC-034 | `grep -q 'fewer than 10\|less than 10' commands/audit-web.md` | AC-005.5 |
| PC-035 | `grep -q 'No upgrade opportunities' commands/audit-web.md` | AC-005.6 |
| PC-036 | `grep -q '/backlog' commands/audit-web.md` (next steps guidance) | AC-005.7 |
| PC-037 | `grep -q 'read-only\|NOT modify\|DO NOT' commands/audit-web.md` | AC-005.8 |
| PC-038 | `ecc validate commands` (validates frontmatter) | AC-001.4 |
| PC-039 | `grep -q 'TodoWrite' commands/audit-web.md` | Convention (from audit-full pattern) |

### Phase 2: Orchestrator Agent (`agents/web-scout.md`)

| PC | Assertion | ACs Covered |
|----|-----------|-------------|
| PC-040 | `test -f agents/web-scout.md` | AC-006.1 |
| PC-041 | `grep -q '^name:' agents/web-scout.md && grep -q '^description:' agents/web-scout.md && grep -q '^tools:' agents/web-scout.md && grep -q '^model:' agents/web-scout.md` | AC-006.1 |
| PC-042 | `grep -q 'web-radar-analyst' agents/web-scout.md` | AC-006.2 |
| PC-043 | `grep -q 'Task' agents/web-scout.md` (parallel Task subagents) | AC-006.2 |
| PC-044 | `grep -q 'focus' agents/web-scout.md` | AC-006.3 |
| PC-045 | `grep -q 'dedup\|deduplicate\|unified' agents/web-scout.md` | AC-006.4 |
| PC-046 | `ecc validate agents` | AC-006.1 |

### Phase 3: Search Agent (`agents/web-radar-analyst.md`)

| PC | Assertion | ACs Covered |
|----|-----------|-------------|
| PC-047 | `test -f agents/web-radar-analyst.md` | AC-007.1 |
| PC-048 | `grep -q 'WebSearch' agents/web-radar-analyst.md` | AC-007.1 |
| PC-049 | `grep -q 'haiku' agents/web-radar-analyst.md` | AC-007.1 |
| PC-050 | `grep -q 'triangulation\|3.*sources' agents/web-radar-analyst.md` | AC-007.2 |
| PC-051 | `grep -q 'title' agents/web-radar-analyst.md && grep -q 'category' agents/web-radar-analyst.md && grep -q 'source' agents/web-radar-analyst.md` | AC-007.3 |
| PC-052 | `ecc validate agents` | AC-007.1 |

### Phase 4: Skill (`skills/web-research-strategy/SKILL.md`)

| PC | Assertion | ACs Covered |
|----|-----------|-------------|
| PC-053 | `test -f skills/web-research-strategy/SKILL.md` | AC-008.1 |
| PC-054 | `grep -q '^name:' skills/web-research-strategy/SKILL.md && grep -q '^description:' skills/web-research-strategy/SKILL.md` | AC-008.1 |
| PC-055 | `grep -q 'keyword' skills/web-research-strategy/SKILL.md && grep -q 'year' skills/web-research-strategy/SKILL.md` | AC-008.3 |
| PC-056 | `grep -q 'pseudo-answer\|pseudo answer' skills/web-research-strategy/SKILL.md` | AC-008.3 |
| PC-057 | `grep -q 'retry\|variation' skills/web-research-strategy/SKILL.md` | AC-008.3 |
| PC-058 | `grep -q 'source weighting\|primary.*secondary' skills/web-research-strategy/SKILL.md` | AC-008.3 |
| PC-059 | `ecc validate skills` | AC-008.1 |

### Phase 5: ADRs

| PC | Assertion | ACs Covered |
|----|-----------|-------------|
| PC-060 | `test -f docs/adr/0020-audit-web-independent.md` | Spec Decision 1 |
| PC-061 | `grep -q 'Accepted' docs/adr/0020-audit-web-independent.md` | Spec Decision 1 |
| PC-062 | `grep -q 'audit-full' docs/adr/0020-audit-web-independent.md` (explains independence rationale) | Spec Decision 1 |
| PC-063 | `test -f docs/adr/0021-technology-radar-format.md` | Spec Decision 2 |
| PC-064 | `grep -q 'Accepted' docs/adr/0021-technology-radar-format.md` | Spec Decision 2 |
| PC-065 | `grep -q 'Feature Opportunities' docs/adr/0021-technology-radar-format.md` (5th quadrant) | Spec Decision 2 |

### Phase 6: Documentation Updates

| PC | Assertion | ACs Covered |
|----|-----------|-------------|
| PC-066 | `grep -q 'audit-web' docs/commands-reference.md` | AC-009.1 |
| PC-067 | `grep -q 'audit-web' CLAUDE.md` | AC-009.2 |
| PC-068 | `grep -q 'status: promoted' docs/backlog/BL-081-web-upgrade-audit.md` | AC-009.3 |
| PC-069 | `grep -q 'BL-081' CHANGELOG.md` | Spec Doc Impact |

### Phase 7: Adversary-Required Additions

| PC | Assertion | ACs Covered |
|----|-----------|-------------|
| PC-070 | `grep -q 'focus.*deps.*dep-scanner' commands/audit-web.md` | AC-003.4 (focus-to-agent mapping) |
| PC-071 | `grep -q 'allowedTools' agents/web-scout.md` | ECC convention (subagent spawn) |
| PC-072 | `grep -q 'unavailable.*proceed\|without tracking' commands/audit-web.md` | Convention (TodoWrite graceful degradation) |
| PC-073 | `grep -q 'rationale' commands/audit-web.md && grep -q 'source.*citation\|source URLs\|3+ source' commands/audit-web.md` | AC-005.2 (per-finding report structure) |
| PC-074 | `grep -q 'create.*directory\|mkdir\|does not exist.*create' commands/audit-web.md` | AC-005.1 (directory creation) |
| PC-075 | `grep -q 'confirmation\|AskUserQuestion\|consent' commands/audit-web.md` | AC-002.7 (cost consent gate display) |
| PC-076 | `grep -q 'cancel\|decline\|cancelled' commands/audit-web.md` | AC-002.8 (graceful cancel) |

Note: AC-008.2 (spec-dev references web-research-strategy) is deferred — updating spec-dev.md is a separate follow-up change to avoid scope creep in this content-heavy implementation.

## AC-to-PC Traceability

| AC | PCs |
|----|-----|
| AC-001.1 | PC-001, PC-006 |
| AC-001.2 | PC-007 |
| AC-001.3 | PC-007 |
| AC-001.4 | PC-002, PC-003, PC-004, PC-038 |
| AC-001.5 | PC-005 |
| AC-002.1 | PC-008 |
| AC-002.2 | PC-009 |
| AC-002.3 | PC-010 |
| AC-002.4 | PC-011 |
| AC-002.5 | PC-012 |
| AC-002.6 | PC-007 |
| AC-002.7 | PC-013 |
| AC-002.8 | PC-013 |
| AC-002.9 | PC-014 |
| AC-002.10 | PC-015 |
| AC-002.11 | PC-016 |
| AC-003.1 | PC-017, PC-018 |
| AC-003.2 | PC-019 |
| AC-003.3 | PC-020 |
| AC-003.4 | PC-007 |
| AC-003.5 | PC-021 |
| AC-003.6 | PC-022 |
| AC-003.7 | PC-023 |
| AC-003.8 | PC-024 |
| AC-004.1 | PC-025 |
| AC-004.2 | PC-026 |
| AC-004.3 | PC-027 |
| AC-004.4 | PC-028 |
| AC-004.5 | PC-029 |
| AC-005.1 | PC-030, PC-031 |
| AC-005.2 | PC-026 |
| AC-005.3 | PC-032 |
| AC-005.4 | PC-033 |
| AC-005.5 | PC-034 |
| AC-005.6 | PC-035 |
| AC-005.7 | PC-036 |
| AC-005.8 | PC-037 |
| AC-006.1 | PC-040, PC-041, PC-046 |
| AC-006.2 | PC-042, PC-043 |
| AC-006.3 | PC-044 |
| AC-006.4 | PC-045 |
| AC-007.1 | PC-047, PC-048, PC-049, PC-052 |
| AC-007.2 | PC-050 |
| AC-007.3 | PC-051 |
| AC-008.1 | PC-053, PC-054, PC-059 |
| AC-008.2 | PC-020 (command references skill) |
| AC-008.3 | PC-055, PC-056, PC-057, PC-058 |
| AC-009.1 | PC-066 |
| AC-009.2 | PC-067 |
| AC-009.3 | PC-068 |

## TDD Implementation Order

Content-only TDD: write a shell test script that asserts pass conditions (RED), then create the Markdown file (GREEN), then refine (REFACTOR).

### Phase 1: Command File
**Layers**: Adapter (command definition)
**Files**: `commands/audit-web.md`
**PCs**: PC-001 through PC-039
**Commit cadence**:
1. `test: add audit-web command pass conditions (PC-001..PC-039)`
2. `feat: create /audit-web command file (BL-081)`
3. `refactor: polish audit-web command wording` (if needed)

**Content requirements** (derived from ACs):
- Frontmatter: `description`, `allowed-tools: [Task, Read, Grep, Glob, WebSearch, Write, TodoWrite, AskUserQuestion]`
- Mandatory workflow banner + narrative conventions reference
- Arguments section: `--focus=<dims>` (comma-separated: deps, arch, tools, ecosystem, competitors, user-requests, blogs, research; default: all)
- TodoWrite checklist for all 4 phases
- Phase 1 INVENTORY: polyglot manifest detection (Cargo.toml, package.json, pyproject.toml, requirements.txt, go.mod), pattern extraction from CLAUDE.md + ARCHITECTURE.md, focus filtering, graceful skip when no manifests found, "architectural patterns: not detected" fallback, 100-dep threshold with 50/category cap, cost consent gate with AskUserQuestion, graceful cancel
- Phase 2 LANDSCAPE SCAN: delegate to `web-scout` agent, pass inventory context, 8 agent categories, reference `web-research-strategy` skill, concurrency cap of 4, WebSearch fallback to exa-web-search, agent failure tolerance, condensed findings only
- Phase 3 EVALUATE: 3-dimension scoring (strategic fit, maturity, migration effort 0-5), radar quadrant classification (Techniques, Tools, Platforms, Languages & Frameworks, Feature Opportunities), ring classification (Adopt, Trial, Assess, Hold), Adopt threshold (fit>=4, maturity>=4, effort<=2), low maturity rule (maturity<2 → Assess/Hold), duplicate merge
- Phase 4 SYNTHESIZE: write report to `docs/audits/web-radar-YYYY-MM-DD.md`, create directory if needed, revision suffix for same-day reruns, radar format grouped by quadrant then ring, per-finding scores and 3+ sources, top-10 terminal summary (show all if <10), "No upgrade opportunities" for empty results, next steps guidance (/backlog, /spec), read-only stop
- Related Agents section listing web-scout and web-radar-analyst

### Phase 2: Orchestrator Agent
**Layers**: Adapter (agent definition)
**Files**: `agents/web-scout.md`
**PCs**: PC-040 through PC-046
**Commit cadence**:
1. `test: add web-scout agent pass conditions (PC-040..PC-046)`
2. `feat: create web-scout orchestrator agent (BL-081)`

**Content requirements**:
- Frontmatter: `name: web-scout`, `description`, `tools: ["Task", "Read", "Grep", "Glob", "WebSearch"]`, `model: opus`, `skills: ["web-research-strategy"]`
- Role description: orchestrate parallel web research dispatch
- Input: tech stack context (dependencies, patterns, tools, domain), scope/focus filter
- Dispatch: launch up to 8 parallel Task subagents using `web-radar-analyst` agent, each with a category prompt and query templates
- Concurrency: cap at 4 simultaneous, queue remaining
- Focus filtering: map `--focus` values to agent subset
- Agent mapping table: focus value -> agent(s)
- Aggregation: collect findings, deduplicate by title/topic, return unified list
- Error handling: note failed agents, proceed with available results
- TodoWrite tracking with graceful degradation

### Phase 3: Search Agent
**Layers**: Adapter (agent definition)
**Files**: `agents/web-radar-analyst.md`
**PCs**: PC-047 through PC-052
**Commit cadence**:
1. `test: add web-radar-analyst agent pass conditions (PC-047..PC-052)`
2. `feat: create web-radar-analyst search agent (BL-081)`

**Content requirements**:
- Frontmatter: `name: web-radar-analyst`, `description`, `tools: ["WebSearch"]`, `model: haiku`
- Role: execute 2-3 web searches for a given category using provided query templates
- Input: category name, tech stack context, query templates
- Process: use `web-research-strategy` skill patterns, execute searches, apply source triangulation (3+ independent sources)
- Output format: structured finding with title, category, source URLs (3+), summary, relevance score (0-5)
- Fallback: if WebSearch unavailable, try exa-web-search; if both fail, return "search skipped" finding
- Keep output condensed (no raw search content)

### Phase 4: Skill
**Layers**: Entity (reusable knowledge)
**Files**: `skills/web-research-strategy/SKILL.md`
**PCs**: PC-053 through PC-059
**Commit cadence**:
1. `test: add web-research-strategy skill pass conditions (PC-053..PC-059)`
2. `feat: create web-research-strategy skill (BL-081)`

**Content requirements**:
- Frontmatter: `name: web-research-strategy`, `description`, `origin: ECC`
- When to Apply section
- Strategy principles: short keyword queries, year-anchored queries, pseudo-answer generation, query variation on retry, source triangulation, source weighting (primary > secondary), channel diversity
- Query template patterns per category
- Source weighting tiers: primary (official docs, release notes, changelogs) > secondary (blog posts, tutorials, conference talks) > tertiary (social media, forums)
- Anti-patterns: natural language queries, single-source findings, stale (undated) results

### Phase 5: ADRs
**Layers**: Entity (architectural decisions)
**Files**: `docs/adr/0020-audit-web-independent.md`, `docs/adr/0021-technology-radar-format.md`
**PCs**: PC-060 through PC-065
**Commit cadence**:
1. `test: add ADR pass conditions (PC-060..PC-065)`
2. `docs: create ADR-0020 and ADR-0021 (BL-081)`

**ADR-0020 content** (audit-web independent from audit-full):
- Status: Accepted
- Context: /audit-full scans internal code health (architecture, security, testing, etc.); /audit-web scans external ecosystem opportunities. Combining them would add latency to /audit-full, conflate internal health with external opportunity, and break the cross-correlation model that /audit-full uses between its domain agents.
- Decision: /audit-web is a standalone command, not a new domain within /audit-full.
- Consequences: separate invocation, separate report output, no cross-correlation with internal audit findings (by design).

**ADR-0021 content** (technology radar format):
- Status: Accepted
- Context: ThoughtWorks Technology Radar uses 4 quadrants (Techniques, Tools, Platforms, Languages & Frameworks) with 4 rings (Adopt, Trial, Assess, Hold). ECC needs a 5th quadrant for feature opportunities discovered via competitor analysis and user request mining.
- Decision: 4+1 quadrants (standard 4 + Feature Opportunities) with standard 4 rings. JSON/CSV compatible with ThoughtWorks BYOR tool if user wants visualization.
- Consequences: extra quadrant requires documentation, compatible with standard radar visualization tools for the standard 4 quadrants.

### Phase 6: Documentation Updates
**Layers**: Adapter (documentation)
**Files**: `docs/commands-reference.md`, `CLAUDE.md`, `docs/backlog/BL-081-web-upgrade-audit.md`, `CHANGELOG.md`
**PCs**: PC-066 through PC-069
**Commit cadence**:
1. `test: add documentation update pass conditions (PC-066..PC-069)`
2. `docs: register /audit-web in command reference and CLAUDE.md (BL-081)`

**Update details**:
- `docs/commands-reference.md`: Add row `| /audit-web | Web-based upgrade discovery — Technology Radar output |` to Audit Commands table
- `CLAUDE.md`: Add `/audit-web` to the audit command parenthetical list in the Slash Commands section
- `docs/backlog/BL-081-web-upgrade-audit.md`: Change `status: open` to `status: promoted` in frontmatter
- `CHANGELOG.md`: Add BL-081 feature entry under current version

## Risks & Mitigations

- **Risk**: `ecc validate` may not recognize new content files if validation rules are strict about known file lists.
  - Mitigation: Run `ecc validate commands`, `ecc validate agents`, `ecc validate skills` after each phase to catch frontmatter issues early.

- **Risk**: Grep-based pass conditions may match false positives (e.g., "focus" appearing in unrelated context).
  - Mitigation: Use specific multi-word patterns where possible (e.g., `strategic fit` not just `fit`). Phase author reviews grep patterns against actual file content.

- **Risk**: CHANGELOG format inconsistency with existing entries.
  - Mitigation: Read existing CHANGELOG entries and match the established format exactly.

## Success Criteria

- [ ] All 69 pass conditions (PC-001 through PC-069) pass
- [ ] `ecc validate commands` passes
- [ ] `ecc validate agents` passes
- [ ] `ecc validate skills` passes
- [ ] `cargo test` still passes (no Rust changes, but verify no regressions)
- [ ] All 10 deliverable files exist or are updated
- [ ] Every AC from the spec maps to at least one passing PC
