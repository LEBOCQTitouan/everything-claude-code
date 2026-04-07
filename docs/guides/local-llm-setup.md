# Local LLM Setup Guide

Optional setup for offloading mechanical agent tasks to a local Ollama instance. Without this setup, ECC works identically — agents execute all tasks on hosted models.

## Prerequisites

- macOS, Linux, or WSL2
- ~4GB RAM for 7B model, ~8GB for 13B model
- Ollama installed

## Step 1: Install Ollama

```bash
# macOS
brew install ollama

# Linux
curl -fsSL https://ollama.ai/install.sh | sh
```

Start the Ollama service:
```bash
ollama serve
```

## Step 2: Pull Models

```bash
# 7B model for schema-fill tasks (cartography agents)
ollama pull mistral:7b-instruct

# 13B model for Mermaid generation + finding aggregation
ollama pull qwen2.5:14b-instruct
```

## Step 3: Add MCP Server

Install the ollama-mcp bridge (tested with version 0.3.x):

```bash
npm install -g @rawveg/ollama-mcp
```

Add to Claude Code:
```bash
claude mcp add ollama-mcp -- npx @rawveg/ollama-mcp
```

## Step 4: Enable in ECC

```bash
ecc config set local-llm.enabled true
ecc config set local-llm.provider ollama
ecc config set local-llm.base-url http://localhost:11434
ecc config set local-llm.model-small "mistral:7b-instruct"
ecc config set local-llm.model-medium "qwen2.5:14b-instruct"
```

## Step 5: Verify

```bash
# Check Ollama is running
curl http://localhost:11434/api/tags

# Check ECC config
cat ~/.ecc/config.toml
# Should show [local_llm] section

# Run a cartography agent — should log "local" path at DEBUG
```

## Troubleshooting

| Problem | Fix |
|---------|-----|
| "ollama_generate tool not found" | Run `claude mcp add ollama-mcp` |
| Agent falls back to hosted model | Check `ollama serve` is running |
| Slow generation | Normal for 7B (~20-60s), 13B (~60-120s) |
| Mermaid validation fails | 13B may need 2-3 retries; mmdc loop handles this |

## Kill Switch

Disable all local delegation instantly:
```bash
ecc config set local-llm.enabled false
```

Agents immediately revert to hosted model execution.

## Eligible Agents

| Agent | Model Tier | Delegated Subtask |
|-------|-----------|-------------------|
| cartographer | 7B (small) | Routing/dispatch from delta JSON |
| cartography-flow-generator | 7B (small) | Schema-fill of flow sections |
| cartography-journey-generator | 7B (small) | Schema-fill of journey sections |
| diagram-updater | 13B (medium) | Mermaid diagram generation |
| diagram-generator | 13B (medium) | Mermaid diagram generation |
| convention-auditor | 13B (medium) | Finding aggregation from grep |
