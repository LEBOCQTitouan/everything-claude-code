---
description: "Multi-agent round-table discussion — assembles a panel from BMAD, ECC, and project-domain agents for cross-perspective analysis"
allowed-tools: ["Agent", "Read", "Grep", "Glob", "Write", "AskUserQuestion"]
---

# /party

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.

Runs a multi-agent round-table on any topic. Assembles a panel from BMAD Roles, ECC Specialists, and Project Domain agents; dispatches them sequentially via the `party-coordinator` agent; and persists a synthesis document to `docs/party/`.

Arguments: `$ARGUMENTS` (topic string — optional)

---

## Phase 0: Topic Handling

**If no argument provided** (empty `$ARGUMENTS`):
- Prompt via `AskUserQuestion`: "What topic should the panel discuss?"
- If the response is empty or whitespace only, re-prompt once: "Topic cannot be empty. What topic should the panel discuss?"
- If still empty, exit with error: "A topic is required to run /party."

**Topic normalization**:
- Truncate to 200 characters for display purposes
- **Slug derivation**: lowercase → replace all non-alphanumeric characters with hyphens → collapse consecutive hyphens → trim leading/trailing hyphens → max 50 characters
- If the slug is empty after normalization, use fallback: `party-session`

Example: "Cross-cutting concerns in ECC's CLI layer (2026)" → `cross-cutting-concerns-in-ecc-s-cli-layer-2026`

---

## Phase 1: Agent Enumeration

Discover available agents from three sources:

| Source | Glob Pattern | Group Label |
|--------|-------------|------------|
| `agents/bmad-*.md` | `agents/bmad-*.md` | **BMAD Roles** |
| `agents/*.md` (excluding `bmad-*` and `party-coordinator`) | `agents/*.md` | **ECC Specialists** |
| `.claude/agents/*.md` | `.claude/agents/*.md` | **Project Domain** |

For each discovered file:
1. Read the file
2. Extract `name` and `description` from YAML frontmatter
3. If frontmatter is malformed or missing `name`, skip the file with a warning

Present the grouped inventory to the user (names + descriptions). This inventory drives Phase 2 selection.

---

## Phase 2: Selection UX

Use `AskUserQuestion` with two options:

> "How would you like to select the panel?
> A) Manual selection — I'll pick agents from the list
> B) Claude-recommended panel — Claude selects 3–6 agents with rationale"

**Constraint**: panel must contain 2–8 agents. Fewer than 2 agents is not a valid panel.

### A) Manual Selection

1. Present the grouped agent list (BMAD / ECC / Project Domain)
2. User picks agents by name or number
3. Deduplicate: if the same agent appears in multiple source groups, include it once
4. Enforce minimum 2 agents: if fewer than 2 unique agents selected, prompt again: "At least 2 agents are required. Please add more."
5. Confirm final selection with user before proceeding

### B) Claude-Recommended Panel

1. Analyze the topic against CLAUDE.md, recent backlog entries, and available agent descriptions
2. Select 3–6 agents that offer complementary perspectives on the topic
3. For each recommended agent, provide a one-line rationale citing the specific reason (e.g., references a CLAUDE.md guideline, a relevant backlog item, or a domain expertise match)
4. Present the recommended panel for user confirmation via `AskUserQuestion`:
   > "Recommended panel for '<topic>':
   > - <agent-name>: <rationale>
   > - ...
   > Approve this panel, or type agent names to adjust."
5. Apply user adjustments (add/remove agents), then confirm

**If fewer than 2 agents are available across all sources**: exit with error message explaining that at minimum 2 agents are needed and suggesting the user create agent files in `agents/` or `.claude/agents/`.

---

## Phase 3: Auto-Generation (Conditional)

**Trigger**: Only if `.claude/agents/` is absent or empty AND no Project Domain agents were found in Phase 1.

**Skip condition**: If `.claude/agents/` already has files, skip this phase entirely — do not re-generate existing agents.

If triggered:
1. Offer auto-generation via `AskUserQuestion`:
   > "No project-domain agents found in `.claude/agents/`. Would you like Claude to analyze the codebase and generate 1–3 domain-specific agents automatically? (yes/no)"
2. If yes:
   - Dispatch the `doc-analyzer` agent (allowedTools: [Read, Grep, Glob]) to analyze the codebase and generate 1-3 domain agent definitions relevant to the project
   - For each generated agent, write a `.md` file to `.claude/agents/` with valid YAML frontmatter (`name`, `description`, `tools`, `model`)
   - Add newly generated agents to the available pool for Phase 2 re-selection if still in progress
3. **Failure handling**: if `doc-analyzer` fails or returns no usable output:
   - Emit a warning: "Auto-generation failed — proceeding with BMAD and ECC agents only."
   - Continue gracefully with the existing pool; do not abort the session

---

## Phase 4: Session Execution

Dispatch the `party-coordinator` agent with:

- `panel`: the confirmed agent list (in selection order)
- `topic`: the full topic string (not truncated)
- `context`: a brief repo context summary — key CLAUDE.md sections, any relevant backlog items identified in Phase 2B, relevant recent audit findings
- `output_path`: the resolved output file path (computed in Phase 5 before dispatch)

The coordinator handles sequential dispatch and synthesis. Collect its output.

---

## Phase 5: Output Persistence

### Path Resolution

Output directory: `docs/party/`

Filename: `<slug>-<YYYYMMDD>.md` where `<slug>` is derived in Phase 0 and `<YYYYMMDD>` is today's date.

**Collision handling**: if `docs/party/<slug>-<YYYYMMDD>.md` already exists, append a numeric suffix: `<slug>-<YYYYMMDD>-2.md`, `<slug>-<YYYYMMDD>-3.md`, etc.

### Directory Creation

Create `docs/party/` directory if it does not already exist. Ensure the directory exists before writing the file.

### Write

Write the output document (returned by `party-coordinator`) to the resolved path.

**Write failure fallback**: if the write fails (permissions, disk full, path error):
- Display the full output document in the conversation as a fallback
- Warn the user: "Write to <path> failed — output shown in conversation. Copy and save manually if needed."

### Output Document Structure

The output document contains:

```markdown
---
date: <YYYY-MM-DD>
topic: <topic>
panel:
  - <agent-name>
---

# Party Session: <topic>

## Panel Composition

<table of agents and their descriptions>

## Per-Agent Output

<each agent's deliberation output, labeled>

## Synthesis

### Per-Agent Summary
### Agreements
### Disagreements
### Recommendations
### Open Questions
```

---

## Related Agents

- `agents/party-coordinator.md` — orchestrates sequential dispatch and synthesis
- `agents/doc-analyzer.md` — used for optional domain agent auto-generation
