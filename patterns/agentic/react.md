---
name: react
category: agentic
tags: [agentic, reasoning, action, loop]
languages: [python, typescript, rust]
difficulty: intermediate
---

## Intent

Enable an LLM agent to solve complex tasks by interleaving reasoning steps with actions, observing results, and iterating until the task is complete.

## Problem

A single LLM call cannot solve multi-step tasks that require interacting with external tools or APIs. The model needs to observe the results of its actions to decide the next step. Without structure, agents loop aimlessly or fail to converge.

## Solution

Implement a Reason-Act-Observe loop. The agent reasons about the current state, selects an action (tool call), observes the result, then reasons again. The loop terminates when the agent produces a final answer or a maximum step count is reached.

## Language Implementations

### Python

```python
from dataclasses import dataclass

@dataclass(frozen=True)
class Step:
    thought: str
    action: str
    observation: str

def react_loop(agent, query: str, tools: dict, max_steps: int = 10) -> str:
    steps: list[Step] = []
    for _ in range(max_steps):
        response = agent.reason(query, steps)
        if response.is_final:
            return response.answer
        observation = tools[response.action](response.action_input)
        steps = [*steps, Step(response.thought, response.action, observation)]
    return "Max steps reached without resolution."
```

### Typescript

```typescript
interface Step {
  readonly thought: string;
  readonly action: string;
  readonly observation: string;
}

async function reactLoop(
  agent: Agent, query: string, tools: ToolMap, maxSteps = 10
): Promise<string> {
  let steps: readonly Step[] = [];
  for (let i = 0; i < maxSteps; i++) {
    const response = await agent.reason(query, steps);
    if (response.isFinal) return response.answer;
    const observation = await tools[response.action](response.actionInput);
    steps = [...steps, { thought: response.thought, action: response.action, observation }];
  }
  return "Max steps reached without resolution.";
}
```

### ECC Integration

ECC agents follow the ReAct pattern in their core execution loop. The agent frontmatter in `agents/` defines available tools, and the Claude Code harness orchestrates the reason-act-observe cycle. The `allowedTools` field in agent frontmatter constrains which actions an agent can take, implementing a bounded action space. See `agents/planner.md` and `agents/tdd-guide.md` for agents that use multi-step ReAct loops with tool calls.

## When to Use

- When tasks require multiple steps with external tool interaction.
- When the agent needs to adapt its plan based on intermediate results.
- When transparency of reasoning is important for debugging.

## When NOT to Use

- When a single LLM call suffices (simple Q&A, classification).
- When latency is critical and multi-step loops are too slow.
- When the task has no external tools to interact with.

## Anti-Patterns

- No maximum step limit — agents can loop indefinitely.
- Not including observations in the context, causing the agent to lose track.
- Allowing too many tools, overwhelming the model's action selection.

## Related Patterns

- [agentic/tool-use](tool-use.md) — the action component of the ReAct loop.
- [agentic/reflection](reflection.md) — self-critique between reasoning steps.
- [agentic/planning](planning.md) — upfront plan before entering the loop.

## References

- Yao et al. — ReAct: Synergizing Reasoning and Acting in Language Models (2023): https://arxiv.org/abs/2210.03629
- LangChain ReAct Agent: https://python.langchain.com/docs/modules/agents/agent_types/react
