---
name: tool-use
category: agentic
tags: [agentic, tools, function-calling, grounding]
languages: [python, typescript, rust]
difficulty: beginner
---

## Intent

Extend an LLM agent's capabilities beyond text generation by providing it with callable tools that interact with external systems, APIs, and data sources.

## Problem

LLMs have knowledge cutoffs, cannot access real-time data, and cannot perform actions in the world. Pure text generation leads to hallucination when facts are needed. The model needs a structured way to invoke external capabilities.

## Solution

Define tools with typed schemas (name, description, parameters). Present available tools to the model. The model selects a tool and provides arguments. The harness executes the tool and returns the result to the model for further reasoning.

## Language Implementations

### Python

```python
from dataclasses import dataclass
from typing import Callable, Any

@dataclass(frozen=True)
class Tool:
    name: str
    description: str
    parameters: dict
    execute: Callable[..., Any]

def dispatch_tool(tools: dict[str, Tool], name: str, args: dict) -> str:
    if name not in tools:
        return f"Error: unknown tool '{name}'"
    tool = tools[name]
    try:
        result = tool.execute(**args)
        return str(result)
    except Exception as e:
        return f"Error: {e}"
```

### Typescript

```typescript
interface Tool {
  readonly name: string;
  readonly description: string;
  readonly parameters: Record<string, ParameterSchema>;
  readonly execute: (args: Record<string, unknown>) => Promise<string>;
}

async function dispatchTool(
  tools: ReadonlyMap<string, Tool>, name: string, args: Record<string, unknown>
): Promise<string> {
  const tool = tools.get(name);
  if (!tool) return `Error: unknown tool '${name}'`;
  try {
    return await tool.execute(args);
  } catch (e) {
    return `Error: ${e instanceof Error ? e.message : String(e)}`;
  }
}
```

**ECC Integration:**

ECC agent frontmatter defines available tools via the `tools` field (e.g., `tools: [Read, Grep, Glob, Bash]`). The Claude Code harness reads this list and constrains the agent's tool access. When spawning subagents, `allowedTools` must be specified per `rules/ecc/development.md`. The `hooks/` system uses `PreToolUse` and `PostToolUse` hooks to validate and post-process tool calls. See `hooks.json` for tool-gating examples like `pre:write-edit:worktree-guard`.

## When to Use

- When the agent needs access to real-time data (search, APIs, databases).
- When tasks require side effects (file writes, deployments, notifications).
- When grounding the model in facts reduces hallucination risk.

## When NOT to Use

- When the model's training data is sufficient for the task.
- When tool execution has dangerous side effects without proper sandboxing.
- When the overhead of tool calls exceeds the benefit (simple text tasks).

## Anti-Patterns

- Providing too many tools — overwhelming the model's selection ability.
- Missing error handling in tool execution — unhandled exceptions crash the loop.
- Not describing tools clearly — the model cannot select tools it does not understand.

## Related Patterns

- [agentic/react](react.md) — tool use is the "Act" step in the ReAct loop.
- [agentic/guardrails](guardrails.md) — validate tool inputs and outputs.
- [agentic/human-in-the-loop](human-in-the-loop.md) — require approval for dangerous tools.

## References

- Anthropic — Tool Use with Claude: https://docs.anthropic.com/en/docs/build-with-claude/tool-use
- Schick et al. — Toolformer (2023): https://arxiv.org/abs/2302.04761
