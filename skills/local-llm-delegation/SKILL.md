---
name: local-llm-delegation
description: MCP delegation pattern for offloading mechanical agent tasks to local Ollama models. Covers when to use, prompt template, fallback logic, and model selection.
origin: ECC
---

# Local LLM Delegation via Ollama MCP

Pattern for agents with `local-eligible: true` frontmatter to delegate mechanical subtasks to a local Ollama instance via the `ollama_generate` MCP tool.

## When to Use

Only for agents performing **mechanical** tasks with zero reasoning requirement:
- Schema filling from structured input (cartography generators)
- Mermaid syntax generation from structured data (diagram agents)
- Tool-output aggregation with threshold rules (convention auditor)

**Never use for:** adversarial review, architecture decisions, security analysis, code review, planning, or any task requiring judgment.

## Delegation Flow

1. Check if `ollama_generate` MCP tool is available
2. If available: call `ollama_generate` with the subtask prompt and specified model
3. Validate the output (schema check, syntax check, format check)
4. If output passes validation: use it
5. If output fails validation: retry once with a corrective prompt
6. After 2 failed attempts: fall back to doing the work on the hosted model (self-execution)
7. Log which path was taken (local/hosted) at DEBUG level

## Subtask Documentation

Each eligible agent MUST document which specific subtask is delegated:
- **cartographer**: routing/dispatch decision from delta JSON
- **cartography-flow-generator**: schema-fill of flow markdown sections
- **cartography-journey-generator**: schema-fill of journey markdown sections
- **diagram-updater**: Mermaid diagram block generation
- **diagram-generator**: Mermaid diagram block generation
- **convention-auditor**: finding aggregation from grep output

## Model Selection

| Task Type | Model Tier | Config Key | Default |
|-----------|-----------|------------|---------|
| Schema fill, routing | small (7B) | `model_small` | `mistral:7b-instruct` |
| Mermaid generation, aggregation | medium (13B) | `model_medium` | `qwen2.5:14b-instruct` |

## Prompt Template

```
You are a {role}. Generate {output_type} from the following input.

Input:
{structured_input}

Output requirements:
{schema_description}

Output ONLY the {output_type}, no explanation.
```

## Fallback Behavior

- **Ollama unavailable** (MCP tool not found): agent executes task itself on hosted model. No error.
- **Ollama returns error**: log WARN, fall back to hosted model.
- **Output fails validation (2x)**: log WARN with truncated output sample, fall back to hosted model.
- **Kill switch**: if `ecc config set local-llm.enabled false`, all delegation is skipped.

## Offline Testing

When `ollama_generate` tool is unavailable (no Ollama installed), agents execute their fallback path. This is the normal mode for users who haven't set up local LLM. No error, no degradation — the agent just does the work itself as it did before BL-128.
