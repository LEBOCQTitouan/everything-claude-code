---
description: Restate requirements, assess risks, create step-by-step plan, then execute with TDD after user confirmation. Supports feature, refactor, and security modes.
---

# Plan Command

**FIRST ACTION**: Call the `EnterPlanMode` tool immediately. This enters Claude Code plan mode which restricts tools to read-only exploration while you research and design the plan. After writing the plan, call `ExitPlanMode` to present it for user approval. Once the user approves, you exit plan mode and gain full tool access for TDD execution.

This command invokes the **planner** agent to create a comprehensive implementation plan, then executes it using TDD after user confirmation.

## Modes

- `/plan <description>` — Feature implementation (default)
- `/plan refactor <description>` — Safe refactoring workflow (architect → changes → tests)
- `/plan security <description>` — Security-focused workflow (security-reviewer → fixes → tests)

## What This Command Does

1. **Restate Requirements** - Clarify what needs to be built
2. **Identify Risks** - Surface potential issues and blockers
3. **Create Step Plan** - Break down implementation into phases with test targets
4. **E2E Assessment** - Determine if new E2E tests are needed based on scope
5. **Wait for Confirmation** - MUST receive user approval before proceeding
6. **Execute with TDD** - After confirmation, implement each phase using RED → GREEN → REFACTOR
7. **Run E2E Tests** - Write new E2E tests if flagged, then run full E2E suite

After all phases complete, run `/verify` for comprehensive quality and architecture review.

## When to Use

Use `/plan` when:
- Starting a new feature
- Making significant architectural changes
- Working on complex refactoring (`/plan refactor`)
- Security hardening (`/plan security`)
- Multiple files/components will be affected
- Requirements are unclear or ambiguous

## How It Works

### Feature Mode (default)

The planner agent will:

1. **Analyze the request** and restate requirements in clear terms
2. **Break down into phases** with specific, actionable steps
3. **Identify dependencies** between components
4. **Assess risks** and potential blockers
5. **Estimate complexity** (High/Medium/Low)
6. **Present the plan** and WAIT for your explicit confirmation

### Refactor Mode

Agent chain: **architect** → implementation → **tdd-guide** → verify

1. Architect agent analyzes current structure and proposes refactoring approach
2. Break into phases with test targets
3. After confirmation: TDD execution per phase
4. Handoff document between agents:

```markdown
## HANDOFF: [previous-agent] -> [next-agent]

### Context
[Summary of what was done]

### Findings
[Key discoveries or decisions]

### Files Modified
[List of files touched]

### Open Questions
[Unresolved items for next agent]

### Recommendations
[Suggested next steps]
```

### Security Mode

Agent chain: **security-reviewer** → fixes → **tdd-guide** → verify

1. Security reviewer audits codebase for vulnerabilities
2. Break findings into prioritized fix phases
3. After confirmation: TDD execution per fix phase

## Example Usage

```
User: /plan I need to add real-time notifications when markets resolve

Agent (planner):
# Implementation Plan: Real-Time Market Resolution Notifications

## Requirements Restatement
- Send notifications to users when markets they're watching resolve
- Support multiple notification channels (in-app, email, webhook)
- Ensure notifications are delivered reliably
- Include market outcome and user's position result

## Implementation Phases

### Phase 1: Database Schema
- Add notifications table with columns: id, user_id, market_id, type, status, created_at
- Add user_notification_preferences table for channel preferences
- Create indexes on user_id and market_id for performance

### Phase 2: Notification Service
- Create notification service in lib/notifications.ts
- Implement notification queue using BullMQ/Redis
- Add retry logic for failed deliveries
- Create notification templates

### Phase 3: Integration Points
- Hook into market resolution logic (when status changes to "resolved")
- Query all users with positions in market
- Enqueue notifications for each user

### Phase 4: Frontend Components
- Create NotificationBell component in header
- Add NotificationList modal
- Implement real-time updates via Supabase subscriptions
- Add notification preferences page

## Dependencies
- Redis (for queue)
- Email service (SendGrid/Resend)
- Supabase real-time subscriptions

## Risks
- HIGH: Email deliverability (SPF/DKIM required)
- MEDIUM: Performance with 1000+ users per market
- MEDIUM: Notification spam if markets resolve frequently
- LOW: Real-time subscription overhead

## Estimated Complexity: MEDIUM

**WAITING FOR CONFIRMATION**: Proceed with this plan? (yes/no/modify)
```

## Important Notes

**CRITICAL**: The planner agent will **NOT** write any code until you explicitly confirm the plan with "yes" or "proceed" or similar affirmative response.

If you want changes, respond with:
- "modify: [your changes]"
- "different approach: [alternative]"
- "skip phase 2 and do phase 3 first"

## Execution Mode (Post-Confirmation)

After the user confirms the plan, execute automatically using TDD:

### TDD Execution Loop

For each phase in the approved plan:

#### 1. SCAFFOLD
- Read the phase's **Test Targets** from the plan
- Create interface/type stubs that `throw new Error('Not implemented')` or return obviously wrong values
- This gives tests something to import

#### 2. RED — Write Failing Tests
- Write unit tests and integration tests listed in the phase's Test Targets
- Include happy path, edge cases, and error scenarios
- Run the test command — **verify tests FAIL** for the right reason (not import errors)
- If tests do not fail: fix the scaffold (ensure stubs throw or return wrong values)
- Commit: `test: add <phase> tests`

#### 3. GREEN — Implement Minimal Code
- Write the minimal implementation to make all tests pass
- Run the test command — **verify tests PASS**
- Run the build command — **verify build passes**
- Commit: `feat: implement <phase>`

#### 4. REFACTOR — Improve Code
- Improve naming, extract constants, reduce duplication
- Run tests again — **verify tests still PASS**
- If no meaningful refactoring needed, skip this step
- Commit: `refactor: improve <phase>`

#### 5. GATE — Phase Complete
- Run the full test suite (not just this phase's tests)
- Run the build command
- Tag checkpoint: `git tag checkpoint/<phase-name>` (lightweight, for rollback reference)
- If either fails: **STOP and fix before proceeding to the next phase**
- If both pass: proceed to the next phase

### Coverage Targets

| Code Type | Target |
|-----------|--------|
| Critical paths (financial, auth, security) | 100% |
| Public API surface | 90%+ |
| General code | 80%+ |

### E2E Testing

After all phases complete:

1. Check the plan's **E2E Assessment** section
2. **If new E2E tests are needed**: write them now using the e2e-runner agent, targeting the scenarios listed in the plan. Commit: `test: add E2E tests for <feature>`
3. **Run the full E2E suite** (existing + newly written). If failures: fix before proceeding.

### Mandatory Code Review

After all phases and E2E tests pass, run `/verify` which invokes the `code-reviewer` agent on the full diff:

1. Address all CRITICAL and HIGH issues — commit each fix
2. Address MEDIUM issues when possible — commit each fix
3. Architecture review runs automatically as part of `/verify`

### Progress Tracking

During execution, track progress for each phase:

```
Phase 1: Database Schema
  [x] SCAFFOLD — interfaces created
  [x] RED — 5 tests written, all failing
  [x] GREEN — implementation passes all tests
  [x] REFACTOR — extracted constants
  [x] GATE — full suite passes, checkpoint tagged

Phase 2: Notification Service
  [x] SCAFFOLD — interfaces created
  [ ] RED — writing tests...
```

### Handling Failures

- **Tests don't fail in RED**: Fix the scaffold — stubs must throw or return wrong values
- **Tests don't pass in GREEN**: Debug implementation, do not modify tests (unless tests are wrong)
- **Build fails**: Use `/build-fix` to resolve, then re-run gate
- **Full suite regresses**: A previous phase broke — fix before continuing
- **Context window running low**: For plans with 5+ phases, suggest executing in batches

## Commit Cadence

Each phase produces up to 3 commits following the TDD cycle:

1. `test: add <phase> tests` — after RED (failing tests written)
2. `feat: implement <phase>` — after GREEN (tests pass)
3. `refactor: improve <phase>` — after REFACTOR (if changes made)

Never accumulate changes across multiple plan phases without committing.

## TDD Best Practices

**DO:**
- Write the test FIRST, before any implementation
- Run tests and verify they FAIL before implementing
- Write minimal code to make tests pass
- Refactor only after tests are green
- Add edge cases and error scenarios
- Aim for 80%+ coverage (100% for critical code)

**DON'T:**
- Write implementation before tests
- Skip running tests after each change
- Write too much code at once
- Ignore failing tests
- Test implementation details (test behavior)
- Mock everything (prefer integration tests)

## Integration with Other Commands

- Use `/build-fix` if build errors occur during execution
- Use `/verify` after plan completes for comprehensive review (code + architecture)
- Use `/e2e` for standalone E2E test generation

## Related Agents

This command invokes:
- `planner` agent — plan generation
- `tdd-guide` agent — TDD execution per phase
- `e2e-runner` agent — E2E test writing and execution
- `architect` agent — refactor mode analysis
- `security-reviewer` agent — security mode audit
