# Agent Teams API Assessment — 2026-04-09

**BL-139** | **Checked at:** 2026-04-09 | **Re-check cadence:** quarterly | **Verdict: wait**

## Executive Summary

Claude Code Agent Teams is experimental (requires env flag), has 7x token cost vs subagents, no SDK API, and no nested teams. ECC's fire-and-return subagent model is architecturally incompatible with Agent Teams' persistent teammate model. **Recommendation: wait for GA + SDK support.**

## ECC Dispatch Surface → Agent Teams Mapping

| # | ECC Dispatch Surface | Current Implementation | Agent Teams Equivalent | Gap |
|---|---------------------|----------------------|----------------------|-----|
| 1 | Agent tool with `allowedTools` | `Agent` tool, `subagent_type` param, tool allowlist | Teammate spawned by team lead | Agent Teams has no `allowedTools` restriction per teammate — all teammates share full tool access |
| 2 | `isolation: "worktree"` on multi-PC waves | Git worktree per subagent, merged sequentially | Each teammate gets own context window | Context isolation yes, but no git worktree isolation — teammates share the same filesystem |
| 3 | `max-concurrent` from team manifest | `teams/implement-team.md` YAML frontmatter | Recommended 3-5 teammates | No configurable concurrency cap — implicit limit via team size |
| 4 | Fix-round budget retry pattern | Parent orchestrator tracks `fix_round_count`, re-dispatches tdd-executor | `TeammateIdle` hook (exit 2 to keep working) | TeammateIdle is coarser — no per-task retry counting, no budget cap |
| 5 | Sequential post-wave merge | Parent merges worktree branches in PC-ID order | No merge ordering guarantee | Teammates work on shared filesystem — file conflicts are the user's problem |

## Wave Dispatch Compatibility Analysis

### Worktree Isolation
Agent Teams does **not** provide worktree isolation. Each teammate operates in the same working directory. ECC's wave dispatch relies on git worktrees for parallel file edits without conflicts. **Incompatible.**

### Merge Ordering
Agent Teams has no merge ordering mechanism. ECC's post-wave merge is sequential in PC-ID order with conflict detection. Teammates would need to be manually sequenced or work on non-overlapping files. **Incompatible.**

### Regression Verification
ECC runs all prior wave commands after each wave merge. Agent Teams' shared task list provides status tracking but no regression orchestration. **tasks.md remains necessary** — Agent Teams tasks are ephemeral, not persisted.

### Batched Dispatch
ECC's batched tdd-executor (co-located PCs in single subagent) is incompatible with Agent Teams' one-context-per-teammate model. Each teammate gets a fresh context window — no batching optimization possible.

## tdd-executor Isolation Evaluation

### Context Isolation
Agent Teams provides context isolation per teammate (each gets own context window). This matches tdd-executor's requirement for fresh context. **Compatible.**

### Context Brief Protocol
tdd-executor receives a structured context brief via the Agent tool's `prompt` parameter. Agent Teams teammates receive instructions via the team lead's messages or the shared task list description. The context brief would need reformatting from a single prompt string to a task description + initial message. **Low migration cost.**

### Fix-Round Budget
The fix-round budget pattern (max 2 retries, then AskUserQuestion) is parent-owned. Agent Teams' `TeammateIdle` hook could partially replace this — exit 2 keeps the teammate working — but lacks:
- Per-task retry counting
- Budget cap enforcement
- User escalation via AskUserQuestion
**Not replaceable — ECC must retain budget logic.**

### Structured JSON Output
tdd-executor returns structured JSON (status, commit_count, files_changed, test_names). Agent Teams teammates communicate via mailbox messages (text). No structured return payload. **Incompatible — would require parsing teammate output.**

### Verdict: complement (not replace)
Agent Teams could provide context isolation, but ECC must retain: git worktree isolation, fix-round budget, structured output parsing, merge ordering. The models are complementary, not substitutional.

## Decision Gate: GA Trigger Conditions

All 5 conditions must be true before ECC integrates Agent Teams:

| # | Trigger Condition | Observable Signal |
|---|-------------------|-------------------|
| 1 | Env flag removed | `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` no longer required in docs |
| 2 | SDK API available | `claude-agent-sdk` Python/TS package exposes team creation programmatically |
| 3 | No breaking changes for 3+ releases | Agent Teams changelog shows only additive changes for 3 consecutive Claude Code releases |
| 4 | Nested teams supported | Documentation describes teammates spawning sub-teams (required for orchestrator → wave → tdd-executor hierarchy) |
| 5 | Per-teammate tool restrictions | `allowedTools` or equivalent configurable per teammate (required for security isolation) |

**Re-evaluation cadence:** Quarterly, or when Anthropic announces Agent Teams GA.

## Token Cost Analysis

| Model | Single Session | 3-Agent Team | Ratio |
|-------|---------------|--------------|-------|
| Typical task | ~50k tokens | ~350k tokens | 7x |
| ECC /implement | ~200k tokens (orchestrator + subagents) | ~1.4M tokens (team) | 7x |

Agent Teams' persistent teammate model means each teammate maintains full conversation history. ECC's fire-and-return subagents discard context after returning — fundamentally cheaper for focused tasks.

## Sources

- [Agent Teams Docs](https://code.claude.com/docs/en/agent-teams)
- [Agent SDK Overview](https://code.claude.com/docs/en/agent-sdk/overview)
- [Agent Teams Guide — claudefa.st](https://claudefa.st/blog/guide/agents/agent-teams)
- [MindStudio Analysis](https://www.mindstudio.ai/blog/claude-code-agent-teams-parallel-collaboration)
- [Claude Managed Agents — testingcatalog](https://www.testingcatalog.com/anthropic-launches-claude-managed-agents-for-businesses/)
- [Agent SDK Python — GitHub](https://github.com/anthropics/claude-agent-sdk-python)
- [Agent Teams Deep Dive — alexop.dev](https://alexop.dev/posts/from-tasks-to-swarms-agent-teams-in-claude-code/)
