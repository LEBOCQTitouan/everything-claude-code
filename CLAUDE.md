# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A collection of production-ready agents, skills, hooks, commands, rules, and MCP configurations for software development using Claude Code. This fork enforces Hexagonal Architecture, DDD, and Clean Code principles.

## Running Tests

```bash
# Run all tests
node 11-tests/run-all.js

# Run individual test files
node 11-tests/lib/utils.test.js
node 11-tests/lib/package-manager.test.js
node 11-tests/hooks/hooks.test.js
```

## Architecture

```
01-docs/         Diagrams, guides, and reference documentation
02-examples/     CLAUDE.md templates for real-world stacks
03-agents/       Specialized subagents (architect, uncle-bob, planner, code-reviewer, ...)
04-commands/     Slash commands (/tdd, /plan, /code-review, ...)
05-skills/       Domain knowledge (tdd-workflow, security-review, backend-patterns, ...)
06-rules/        Always-follow guidelines (common/ + typescript/ + python/ + golang/)
07-hooks/        Trigger-based automations (hooks.json)
08-contexts/     Dynamic system prompt injection
09-mcp-configs/  MCP server configurations
10-scripts/      Cross-platform Node.js utilities for hooks and setup
11-tests/        Test suite for scripts and utilities
```

## Key Commands

- `/tdd` - Test-driven development workflow
- `/plan` - Implementation planning
- `/e2e` - Generate and run E2E tests
- `/code-review` - Quality review (includes uncle-bob Clean Code audit)
- `/build-fix` - Fix build errors
- `/learn` - Extract patterns from sessions
- `/skill-create` - Generate skills from git history
- `/update-codemaps` - Regenerate architectural codemaps
- `/claw` - Start NanoClaw persistent REPL
- `/evolve` - Analyze instincts and suggest evolved structures
- `/harness-audit` - Audit agent harness setup
- `/loop-start` - Start autonomous agent loop
- `/loop-status` - Check loop status
- `/quality-gate` - Run quality gate checks
- `/model-route` - Route to optimal model by task complexity

## Development Notes

- Agent format: Markdown with YAML frontmatter (name, description, tools, model)
- Skill format: Markdown with clear sections for when to use, how it works, examples
- Hook format: JSON with matcher conditions and command/notification hooks
- File naming: lowercase with hyphens (e.g., `python-reviewer.md`, `tdd-workflow.md`)
