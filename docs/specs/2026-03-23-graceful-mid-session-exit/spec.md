# Spec: Graceful Mid-Session Exit + Implement Context Clear (BL-055, BL-054)

## Problem Statement

Long-running ECC pipeline commands (`/implement`, `/audit-full`) can exhaust the context window mid-phase, causing output quality degradation. Currently there is no mechanism to detect context pressure, save state, and exit cleanly. Additionally, the `/implement` command starts with leftover spec/design conversation context that wastes headroom for the most context-intensive phase. With the campaign manifest (BL-035) externalizing all state to disk, session interruption should be zero-cost — but only if the agent actually saves state and exits before quality degrades.

## Research Summary

- COLLAPSE.md pattern: graduated thresholds (65% warn, 75% checkpoint, 85% critical exit) for AI agent context management
- LangGraph uses node-level checkpointing (before/after each graph node) for workflow resumability
- Best practice: always write to temp file first, then rename (atomic writes prevent corruption)
- Agent self-assessment is common for Markdown-instruction-based agents, but precise data via side-channel is more reliable
- Claude Code's statusline receives `context_window.used_percentage` via stdin JSON — proven data path for the side-channel
- PreToolUse/PostToolUse hooks do NOT receive context_window data — only statusline and Stop hooks do

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Statusline side-channel for context detection | PreToolUse hooks don't receive context data; statusline is the only continuous source of `used_percentage` | Yes |
| 2 | Two thresholds: 75% warn, 85% mandatory exit | Research shows graduated approach; 75% gives user control, 85% prevents quality degradation | Yes (same ADR) |
| 3 | Session-scoped temp file (`$TMPDIR/ecc-context-$SESSION_ID.pct`) | Auto-cleaned on reboot, no collision between parallel sessions, no git pollution | Yes (same ADR) |
| 4 | Scope to /implement and /audit-full only | These are the heaviest context consumers; spec/design delegate to subagents and are shorter-lived | No |
| 5 | Bundle BL-054 (planned clear at /implement start) | Shares infrastructure (context reading, campaign writes); eliminates duplicate design work | No |
| 6 | Full re-entry for /audit-full | Persist domain results, re-entry skips completed domains. Novel state management pattern creates precedent | Yes (ADR 0014) |
| 7 | Shared graceful-exit skill | Prevents god-utility; skill defines protocol, each command owns state-save logic | No |
| 8 | Warn at 75% = display + continue; Exit at 85% = save + inform + STOP | 75% warning is advisory (user can /compact); 85% is mandatory save-and-exit | No |
| 9 | Thresholds are fixed constants for v1 | Two-threshold system validated by research; configurability deferred to avoid premature abstraction. Env var override can be added later if needed | No |
| 10 | 95% hard ceiling — immediate STOP | Prevents unbounded logical units from defeating the exit mechanism. Partial state dump with warning | No |

## User Stories

### US-001: Context percentage side-channel

**As a** pipeline command, **I want** to read the current context window usage percentage from a file, **so that** I can make informed decisions about when to checkpoint or exit.

#### Acceptance Criteria

- AC-001.1: Given the statusline script receives JSON with `context_window.used_percentage`, when the script renders, then it also writes the percentage to `$TMPDIR/ecc-context-<session_id>.pct`
- AC-001.2: Given the temp file exists, when a command runs `cat $TMPDIR/ecc-context-<session_id>.pct`, then it returns a numeric percentage (integer, 0-100)
- AC-001.3: Given no statusline is configured (file doesn't exist), when a command checks for the file, then it gracefully degrades (treats context as unknown, proceeds without checking)
- AC-001.4: Given two parallel Claude Code sessions, when both run with different session IDs, then each writes to its own temp file with no collision
- AC-001.5: Given a session ends, when the temp directory is cleaned (reboot or manual), then no stale files remain
- AC-001.6: Given the statusline writes the temp file, session identity uses `$CLAUDE_SESSION_ID` with fallback to `$PPID`. Directory uses `${TMPDIR:-/tmp}`. (Consistent with `suggest-compact.js` pattern.)
- AC-001.7: Given the statusline writes the temp file, it uses `mktemp` in the same directory followed by `mv` to the target path (atomic write, prevents partial reads)
- AC-001.8: Given the temp file contains non-numeric or out-of-range (outside 0-100) data, when a command reads it, then it treats the value as "unknown" and proceeds without checking

#### Dependencies

- Depends on: none

### US-002: Graceful-exit skill (shared protocol)

**As a** command author, **I want** a shared skill defining the context-check protocol, state-dump contract, and exit message format, **so that** all commands integrate graceful exit consistently.

#### Acceptance Criteria

- AC-002.1: Given the skill exists at `skills/graceful-exit/SKILL.md`, when a command references it, then it has access to: context-check bash snippet, threshold constants (75% warn, 85% exit), state-dump contract, exit message template, and re-entry guidance
- AC-002.2: Given the skill defines a context-check bash snippet, when a command runs it at a breakpoint, then it returns the current context percentage (or "unknown" if the side-channel file is missing)
- AC-002.3: Given the skill defines a warn threshold (75%), when the check returns >= 75% and < 85%, then the command displays: "Context at XX%. Consider running /compact or finishing the current phase." and continues
- AC-002.4: Given the skill defines an exit threshold (85%), when the check returns >= 85%, then the command MUST: (a) complete the current logical unit, (b) write state to campaign.md, (c) display exit message, (d) STOP
- AC-002.5: Given the skill defines a state-dump contract, when a graceful exit triggers, then campaign.md's Resumption Pointer is updated with mid-phase granularity. For `/implement`: `Current step: Phase 3 — Wave 2 complete (PC-001, PC-002 done)` / `Next action: Resume TDD at PC-003 (wave 3)`. For `/audit-full`: `Current step: Phase 2 — domains complete: architecture, security, testing` / `Next action: Resume domain audits: conventions, errors, observability, documentation; then correlation + report`. Partial dir path included for audit.
- AC-002.6: Given the skill defines an exit message, when exit triggers, then the message includes: context percentage, campaign file path, and the exact command to re-run
- AC-002.7: Given the side-channel file is missing (no statusline configured), when the context-check runs, then it returns "unknown" and the command proceeds without checking (graceful degradation)
- AC-002.8: Given context reaches 95% before the current logical unit completes, when the hard ceiling is hit, then the command STOPs immediately with a partial-state dump and warning: "Context at 95% (hard ceiling). State may be incomplete. Check campaign.md and tasks.md before resuming."
- AC-002.9: Given the skill defines a context-check script, then the script is a standalone executable at `skills/graceful-exit/context-check.sh`, independently testable with a mock temp file

#### Dependencies

- Depends on: US-001
- Runtime dependency: BL-035 (campaign manifest) — implemented

### US-003: /implement context clear gate (BL-054)

**As a** developer starting `/implement`, **I want** the command to offer a context clear before starting TDD, **so that** I begin with maximum context headroom and no spec/design conversation noise.

#### Acceptance Criteria

- AC-003.1: Given `/implement` enters Phase 0, when state validation passes, then the command reads the context percentage from the side-channel file
- AC-003.2: Given context percentage is available, when Phase 0 completes, then the command displays: "Context is at XX%. Implementation starts from disk artifacts." and offers AskUserQuestion: "Clear context and start fresh? (Plan at `<design_path>` will be loaded.)" with options ["Clear and restart (Recommended)", "Continue in current context"]
- AC-003.3: Given the user selects "Clear and restart", then the command instructs: "Run `/compact` then re-run `/implement`. All state is on disk. After compacting, read `campaign.md` Resumption Pointer for re-entry context." and STOPs
- AC-003.4: Given the user selects "Continue in current context", then the command proceeds normally (no force)
- AC-003.5: Given context percentage is unknown (side-channel file missing), then the gate is skipped silently — no question asked

#### Dependencies

- Depends on: US-001, US-002

### US-004: /implement graceful exit at breakpoints

**As a** developer running `/implement` on a large feature, **I want** the command to check context pressure between TDD waves and phases, **so that** it exits cleanly before quality degrades.

#### Acceptance Criteria

- AC-004.1: Given `/implement` completes a TDD wave and regression verification passes, when the next wave is about to start, then a context checkpoint runs per the graceful-exit skill
- AC-004.2: Given `/implement` transitions between phases (Phase 3->4, 4->5, 5->6, 6->7), when the current phase completes, then a context checkpoint runs
- AC-004.3: Given a checkpoint returns >= 85%, when the current logical unit (wave, phase) has completed, then the command: (a) updates tasks.md with current progress, (b) updates campaign.md Resumption Pointer with last completed wave/PC and next action, (c) displays exit message, (d) STOPs
- AC-004.4: Given a checkpoint returns >= 75% and < 85%, when the check runs, then the command displays the warning message and continues
- AC-004.5: Given a checkpoint returns < 75% or "unknown", when the check runs, then execution continues silently
- AC-004.6: Given a new session runs `/implement` after graceful exit, when it reads campaign.md and tasks.md, then it resumes from the exact PC/wave where the previous session stopped (verifies existing Phase 0 re-entry logic works with the new Resumption Pointer data written by graceful exit)
- AC-004.7: Given exit triggers between wave completion and the next wave, when regression verification has passed for the completed wave, then exit proceeds (never mid-regression)
- AC-004.8: Given the 85% exit triggers without a prior 75% warning (context jumped between checkpoints), then the exit still proceeds normally — the 85% threshold is independent of the 75% warning

#### Dependencies

- Depends on: US-002

### US-005: /audit-full graceful exit with domain-level re-entry

**As a** developer running a full codebase audit, **I want** the command to check context pressure between domain audit completions and resume only incomplete domains on re-entry, **so that** partial audit work is not wasted.

#### Acceptance Criteria

- AC-005.1: Given `/audit-full` is executing Phase 2 (Domain Audits), when a batch of domain agents completes, then a context checkpoint runs
- AC-005.2: Given a checkpoint returns >= 85%, when the current batch of agents has finished, then the command: (a) writes completed domain results to `docs/audits/partial-<timestamp>/`, (b) records which domains completed in campaign.md, (c) displays exit message, (d) STOPs
- AC-005.3: Given a new session runs `/audit-full` after graceful exit, when it reads campaign.md and the partial results directory, then it re-runs only the incomplete domains and the correlation/report phases
- AC-005.4: Given all domains completed but correlation triggers exit, when domain findings are on disk, then re-entry runs only the correlation and report phases
- AC-005.5: Given a checkpoint returns >= 75% and < 85%, then a warning is displayed and the audit continues
- AC-005.6: Given no prior partial results exist (first run or clean re-run), then the audit runs all domains normally
- AC-005.7: Given Phase 2 dispatches domain agents via the `audit-orchestrator` agent, when a checkpoint evaluates, then the **orchestrator** runs the context-check after each individual domain agent returns (not the audit-full command). In-flight agents are allowed to finish before exit.
- AC-005.8: Given `/audit-full` completes successfully (all domains + correlation + report), then all `docs/audits/partial-*/` directories from prior graceful exits are removed
- AC-005.9: Given a graceful exit, when the partial directory is created, then the exact path is recorded in campaign.md's Resumption Pointer. On re-entry, that path is the authoritative source for completed domain results.
- AC-005.10: Given campaign.md does not exist when `/audit-full` graceful exit triggers, then completed domain results are still written to `docs/audits/partial-<timestamp>/`, a minimal resumption file is written to `.claude/workflow/audit-resume.md` listing completed domains, the partial directory path, and the re-run command, and the exit message references this file instead of campaign.md

#### Dependencies

- Depends on: US-002
- Runtime dependency: BL-035 (campaign manifest) — implemented

### US-006: Documentation and glossary updates

**As a** developer or contributor, **I want** the graceful exit pattern documented in the glossary, ADR, and related skills, **so that** the pattern is discoverable and existing guidance reflects the new capability.

#### Acceptance Criteria

- AC-006.1: Given the glossary at `docs/domain/glossary.md`, when updated, then it contains entries for "Graceful Exit" and "Context Checkpoint"
- AC-006.2: Given ADR 0014 is created, then it documents: the two-threshold design, statusline side-channel mechanism, and the decision to scope to /implement + /audit-full
- AC-006.3: Given the strategic-compact skill, when updated, then the Compaction Decision Guide includes: "If context reaches 85%, the command saves state and exits automatically (graceful-exit skill)"
- AC-006.4: Given the campaign-manifest skill, when updated, then the Incremental Updates section documents Resumption Pointer updates at graceful-exit checkpoints
- AC-006.5: Given CHANGELOG.md, when updated, then it contains a BL-055 + BL-054 entry

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `statusline/statusline-command.sh` | Infrastructure (adapter) | Add side-channel file write |
| `skills/graceful-exit/SKILL.md` | Domain knowledge (new skill) | Create with protocol, thresholds, templates |
| `skills/graceful-exit/context-check.sh` | Infrastructure (script) | Standalone context-check script |
| `commands/implement.md` | Driving adapter (command) | Add context clear gate + breakpoint checkpoints |
| `commands/audit-full.md` | Driving adapter (command) | Add re-entry logic (read partial results, skip completed domains) |
| `agents/audit-orchestrator.md` | Agent (orchestrator) | Add context checkpoint after each domain agent return, partial results write, campaign.md update |
| `skills/strategic-compact/SKILL.md` | Domain knowledge (skill) | Update with graceful-exit reference |
| `skills/campaign-manifest/SKILL.md` | Domain knowledge (skill) | Update Incremental Updates section |
| `docs/domain/glossary.md` | Documentation | Add 2 entries |
| `docs/adr/0014-context-aware-graceful-exit.md` | Documentation | Create ADR |
| `CHANGELOG.md` | Documentation | Add entry |

## Constraints

- Statusline side-channel is best-effort: if no statusline configured, commands proceed without context checking (graceful degradation)
- Exit only after current logical unit completes — never mid-wave, mid-PC, or mid-regression (unless 95% hard ceiling)
- Campaign manifest (BL-035) must be active for state dumps; if campaign.md doesn't exist, write minimal state to `.claude/workflow/`
- The side-channel temp file MUST use atomic writes (write to temp, then rename)
- No Rust crate changes — entirely content-layer
- If BL-053 (poweruser statusline) has been implemented, add the side-channel write to that script. The side-channel write is additive and script-agnostic

## Non-Requirements

- `/spec-dev`, `/spec-fix`, `/spec-refactor` context checking (shorter-lived, subagent-heavy)
- `/design` context checking (shorter-lived)
- `/verify` context checking (fast command)
- Precise token counting or estimation from character counts
- Automatic context clearing (user must manually `/compact` or start new session)
- Real-time context display in commands (statusline already handles this)
- Configurable thresholds (fixed for v1, env var override deferred)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| Statusline adapter | Modify (side-channel write) | New file I/O path from statusline to commands |
| /implement command | Modify (context gate + checkpoints) | New exit path mid-implementation |
| /audit-full + orchestrator | Modify (checkpoints + re-entry) | New exit path + resume path for audits |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New concept | Domain | `docs/domain/glossary.md` | Add "Graceful Exit", "Context Checkpoint" |
| New decision | ADR | `docs/adr/0014-context-aware-graceful-exit.md` | Create |
| Feature entry | Project | `CHANGELOG.md` | Add BL-055 + BL-054 entry |
| Skill update | Skill | `skills/strategic-compact/SKILL.md` | Add graceful exit as backstop |
| Skill update | Skill | `skills/campaign-manifest/SKILL.md` | Document checkpoint-triggered Resumption Pointer updates |
| ADR index | Index | `docs/adr/README.md` | Add ADR 0014 |

## Open Questions

None — all resolved during grill-me interview and adversarial review.
