---
name: multi-agent
category: agentic
tags: [agentic, orchestration, delegation, parallel]
languages: [python, typescript, rust]
difficulty: advanced
---

## Intent

Solve complex tasks by coordinating multiple specialized agents, each with focused capabilities, under the direction of an orchestrator agent.

## Problem

A single agent with all tools and responsibilities becomes overloaded, leading to poor tool selection, context window exhaustion, and unreliable behavior. Complex tasks benefit from division of labor, but agents need coordination to avoid conflicts and redundant work.

## Solution

Create specialized agents with narrow tool sets and focused system prompts. An orchestrator agent decomposes the task, delegates sub-tasks to specialists, collects results, and synthesizes the final output. Communication follows defined protocols (shared state, message passing, or structured handoffs).

## Language Implementations

### Python

```python
from dataclasses import dataclass

@dataclass(frozen=True)
class AgentResult:
    agent_id: str
    output: str
    status: str

@dataclass(frozen=True)
class TaskAssignment:
    agent_id: str
    sub_task: str

async def orchestrate(
    orchestrator, specialists: dict, task: str
) -> str:
    assignments = orchestrator.decompose(task)
    results = await asyncio.gather(*[
        specialists[a.agent_id].execute(a.sub_task)
        for a in assignments
    ])
    return orchestrator.synthesize(task, results)
```

### Typescript

```typescript
interface AgentResult {
  readonly agentId: string;
  readonly output: string;
  readonly status: "success" | "failure";
}

async function orchestrate(
  orchestrator: Orchestrator,
  specialists: ReadonlyMap<string, Agent>,
  task: string
): Promise<string> {
  const assignments = await orchestrator.decompose(task);
  const results = await Promise.all(
    assignments.map(a => specialists.get(a.agentId)!.execute(a.subTask))
  );
  return orchestrator.synthesize(task, results);
}
```

### ECC Integration

ECC is built around multi-agent orchestration. Commands like `/verify` run `code-reviewer`, `arch-reviewer`, and language-specific reviewers in parallel (see `commands/verify.md`). The `/audit-full` command launches all audit agents simultaneously. Each agent has constrained `tools` and `allowedTools` in its frontmatter per `rules/ecc/development.md`. The `agents/` directory contains 30+ specialists. Parallel execution is mandated in `rules/common/agents.md` for independent operations.

## When to Use

- When the task requires diverse expertise (security + code review + architecture).
- When sub-tasks are independent and can run in parallel.
- When a single agent's context window would be exhausted by the full task.

## When NOT to Use

- When the task is simple enough for a single agent.
- When sub-tasks are tightly coupled and require shared mutable state.
- When the orchestration overhead exceeds the parallelization benefit.

## Anti-Patterns

- Giving all agents all tools — defeats the purpose of specialization.
- No coordination protocol — agents duplicate work or produce conflicting outputs.
- Too many agents for a simple task — orchestration overhead dominates.

## Related Patterns

- [agentic/planning](planning.md) — the orchestrator uses planning to decompose tasks.
- [agentic/tool-use](tool-use.md) — each specialist agent uses focused tool sets.
- [agentic/guardrails](guardrails.md) — apply guardrails to each agent independently.

## References

- Wu et al. — AutoGen: Enabling Next-Gen LLM Applications via Multi-Agent Conversation (2023): https://arxiv.org/abs/2308.08155
- Hong et al. — MetaGPT: Multi-Agent Collaborative Framework (2023): https://arxiv.org/abs/2308.00352
