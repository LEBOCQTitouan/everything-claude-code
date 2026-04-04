# ADR 0049: Stop Event over PostToolUse for Cost Tracking

## Status

Accepted

## Context

BL-096 needs to capture token usage data from Claude Code sessions. Two hook events were considered: PostToolUse (fires after each tool call) and Stop (fires after each Claude response turn).

## Decision

Use the Stop event for cost tracking, not PostToolUse.

## Consequences

- Stop event carries `usage.input_tokens` and `usage.output_tokens` in its JSON payload — billing data is available
- PostToolUse payload contains tool input/output but no token usage metrics — cannot track costs
- Granularity is per-response-turn, not per-tool-call — acceptable for cost analytics
- The existing `cost_tracker` hook already uses Stop, so this is backward compatible
- Agent type (`agent_type` field) is available in the Stop payload for per-agent attribution
