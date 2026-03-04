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
01-agents/       Specialized subagents (architect, uncle-bob, planner, code-reviewer, ...)
02-commands/     Slash commands (/tdd, /plan, /code-review, ...)
03-skills/       Domain knowledge (tdd-workflow, security-review, backend-patterns, ...)
04-rules/        Always-follow guidelines (common/ + typescript/ + python/ + golang/)
05-hooks/        Trigger-based automations (hooks.json)
06-contexts/     Dynamic system prompt injection
07-mcp-configs/  MCP server configurations
08-examples/     CLAUDE.md templates for real-world stacks
09-docs/         Diagrams, guides, and reference documentation
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

## Development Notes

- Agent format: Markdown with YAML frontmatter (name, description, tools, model)
- Skill format: Markdown with clear sections for when to use, how it works, examples
- Hook format: JSON with matcher conditions and command/notification hooks
- File naming: lowercase with hyphens (e.g., `python-reviewer.md`, `tdd-workflow.md`)
