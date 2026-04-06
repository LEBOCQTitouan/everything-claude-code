---
name: human-in-the-loop
category: agentic
tags: [agentic, safety, approval, oversight]
languages: [python, typescript, rust]
difficulty: beginner
---

## Intent

Ensure safety and correctness by requiring human approval at critical decision points in an agent's execution, preventing irreversible or high-risk actions without oversight.

## Problem

Autonomous agents can take destructive actions (deleting data, deploying to production, sending communications) that are difficult or impossible to reverse. Full autonomy creates unacceptable risk for high-stakes operations. Agents may also misinterpret ambiguous instructions.

## Solution

Identify critical decision points and insert approval gates. The agent pauses execution, presents the proposed action with context to a human, and waits for approval, rejection, or modification. Non-critical actions proceed automatically. The approval surface is configurable per task type.

## Language Implementations

### Python

```python
from dataclasses import dataclass
from enum import Enum

class Decision(Enum):
    APPROVE = "approve"
    REJECT = "reject"
    MODIFY = "modify"

@dataclass(frozen=True)
class ApprovalRequest:
    action: str
    context: str
    risk_level: str

def requires_approval(action: str, risk_actions: frozenset[str]) -> bool:
    return action in risk_actions

async def human_gate(request: ApprovalRequest) -> Decision:
    print(f"[APPROVAL REQUIRED] {request.action}")
    print(f"Context: {request.context}")
    print(f"Risk: {request.risk_level}")
    response = input("approve/reject/modify: ").strip().lower()
    return Decision(response)
```

### Typescript

```typescript
interface ApprovalRequest {
  readonly action: string;
  readonly context: string;
  readonly riskLevel: "low" | "medium" | "high" | "critical";
}

type Decision = "approve" | "reject" | "modify";

function requiresApproval(
  action: string, riskActions: ReadonlySet<string>
): boolean {
  return riskActions.has(action);
}

async function humanGate(request: ApprovalRequest): Promise<Decision> {
  // In practice: send to UI, Slack, or CLI prompt
  return await promptUser(request);
}
```

### ECC Integration

ECC implements human-in-the-loop through Plan Mode. The `/spec`, `/design`, and `/implement` commands enter Plan Mode via `EnterPlanMode`/`ExitPlanMode` (required per `rules/ecc/development.md`), pausing for user review before execution. The `hooks/` system uses `PreToolUse` hooks to gate dangerous operations — `pre:write-edit:worktree-guard` blocks writes on main branch. The `grill-me` skill in `skills/grill-me/` uses `AskUserQuestion` for structured one-question-at-a-time interviews.

## When to Use

- When actions are irreversible (deployments, data deletion, financial transactions).
- When the agent's task is ambiguous and benefits from human clarification.
- When regulatory or compliance requirements mandate human oversight.

## When NOT to Use

- When the action is low-risk and easily reversible.
- When latency requirements preclude waiting for human input.
- When the approval surface is so broad that every action requires approval (approval fatigue).

## Anti-Patterns

- Requiring approval for every action — causes approval fatigue and slows execution.
- Not providing sufficient context in the approval request — humans cannot make informed decisions.
- No timeout mechanism — agent blocks indefinitely waiting for approval.

## Related Patterns

- [agentic/guardrails](guardrails.md) — automated safety checks complement human oversight.
- [agentic/planning](planning.md) — review the plan upfront to reduce mid-execution approvals.
- [agentic/tool-use](tool-use.md) — gate specific tools behind approval.

## References

- Anthropic — Human-in-the-loop patterns: https://docs.anthropic.com/en/docs/build-with-claude/tool-use#human-in-the-loop
- NIST AI Risk Management Framework: https://www.nist.gov/artificial-intelligence/ai-risk-management-framework
