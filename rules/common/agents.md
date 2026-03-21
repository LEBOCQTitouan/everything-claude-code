# Agent Orchestration

## Available Agents

Located in `~/.claude/agents/`:

| Agent | Purpose | When to Use |
|-------|---------|-------------|
| planner | Implementation planning | Complex features, refactoring |
| architect | System design | Architectural decisions |
| tdd-guide | Test-driven development | New features, bug fixes |
| code-reviewer | Code review | After writing code |
| security-reviewer | Security analysis | Before commits |
| build-error-resolver | Fix build errors | When build fails |
| e2e-runner | E2E testing | Critical user flows |
| refactor-cleaner | Dead code cleanup | Code maintenance |
| doc-updater | Documentation | Updating docs |

## Command → Agent Mapping

Agents are invoked automatically by the 5 commands:

| Command | Agents Used |
|---------|------------|
| `/spec` | planner, tdd-guide, architect (refactor mode), security-reviewer (security mode) |
| `/build-fix` | build-error-resolver |
| `/verify` | code-reviewer, arch-reviewer, go-reviewer, python-reviewer (auto-detected) |
| `/e2e` | e2e-runner |
| `/doc-suite` | doc-orchestrator, doc-analyzer, doc-generator, doc-validator, doc-reporter |

## Parallel Task Execution

ALWAYS use parallel Task execution for independent operations:

```markdown
# GOOD: Parallel execution
Launch 3 agents in parallel:
1. Agent 1: Security analysis of auth module
2. Agent 2: Performance review of cache system
3. Agent 3: Type checking of utilities

# BAD: Sequential when unnecessary
First agent 1, then agent 2, then agent 3
```

## Multi-Perspective Analysis

For complex problems, use split role sub-agents:
- Factual reviewer
- Senior engineer
- Security expert
- Consistency reviewer
- Redundancy checker
