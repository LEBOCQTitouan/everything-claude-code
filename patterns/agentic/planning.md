---
name: planning
category: agentic
tags: [agentic, planning, decomposition, strategy]
languages: [python, typescript, rust]
difficulty: intermediate
---

## Intent

Improve agent success on complex tasks by generating an explicit plan before execution, decomposing the problem into ordered sub-tasks that can be tracked and adjusted.

## Problem

Agents that jump directly into execution on complex tasks often take wrong turns, miss steps, or work inefficiently. Without a plan, there is no way to track progress, detect stalls, or parallelize independent sub-tasks.

## Solution

Before executing, the agent analyzes the task and produces a structured plan: an ordered list of sub-tasks with dependencies. The plan is reviewed (by human or automated check), then executed step by step. Progress is tracked against the plan, and re-planning occurs if a step fails.

## Language Implementations

### Python

```python
from dataclasses import dataclass

@dataclass(frozen=True)
class SubTask:
    id: str
    description: str
    dependencies: tuple[str, ...]
    status: str = "pending"

@dataclass(frozen=True)
class Plan:
    goal: str
    tasks: tuple[SubTask, ...]

def create_plan(agent, goal: str) -> Plan:
    raw_plan = agent.decompose(goal)
    tasks = tuple(
        SubTask(id=t["id"], description=t["description"],
                dependencies=tuple(t.get("deps", [])))
        for t in raw_plan
    )
    return Plan(goal=goal, tasks=tasks)

def next_executable(plan: Plan) -> SubTask | None:
    completed = {t.id for t in plan.tasks if t.status == "done"}
    for task in plan.tasks:
        if task.status == "pending" and all(d in completed for d in task.dependencies):
            return task
    return None
```

### Typescript

```typescript
interface SubTask {
  readonly id: string;
  readonly description: string;
  readonly dependencies: readonly string[];
  readonly status: "pending" | "in-progress" | "done" | "failed";
}

interface Plan {
  readonly goal: string;
  readonly tasks: readonly SubTask[];
}

function nextExecutable(plan: Plan): SubTask | undefined {
  const completed = new Set(plan.tasks.filter(t => t.status === "done").map(t => t.id));
  return plan.tasks.find(
    t => t.status === "pending" && t.dependencies.every(d => completed.has(d))
  );
}
```

### ECC Integration

ECC's `/spec` and `/design` commands implement the planning pattern as a mandatory first phase. The `planner` agent in `agents/planner.md` decomposes features into phases with test targets. The `ecc-workflow` binary enforces phase ordering (`spec` -> `design` -> `implement`) via state machine in `.claude/workflow/state.json`. The `/implement` command uses `TodoWrite` for progress tracking and `tasks.md` for persistent sub-task state, following the artifact schema in `skills/artifact-schemas/`.

## When to Use

- When the task involves multiple steps with dependencies between them.
- When progress tracking and human oversight are important.
- When independent sub-tasks can be parallelized across agents.

## When NOT to Use

- When the task is simple enough that planning overhead exceeds execution time.
- When the domain is too unpredictable for upfront planning (use ReAct instead).
- When real-time responsiveness is required and planning latency is unacceptable.

## Anti-Patterns

- Over-planning — spending more time planning than executing.
- Rigid plans that cannot adapt when a sub-task fails or reveals new information.
- Plans without dependency ordering, leading to blocked execution.

## Related Patterns

- [agentic/react](react.md) — execute plan steps using the ReAct loop.
- [agentic/multi-agent](multi-agent.md) — distribute plan sub-tasks across specialized agents.
- [agentic/reflection](reflection.md) — review and revise the plan before execution.

## References

- Huang et al. — Understanding the planning of LLM agents (2024): https://arxiv.org/abs/2402.02716
- Wang et al. — Plan-and-Solve Prompting (2023): https://arxiv.org/abs/2305.04091
