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
   - Use **planner** agent via `/plan` to create implementation plan
   - Plan includes test targets per phase and E2E assessment
   - Identify dependencies and risks
   - Break down into phases
   - **Wait for user confirmation before executing**

2. **TDD Approach** _(automatic after plan confirmation)_
   - `/plan` executes TDD per phase after confirmation
   - Write tests first (RED) → **commit tests**
   - Implement to pass tests (GREEN) → **commit implementation**
   - Refactor (IMPROVE) → **commit refactor**
   - Verify 80%+ coverage
   - Gate: full test suite + build must pass between phases

3. **E2E Testing** _(after all phases)_
   - Plan's E2E assessment determines if new E2E tests are needed
   - If yes: write E2E tests for flagged scenarios → **commit E2E tests**
   - Run full E2E suite (existing + new)

4. **Verify** _(mandatory before shipping)_
   - Run `/verify` for comprehensive quality gate: build + tests + lint + code review + architecture review
   - Address CRITICAL and HIGH issues → **commit each fix**
   - Fix MEDIUM issues when possible → **commit each fix**

5. **Commit Continuously**
   - Commit after every logical change — see [git-workflow.md](./git-workflow.md) for cadence rules
   - Follow conventional commits format
   - Never accumulate uncommitted work across multiple concerns
   - See [git-workflow.md](./git-workflow.md) for commit message format and PR process
