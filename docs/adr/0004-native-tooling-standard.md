# 0004. Native Claude Code Tooling Standard

Date: 2026-03-21

## Status

Accepted

## Context

A native tooling audit of 163 files found 160 instances where Claude Code's native tools (TodoWrite, TodoRead, allowedTools, AskUserQuestion, context:fork) were not leveraged in command and agent definitions. This caused:

1. **No progress visibility** — multi-phase commands (plan-dev, solution, verify) had no way to track or resume progress across sessions
2. **Over-permissioned agents** — sub-agents spawned without allowedTools could access tools beyond their needs (e.g., Write for read-only reviewers)
3. **Context pollution** — parallel agent outputs merged into a single context window without isolation
4. **Free-text disambiguation** — the plan router used free-text correction instead of structured options

## Decision

Adopt a native tooling standard across all command and agent markdown files:

- **TodoWrite checklists** on every multi-phase command and orchestrator agent, with graceful degradation when unavailable
- **TodoRead re-entry** on commands that support session resumption (solution, implement)
- **allowedTools annotations** on every agent spawn line, scoped to the minimum tool set
- **context:"fork"** on parallel agent spawns in orchestrators to isolate intermediate output
- **AskUserQuestion with structured options** for disambiguation (replacing free-text)

## Consequences

**Positive:**
- Users can see phase-by-phase progress in long-running commands
- Session crashes mid-pipeline can resume from the last completed phase
- Sub-agents cannot accidentally write files or execute commands beyond their role
- Parallel agent outputs stay isolated until the orchestrator aggregates them

**Negative:**
- Adds ~10-15 lines of boilerplate to each command/agent file
- TodoWrite items must be kept in sync with phase names if phases are renamed
- allowedTools must be updated when agent capabilities change
