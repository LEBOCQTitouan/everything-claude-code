# Development Workflow

> This file extends [common/git-workflow.md](./git-workflow.md) with the full feature development process that happens before git operations.

The Feature Implementation Workflow describes the development pipeline: research, planning, TDD, code review, and then committing to git.

## Feature Implementation Workflow

0. **Research & Reuse** _(mandatory before any new implementation)_
   - **GitHub code search first:** Run `gh search repos` and `gh search code` to find existing implementations, templates, and patterns before writing anything new.
   - **Exa MCP for research:** Use `exa-web-search` MCP during the planning phase for broader research, data ingestion, and discovering prior art.
   - **Check package registries:** Search npm, PyPI, crates.io, and other registries before writing utility code. Prefer battle-tested libraries over hand-rolled solutions.
   - **Search for adaptable implementations:** Look for open-source projects that solve 80%+ of the problem and can be forked, ported, or wrapped.
   - Prefer adopting or porting a proven approach over writing net-new code when it meets the requirement.

1. **Plan First**
   - Use **planner** agent to create implementation plan
   - Generate planning docs before coding: PRD, architecture, system_design, tech_doc, task_list
   - Identify dependencies and risks
   - Break down into phases

2. **TDD Approach** _(commit at each transition)_
   - Use **tdd-guide** agent
   - Write tests first (RED) → **commit tests**
   - Implement to pass tests (GREEN) → **commit implementation**
   - Refactor (IMPROVE) → **commit refactor**
   - Verify 80%+ coverage

3. **Code Review**
   - Use **code-reviewer** agent immediately after writing code
   - Address CRITICAL and HIGH issues → **commit each fix**
   - Fix MEDIUM issues when possible → **commit each fix**

4. **Commit Continuously**
   - Commit after every logical change — see [git-workflow.md](./git-workflow.md) for cadence rules
   - Follow conventional commits format
   - Never accumulate uncommitted work across multiple concerns
   - See [git-workflow.md](./git-workflow.md) for commit message format and PR process
