---
name: reflection
category: agentic
tags: [agentic, self-critique, quality, iterative]
languages: [python, typescript, rust]
difficulty: intermediate
---

## Intent

Improve agent output quality by having the agent critique its own work and iterate, catching errors and improving reasoning before producing a final result.

## Problem

LLM agents produce outputs that contain errors, hallucinations, or suboptimal reasoning. A single pass through the model misses issues that a second look would catch. Without self-review, the agent's first attempt is the final attempt.

## Solution

After producing an initial output, the agent evaluates its own work against explicit criteria. If the self-critique identifies issues, the agent revises and re-evaluates. This loop continues until the output passes the quality check or a maximum iteration count is reached.

## Language Implementations

### Python

```python
from dataclasses import dataclass

@dataclass(frozen=True)
class Critique:
    passed: bool
    issues: tuple[str, ...]
    suggestions: tuple[str, ...]

def reflect_loop(agent, task: str, max_rounds: int = 3) -> str:
    output = agent.generate(task)
    for _ in range(max_rounds):
        critique = agent.critique(task, output)
        if critique.passed:
            return output
        output = agent.revise(task, output, critique)
    return output  # best effort after max rounds
```

### Typescript

```typescript
interface Critique {
  readonly passed: boolean;
  readonly issues: readonly string[];
  readonly suggestions: readonly string[];
}

async function reflectLoop(agent: Agent, task: string, maxRounds = 3): Promise<string> {
  let output = await agent.generate(task);
  for (let i = 0; i < maxRounds; i++) {
    const critique = await agent.critique(task, output);
    if (critique.passed) return output;
    output = await agent.revise(task, output, critique);
  }
  return output;
}
```

**ECC Integration:**

ECC uses reflection extensively in its adversarial review pipeline. The `spec-adversary` and `solution-adversary` agents in `agents/` critique spec and design artifacts, producing PASS/FAIL verdicts with dimensional rationale. The `/verify` command runs `code-reviewer` and `arch-reviewer` as reflection agents that critique implementation quality. The adversary conventions in `rules/ecc/development.md` mandate rationale for every evaluated dimension.

## When to Use

- When output quality is more important than latency.
- When the task has clear evaluation criteria the agent can check against.
- When errors in the output are costly (code generation, factual claims).

## When NOT to Use

- When latency is the primary concern and a single pass is acceptable.
- When evaluation criteria are subjective and cannot be codified.
- When the additional API calls are cost-prohibitive.

## Anti-Patterns

- Reflecting without concrete criteria — the agent says "looks good" every time.
- Too many reflection rounds without convergence — diminishing returns after 2-3 rounds.
- Using the same prompt for generation and critique — separate concerns for better results.

## Related Patterns

- [agentic/react](react.md) — reflection can be inserted between ReAct steps.
- [agentic/guardrails](guardrails.md) — external validation complements self-reflection.
- [agentic/planning](planning.md) — reflect on plans before execution.

## References

- Shinn et al. — Reflexion: Language Agents with Verbal Reinforcement Learning (2023): https://arxiv.org/abs/2303.11366
- Madaan et al. — Self-Refine: Iterative Refinement with Self-Feedback (2023): https://arxiv.org/abs/2303.17651
