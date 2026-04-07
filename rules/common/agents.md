# Agent Orchestration

Use `Agent` tool to spawn subagents. Always specify `allowedTools`.
Run independent agents in parallel — never sequentially without cause.

| Command | Agents Used |
|---------|------------|
| `/spec` | planner, tdd-guide, architect, security-reviewer |
| `/build-fix` | build-error-resolver |
| `/verify` | code-reviewer, arch-reviewer, language reviewers |
| `/e2e` | e2e-runner |
| `/doc-suite` | doc-orchestrator pipeline |

Full agent listing: `~/.claude/agents/`
