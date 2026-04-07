# ECC Development Conventions

Meta-steering for how ECC agents, commands, skills, and hooks should be built. When working on this project, follow these conventions.

## Agent Conventions

- All agents with 4+ step workflows SHOULD use TodoWrite for tracking. Graceful degradation is implicit — if unavailable, proceed without tracking (do not repeat this inline)
- All agents that spawn subagents MUST specify `allowedTools` on every spawn
- All agents MUST have frontmatter: `name`, `description`, `tools`, `model`, `effort`
- Agent `effort` field (low, medium, high, max) should match model tier: haiku=low, sonnet=medium/high, opus=high/max
- Agents with cross-session concerns SHOULD have `memory: project`
- Read-only analysis agents MUST NOT have `Write` or `Edit` in tools

## Command Conventions

- All pipeline commands (`/spec-*`, `/design`, `/implement`) MUST use `EnterPlanMode`/`ExitPlanMode`
- All commands MUST have `allowed-tools` in frontmatter listing every tool they use
- Commit instructions MUST use "You MUST commit immediately" language — never "Commit:"
- All commands MUST persist artifacts to `docs/specs/` after adversarial review PASS

## Skill Conventions

- New skills MUST be under 500 words for v1
- Skill directory name MUST match the `name` field in frontmatter
- Skills MUST have frontmatter: `name`, `description`, `origin: ECC`
- Skills referencing external tools MUST include graceful degradation

## Hook Conventions

- All hooks MUST check `ECC_WORKFLOW_BYPASS` at the top: `[ "${ECC_WORKFLOW_BYPASS:-}" = "1" ] && exit 0`
- Hooks MUST use atomic writes via `mktemp` + `mv` for any file mutations
- Hook scripts MUST use `set -uo pipefail`

## Adversary Conventions

- Adversary verdicts MUST include rationale for every dimension evaluated
- Adversaries MUST have `skills: ["clean-craft"]` for quality reference
- Adversaries SHOULD have `memory: project` to detect recurring weaknesses

## Anti-Patterns

- DO NOT add new tools to agent frontmatter without justification
- DO NOT create commands without Plan Mode — user review is mandatory
- DO NOT skip TodoWrite because "it's a simple change"
- DO NOT use `Write` in agents that only analyze — use conversation output
