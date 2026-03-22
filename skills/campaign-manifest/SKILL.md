---
name: campaign-manifest
description: Campaign manifest schema and lifecycle — single orientation file per work item for amnesiac agents
origin: ECC
---

# Campaign Manifest

A campaign manifest (`campaign.md`) indexes all artifacts, decisions, and progress for a work item. A fresh agent reads this single file to orient instantly.

## Schema

```markdown
# Campaign: <title>

## Status
Status: <in-progress|spec-complete|design-complete|implementing|done>
Concern: <dev|fix|refactor>
Started: <ISO 8601>
Updated: <ISO 8601>

## Artifacts
| Type | Path | Status |
|------|------|--------|
| Spec | docs/specs/.../spec.md | draft / passed / revised |
| Design | docs/specs/.../design.md | draft / passed / revised |
| Tasks | docs/specs/.../tasks.md | active / complete |
| ADR | docs/adr/NNN-*.md | created / n/a |

## Toolchain
- Test: <command>
- Lint: <command>
- Build: <command>

## Grill-Me Decisions
| # | Question | Answer | Source |
|---|----------|--------|--------|

## Adversary History
| Round | Phase | Verdict | Key Findings |
|-------|-------|---------|--------------|

## Agent Outputs
| Agent | Phase | Summary |
|-------|-------|---------|

## Commit Trail
| SHA | Message | PC |
|-----|---------|-----|

## Resumption Pointer
Current step: <description>
Next action: <what to do next>
```

## Lifecycle

### Creation (Phase 0)

Campaign.md is created in `.claude/workflow/campaign.md` at Phase 0 when toolchain detection completes. Initial state: `Status: in-progress`, empty tables, toolchain values populated.

### Migration (after adversary PASS)

When the spec directory `docs/specs/YYYY-MM-DD-<slug>/` is created after adversary PASS, move `campaign.md` from `.claude/workflow/` to that directory. Update `artifacts.campaign_path` in state.json.

### Incremental Updates

- **Grill-me**: Append Q&A row after each answered question
- **Agent outputs**: Append summary row after each agent task completes
- **Adversary rounds**: Append verdict row after each adversary round
- **Commit trail**: Parent orchestrator appends SHA after each TDD commit (never subagents directly)
- **Resumption pointer**: Update at each phase transition and PC completion

### Malformed Recovery

Malformed means: missing any required section header (Status, Artifacts, Grill-Me Decisions, Adversary History, Agent Outputs, Commit Trail, Resumption Pointer) OR table bodies unparseable when rows expected per state.json timestamps. Regenerate from state.json artifacts and persisted files with warning.

### Write Mechanism

Campaign.md writes use Claude's Write tool (not shell atomic writes). Campaign writes during parallel TDD waves are performed by the parent orchestrator after subagent completion, never by subagents directly.
