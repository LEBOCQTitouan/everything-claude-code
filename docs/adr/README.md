# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for the ECC project.

## Format

Each ADR follows the structure:

- **Status**: Accepted, Superseded, or Deprecated
- **Context**: The forces at play, including technical, business, and social
- **Decision**: The change we are making
- **Consequences**: What becomes easier or harder as a result

## Numbering

ADRs are numbered sequentially: `0001-short-title.md`, `0002-short-title.md`, etc.

## Index

| ADR | Title | Status |
|-----|-------|--------|
| [0001](0001-hexagonal-architecture.md) | Hexagonal Architecture with 6 Crates | Accepted |
| [0002](0002-hook-based-state-machine.md) | Hook-Based State Machine for Workflows | Accepted |
| [0003](0003-manifest-based-dev-mode.md) | Manifest-Based Dev Mode Switching | Accepted |
| [0004](0004-native-tooling-standard.md) | Native Tooling Standard | Accepted |
| [0005](0005-file-based-memory-system.md) | File-Based Memory System | Accepted |
| [0006](0006-doc-first-spec-driven-pipeline.md) | Doc-First Spec-Driven Pipeline | Accepted |
| [0007](0007-subagent-isolation-for-tdd.md) | Subagent Isolation for TDD | Accepted |
| [0008](0008-designs-directory-convention.md) | Designs Directory Convention | Accepted |
| [0009](0009-phase-summary-convention.md) | Phase Summary Convention | Accepted |
| [0010](0010-skill-frontmatter-validation.md) | Skill Frontmatter Validation | Accepted |
| [0011](0011-command-narrative-convention.md) | Command Narrative Convention | Accepted |
| [0012](0012-wave-parallel-tdd.md) | Wave-Based Parallel TDD Execution | Accepted |
| [0013](0013-campaign-manifest-convention.md) | Campaign Manifest Convention | Accepted |
| [0014](0014-context-aware-graceful-exit.md) | Context-Aware Graceful Exit Convention | Superseded |
| [0037](0037-workflow-templates-content-type.md) | Workflow Templates Content Type | Accepted |
| [0038](0038-scaffold-command-distribution.md) | Scaffold Command Distribution | Accepted |
| [0039](0039-cargo-dist-adoption.md) | cargo-dist Adoption for Binary Distribution | Accepted |
| [0040](0040-cosign-custom-job.md) | Cosign Signing as Custom Post-Build Job | Accepted |
| [0041](0041-session-detection-pattern.md) | Session Detection Pattern for Backlog Filtering | Accepted |
| [0042](0042-context-pre-hydration.md) | Context Pre-Hydration Pattern | Accepted |
| [0042](0042-lazy-worktree-write-guard.md) | Lazy Worktree via Write-Guard Pattern | Accepted |
| [0043](0043-conditional-rule-loading.md) | Conditional Rule Loading via applies-to Frontmatter | Accepted |
| [0044](0044-cartography-index-full-regeneration.md) | Cartography INDEX.md Full Regeneration | Accepted |
| [0045](0045-effort-based-thinking-enforcement.md) | Effort-Based Thinking Enforcement | Accepted |
| [0046](0046-effort-to-tokens-mapping.md) | Effort-to-Tokens Mapping | Accepted |
| [0047](0047-sqlite-over-jsonl-for-cost.md) | SQLite over JSONL for Cost Storage | Accepted |
| [0048](0048-separate-cost-db.md) | Separate Cost Database from Logs Database | Accepted |
| [0049](0049-stop-event-for-cost-tracking.md) | Stop Event over PostToolUse for Cost Tracking | Accepted |
| [0052](0052-cartography-hook-to-doc-orchestrator.md) | Move Cartography Delta Processing to Doc-Orchestrator | Accepted |
| [0053](0053-handler-trait-dispatch.md) | Handler Trait for Hook Dispatch | Accepted |
| [0054](0054-post-merge-worktree-auto-deletion.md) | Post-Merge Worktree Auto-Deletion with Safety Checks | Accepted |
| [0055](0055-auditable-workflow-bypass.md) | Auditable Workflow Bypass | Accepted |
| [0056](0056-ecc-workflow-bypass-deprecation.md) | Deprecation of ECC_WORKFLOW_BYPASS | Completed |
| [0057](0057-release-automation-tooling.md) | Release Automation Tooling | Accepted |
| [0058](0058-composite-design-reviewer.md) | Composite Design Reviewer | Accepted |
| [0059](0059-backlog-repository-port.md) | Promote Backlog to Full Hexagonal Concern with Port Traits | Accepted |
| [0060](0060-declarative-tool-manifest.md) | Declarative Tool Manifest (BL-146) | Accepted |
| [0061](0061-grill-me-foundation-mode.md) | Grill-Me Foundation-Mode for Project-Level Documents | Accepted |
| [0062](0062-aaif-alignment-stance.md) | AAIF Alignment Stance — Additive Alignment | Accepted |
| [0063](0063-pc-self-evaluation.md) | Post-PC Self-Evaluation Architecture | Accepted |
| [0064](0064-party-command.md) | Party Command Design Decisions (BL-144) | Accepted |
| [0065](0065-tribal-knowledge-docs.md) | Tribal Knowledge Documentation Upgrade (BL-152) | Accepted |
