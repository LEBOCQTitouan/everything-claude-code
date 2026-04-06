---
name: guardrails
category: agentic
tags: [agentic, safety, validation, constraints]
languages: [python, typescript, rust]
difficulty: intermediate
---

## Intent

Constrain agent behavior through automated validation of inputs, outputs, and actions, preventing harmful, off-topic, or incorrect results without requiring human intervention at every step.

## Problem

Agents can produce hallucinated content, execute dangerous actions, leak sensitive data, or drift off-topic. Human-in-the-loop approval does not scale to every action. You need automated safety checks that run continuously without blocking the agent on routine operations.

## Solution

Implement validation layers at three points: input (sanitize and validate user requests), execution (constrain available tools and validate tool arguments), and output (check responses for policy compliance, factual grounding, and format correctness). Failed checks trigger retry, escalation, or graceful degradation.

## Language Implementations

### Python

```python
from dataclasses import dataclass

@dataclass(frozen=True)
class GuardrailResult:
    passed: bool
    violations: tuple[str, ...]

def check_output(output: str, rules: tuple[callable, ...]) -> GuardrailResult:
    violations = tuple(
        msg for rule in rules
        if (msg := rule(output)) is not None
    )
    return GuardrailResult(passed=len(violations) == 0, violations=violations)

def no_pii(output: str) -> str | None:
    import re
    if re.search(r'\b\d{3}-\d{2}-\d{4}\b', output):
        return "Output contains SSN-like pattern"
    return None

def max_length(limit: int):
    def check(output: str) -> str | None:
        if len(output) > limit:
            return f"Output exceeds {limit} characters"
        return None
    return check
```

### Typescript

```typescript
interface GuardrailResult {
  readonly passed: boolean;
  readonly violations: readonly string[];
}

type Rule = (output: string) => string | null;

function checkOutput(output: string, rules: readonly Rule[]): GuardrailResult {
  const violations = rules
    .map(rule => rule(output))
    .filter((v): v is string => v !== null);
  return { passed: violations.length === 0, violations };
}

const noPII: Rule = (output) =>
  /\b\d{3}-\d{2}-\d{4}\b/.test(output) ? "Output contains SSN-like pattern" : null;
```

**ECC Integration:**

ECC implements guardrails through its hook system in `hooks/`. `PreToolUse` hooks validate tool calls before execution — `pre:write-edit:worktree-guard` prevents writes on protected branches. `PostToolUse` hooks run after execution for auto-formatting and checks. The `pre:edit-write:workflow-branch-guard` blocks workflow file edits on main. The `ECC_WORKFLOW_BYPASS` environment variable controls hook enforcement per `rules/ecc/development.md`. Security guardrails are codified in `rules/common/security.md` with mandatory pre-commit checks.

## When to Use

- When agents operate with autonomy and mistakes have consequences.
- When outputs must comply with policies (PII, content policy, format).
- When tool calls must be validated before execution (file paths, API parameters).

## When NOT to Use

- When the agent operates in a fully sandboxed environment with no side effects.
- When guardrails add latency that exceeds the safety benefit.
- When rules are too complex to codify and require human judgment.

## Anti-Patterns

- Guardrails that are too strict — blocking legitimate actions and frustrating users.
- Checking only outputs while ignoring inputs and tool arguments.
- No bypass mechanism for trusted contexts — guardrails should be configurable.

## Related Patterns

- [agentic/human-in-the-loop](human-in-the-loop.md) — human oversight for cases guardrails cannot handle.
- [agentic/reflection](reflection.md) — self-critique complements external guardrails.
- [agentic/tool-use](tool-use.md) — guardrails validate tool arguments and results.

## References

- Anthropic — Guardrails: https://docs.anthropic.com/en/docs/build-with-claude/prompt-caching
- NeMo Guardrails: https://github.com/NVIDIA/NeMo-Guardrails
- OWASP LLM Top 10: https://owasp.org/www-project-top-10-for-large-language-model-applications/
