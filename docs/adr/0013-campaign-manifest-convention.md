# 0013. Campaign Manifest Convention

Date: 2026-03-22

## Status

Accepted

## Context

The ECC pipeline commands (`/spec-*`, `/design`, `/implement`) relied on conversation context as the primary state carrier for intermediate artifacts: grill-me interview answers, agent analysis outputs, adversary verdict history, draft specs/designs, commit SHA accumulators, and detected toolchain commands. If a session was interrupted or context compacted mid-pipeline, this state was lost and had to be regenerated. The `/implement` command had been hardened with tasks.md persistence (BL-030) and subagent isolation (BL-031), but spec and design commands remained fragile.

The design principle: if losing context would lose progress, the system is fragile. All state should be externalized to disk so clearing context is always free. This supplements (not replaces) the existing memory system — campaign captures work-item-specific state, memory captures cross-project preferences.

## Decision

Introduce a campaign manifest (`campaign.md`) per work item that indexes all artifacts, decisions, and progress. The campaign manifest:

1. **Bootstraps** in `.claude/workflow/campaign.md` at Phase 0 (when toolchain detection completes), then moves to `docs/specs/YYYY-MM-DD-<slug>/` after the spec directory is created
2. **Contains**: Status, Artifacts table, Toolchain, Grill-Me Decisions, Adversary History, Agent Outputs, Commit Trail, and Resumption Pointer sections
3. **Updates incrementally**: each grill-me answer, adversary round, agent output, and commit appends a row
4. **Is written by** the parent orchestrator via Claude's Write tool (not shell hooks), centralized in the `spec-pipeline-shared` skill
5. **Parallel writes**: during wave-based TDD, only the parent orchestrator writes to campaign.md (never subagents)

The `campaign_path` is stored in `state.json` alongside `spec_path`, `design_path`, and `tasks_path`.

## Consequences

**Positive:**
- Fresh agents can orient from one file read — no directory exploration needed
- Session interruption at any pipeline phase loses zero state
- Decision trail (grill-me answers, adversary verdicts) is preserved across sessions
- Conversation becomes a presentation layer only — all truth lives on disk
- `strategic-compact` can recommend mid-pipeline compaction as safe

**Negative:**
- Campaign.md adds a file per work item (~100-200 lines at completion)
- Every pipeline command gains additional write instructions
- The `spec-pipeline-shared` skill grows with campaign lifecycle sections
- Testing relies on structural grep checks rather than behavioral E2E tests
