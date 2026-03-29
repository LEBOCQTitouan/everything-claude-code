---
description: Web-based upgrade scan with Technology Radar output
allowed-tools: [Task, Read, Grep, Glob, WebSearch, Write, TodoWrite, AskUserQuestion]
---

# Web-Based Upgrade Audit

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.
>
> **Narrative**: See `skills/narrative-conventions/SKILL.md` conventions. Before each phase transition, agent delegation, and cost gate, tell the user what is happening and why.

Scans the web for upgrade opportunities, new tools, architectural patterns, ecosystem developments, and feature ideas relevant to this project. Produces a Technology Radar report in `docs/audits/`.

Scope: $ARGUMENTS (or all dimensions if none provided)

**STOP. DO NOT modify source code.** This is a read-only audit.

> **Tracking**: Create a TodoWrite checklist for this command's phases. If TodoWrite is unavailable, proceed without tracking — the workflow executes identically.

TodoWrite items:
- "Phase 1: INVENTORY — detect manifests and patterns"
- "Cost consent gate — ask user confirmation"
- "Phase 2: LANDSCAPE SCAN — parallel web search agents"
- "Phase 3: EVALUATE — score and classify findings"
- "Phase 4: SYNTHESIZE — write radar report"
- "Present terminal summary"

Mark each item complete as the phase finishes.

## Arguments

- `--focus=<dims>` — comma-separated list of dimensions to scan (default: all). Values: `deps`, `arch`, `tools`, `ecosystem`, `competitors`, `user-requests`, `blogs`, `research`

### Focus-to-Agent Mapping

| Focus value | Agent(s) launched |
|-------------|------------------|
| `deps` | dep-scanner |
| `arch` | arch-scout |
| `tools` | tool-scout |
| `ecosystem` | ecosystem-scout |
| `competitors` | competitor-scout |
| `user-requests` | user-request-miner |
| `blogs` | blog-miner |
| `research` | research-scout |
| `features` | competitor-scout + user-request-miner + blog-miner + research-scout |

When `--focus` is omitted, all 8 agents run. When `--focus=deps` is specified, only the dep-scanner agent is launched. Use the mapping table above to determine which agents to launch for each focus value.

---

## Phase 1: INVENTORY

Tell the user: "Starting Phase 1 — scanning your project for dependencies and architectural patterns to ground the web search queries."

Scan the project root for dependency manifests. For each manifest found, extract direct dependencies with their current versions:

| Manifest | Language/Platform | What to extract |
|----------|------------------|-----------------|
| `Cargo.toml` | Rust | `[dependencies]` and `[dev-dependencies]` sections |
| `package.json` | Node.js | `dependencies` and `devDependencies` fields |
| `pyproject.toml` | Python | `[tool.poetry.dependencies]` or `[project.dependencies]` |
| `requirements.txt` | Python | All pinned packages |
| `go.mod` | Go | `require` directives |

If no recognized dependency manifest is found (no Cargo.toml, package.json, pyproject.toml, requirements.txt, or go.mod), report "No dependency manifests detected" and skip dependency scanning.

Extract architectural patterns from `CLAUDE.md` and `docs/ARCHITECTURE.md` (hexagonal, DDD, TDD, event-driven, etc.). If `CLAUDE.md` or `docs/ARCHITECTURE.md` is missing, skip pattern extraction and note "architectural patterns: not detected".

If `--focus=deps` was specified, limit inventory output to dependency information only. Apply the focus filter before the cost consent gate.

**Dependency count cap**: If more than 100 dependencies are detected, group them by category and cap search queries at 50 per category to prevent excessive token burn.

**Cost consent gate**: Before proceeding to Phase 2, use AskUserQuestion to display the audit scope and ask for confirmation:

> "Phase 1 complete. Here is the audit scope:
> - N dependencies detected
> - M architectural patterns identified
> - K tool areas to scan
> - Estimated ~N*3 web searches
>
> This will consume search quota. Proceed with Phase 2 LANDSCAPE SCAN? (yes/no)"

If the user declines, exit gracefully with: "Audit cancelled by user."

---

## Phase 2: LANDSCAPE SCAN

Tell the user: "Starting Phase 2 — dispatching parallel web search agents to scan the ecosystem. This may take a few minutes."

Delegate to the `web-scout` orchestrator agent, passing the full inventory context (dependencies, patterns, tool areas, focus filter).

The web-scout agent launches the following parallel Task subagents — one per active focus dimension:

| Agent | Dimension |
|-------|-----------|
| dep-scanner | Dependency upgrades, newer major versions, deprecations |
| arch-scout | Architectural patterns, frameworks, structural improvements |
| tool-scout | Developer tooling, build systems, linters, CI improvements |
| ecosystem-scout | Ecosystem trends, standards, community adoption |
| competitor-scout | Competing products and OSS projects with relevant features |
| user-request-miner | Community-requested features in similar projects |
| blog-miner | Blog posts and conference talks on relevant topics |
| research-scout | Academic papers, RFCs, and research relevant to the domain |

Focus-to-agent mapping: `focus.*deps.*dep-scanner` — when `--focus=deps` is active, only the dep-scanner is launched. When `--focus=tools`, only tool-scout. Refer to the Arguments section mapping table for the full list.

The `web-research-strategy` skill governs how each subagent formulates search queries: short keyword queries, year-anchored searches, pseudo-answer generation, query variation on retry, and source weighting (primary > secondary > tertiary).

**Source triangulation**: Each finding must be validated with 3+ independent sources before being reported. Single-source findings are discarded.

**Concurrency**: The orchestrator enforces a concurrency cap of 4 simultaneous agents to avoid rate limiting. Remaining agents queue and start as slots free up.

**WebSearch fallback**: If WebSearch is unavailable, each subagent falls back to exa-web-search MCP. If both are unavailable, the subagent reports "search skipped" for its dimension without failing the pipeline.

**Agent failure tolerance**: If one or more subagents fail or timeout, proceed with available results and note which agents failed in the report.

**Output constraint**: Each subagent returns only condensed findings — no raw search content. Condensed findings are passed to Phase 3.

---

## Phase 3: EVALUATE

Tell the user: "Starting Phase 3 — scoring and classifying findings into Technology Radar quadrants."

Invoke the `web-radar-analyst` agent (or evaluate inline) to score and classify all condensed findings from Phase 2.

### 3-Dimension Scoring (0–5 scale)

Each finding is scored on three dimensions:

| Dimension | Description |
|-----------|-------------|
| **strategic fit** | How well does this align with the project's direction and goals? (0=irrelevant, 5=critical fit) |
| **maturity** | How production-ready and battle-tested is this? (0=experimental, 5=industry standard) |
| **migration effort** | How hard is it to adopt? (0=trivial, 5=very high effort — lower is better) |

### Radar Quadrant Classification

Assign each finding to one of five quadrants:

- **Techniques** — processes, practices, methods (e.g., TDD, event sourcing, hexagonal architecture)
- **Tools** — software tools, frameworks, libraries, platforms (e.g., linters, CI systems, build tools)
- **Platforms** — hosting, infrastructure, cloud services, databases
- **Languages & Frameworks** — programming languages, major application frameworks, runtimes
- **Feature Opportunities** — competitor features, user-requested capabilities, product improvements

### Ring Classification Rules

| Ring | Criteria |
|------|---------|
| **Adopt** | strategic fit >= 4 AND maturity >= 4 AND migration effort <= 2 |
| **Trial** | Promising but not fully proven in production contexts |
| **Assess** | Worth watching; investigate further before committing |
| **Hold** | Avoid for new work; monitor for decline or maturity |

Low maturity rule: if maturity < 2, place in Assess or Hold regardless of fit score.

**Duplicate merge**: If duplicate findings from different agents cover the same tool or technique, merge them into a single entry with combined source citations from all originating agents.

---

## Phase 4: SYNTHESIZE

Tell the user: "Starting Phase 4 — synthesizing findings into a Technology Radar report."

### 4a. Write the Report

Write the report to `docs/audits/web-radar-YYYY-MM-DD.md` using today's date. If the `docs/audits/` directory does not exist, create the directory before writing the report.

If a report file already exists for today's date, append a revision suffix: `web-radar-YYYY-MM-DD-r2.md` (and r3, r4 for subsequent reruns on the same day).

**Report structure**:

```markdown
# Web Upgrade Radar — YYYY-MM-DD

## Project Profile
- **Repository**: <repo name>
- **Date**: YYYY-MM-DD
- **Focus**: <focus dims or "all">
- **Agents**: dep-scanner, arch-scout, tool-scout, ecosystem-scout, competitor-scout, user-request-miner, blog-miner, research-scout

## Techniques

### Adopt
#### [Finding Title]
- **Ring**: Adopt
- **Strategic Fit**: N/5 | **Maturity**: N/5 | **Migration Effort**: N/5
- **Rationale**: Why this is recommended for adoption
- **Source Citations**: [Source 1](url), [Source 2](url), [Source 3](url)

### Trial
...

### Assess
...

### Hold
...

## Tools
(same structure)

## Platforms
(same structure)

## Languages & Frameworks
(same structure)

## Feature Opportunities
(same structure)

## Next Steps

To act on findings, run `/backlog add` or `/spec` referencing this report.
```

If no findings were produced in any quadrant, the report body should indicate: "No upgrade opportunities identified — project is current."

Per-finding entries must include: title, ring, fit/maturity/effort scores, rationale, and 3+ source citations (source URLs).

### 4b. Present Terminal Summary

Display the top 10 findings ranked by impact/effort ratio (strategic fit minus migration effort). If fewer than 10 findings exist, show all findings.

```
Web Upgrade Radar Complete
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Report: docs/audits/web-radar-YYYY-MM-DD.md
  Total findings: N

  Top findings by impact/effort ratio:
  1. [Adopt] <title> — fit:N maturity:N effort:N
  2. [Adopt] <title> — fit:N maturity:N effort:N
  ...

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**STOP. DO NOT modify source code.**

Say: "To act on findings, run `/backlog add` or `/spec` referencing this report."

---

## Related Agents

- `agents/web-scout.md` — orchestrator for parallel web search dispatch
- `agents/web-radar-analyst.md` — reusable search and scoring agent per category

## Adversarial Challenge

> After the analysis phase completes, launch an independent adversary to challenge the findings.

Launch a Task with the `audit-challenger` agent (allowedTools: [Read, Grep, Glob, Bash, WebSearch]):

- Pass the findings from the analysis phase as structured input (finding ID, severity, description, evidence)
- The agent independently re-interrogates the codebase and searches web for best practices
- Collect challenged findings: confirmed, refuted, or amended with per-finding rationale

### Quality Check

If the adversary output lacks structured per-finding verdicts (each with finding ID, verdict {confirmed|refuted|amended}, and rationale):
1. Retry once with a stricter prompt demanding the exact output format
2. If second attempt still lacks structure, surface a "Low-quality adversary output" warning alongside the raw content and proceed

### Disagreement Handling

When audit and adversary disagree on a finding:
- Display both the original finding and the challenger's assessment side by side
- Include the challenger's recommendation
- Prompt the user for final decision: accept audit / accept challenger / custom resolution

### Graceful Degradation

If the audit-challenger agent fails to spawn or returns an error:
- Emit: "Adversary challenge skipped: <reason>"
- Proceed with unchallenged findings

