# Spec: /comms-plan — Content Ideation & Scheduling Command

## Problem Statement

The comms pipeline can generate content (`/comms-generate`) and manage it (`/comms`), but there's no structured way to plan WHAT to create and WHEN. Content planning requires understanding the user's goal, audience, and timeline — then researching trending topics and generating a schedule. This needs a purpose-built interview process distinct from grill-me (which is for technical spec validation, not content strategy).

## Research Summary

- **Structured interview-as-conversation**: each answer guides the next question (not rigid questionnaire) — Pressmaster.ai pattern
- **4-step AI ideation workflow**: scrape trends → expand into ideas → cluster by theme → generate outlines
- **60-day planning horizon**: optimal for strategic planning with trend adaptability
- **Editorial calendar essentials**: what content, which channel, publish date, owner, status
- **78% of brands use interviews in content strategy** but only 31% use structured templates (HubSpot 2026)
- **Anti-pattern**: planning horizon <30 days → no trend adaptation; unmeasured ROI → no optimization

## Decisions

| # | Decision | Rationale | ADR? |
|---|----------|-----------|------|
| 1 | 4-stage interview (Goal → Audience → Channels → Timeline) | Research-backed, conversational, not adversarial | No |
| 2 | NOT grill-me | grill-me is for technical validation; content planning needs different structure | No |
| 3 | Web research after interview | Search for trends relevant to the stated goal | No |
| 4 | Output to comms/plans/ | Separate from drafts; plans are blueprints, drafts are content | No |
| 5 | Schedule as markdown table | Date, Channel, Idea, Status, Dependencies | No |

## User Stories

### US-001: /comms-plan Command

**As a** developer, **I want** `/comms-plan` to guide me through content planning with a structured interview and produce a publication schedule, **so that** I have a strategic plan before generating content.

#### Acceptance Criteria

- AC-001.1: Given `/comms-plan` is invoked, when the interview starts, then it asks the 4 stages in order: Goal → Audience → Channels → Timeline
- AC-001.2: Given each interview answer, when the next question is asked, then it adapts based on the previous answer (conversational, not rigid)
- AC-001.3: Given the interview completes, when web research runs, then it searches for trending topics relevant to the stated goal
- AC-001.4: Given research completes, when the plan is generated, then it contains per-channel content ideas with suggested dates
- AC-001.5: Given the plan is generated, when output is written, then it goes to `comms/plans/YYYY-MM-DD-{slug}.md`
- AC-001.6: Given the plan has cross-channel content, when dependencies exist, then the schedule shows dependency relationships
- AC-001.7: Given the command file, when inspected, then it has proper frontmatter and follows ECC command conventions

#### Dependencies
- None (builds on existing comms infrastructure)

## Affected Modules

| Module | Layer | Change Type |
|--------|-------|-------------|
| `commands/comms-plan.md` | Command | Create |
| `docs/commands-reference.md` | Docs | Modify |
| `CHANGELOG.md` | Docs | Modify |

## Constraints
- No Rust code — Markdown command only
- Interview is purpose-built (NOT grill-me)
- Web research via WebSearch subagent
- Output to comms/plans/, not comms/drafts/

## Non-Requirements
- Automated scheduling/publishing
- Calendar integration (Google Calendar, etc.)
- Recurring schedule generation
- Content performance tracking

## Implementation

Create `commands/comms-plan.md` with:

1. **Frontmatter**: description, allowed-tools (Read, Write, Bash, Grep, Glob, Agent, AskUserQuestion, WebSearch, TodoWrite)

2. **Interview Phase** (4 stages, one question at a time via AskUserQuestion):
   - **Stage 1 — Goal**: "What's your content goal?" with options (launch announcement, weekly engagement, awareness campaign, conference promotion, Other)
   - **Stage 2 — Audience**: Based on goal, suggest audiences. "Who are you targeting?" (developers, executives, community, general public)
   - **Stage 3 — Channels**: Based on audience, suggest channel priorities. "Which channels?" with ranked recommendations
   - **Stage 4 — Timeline**: "Planning horizon?" (1 week, 2 weeks, 1 month, 60 days)

3. **Research Phase**: Launch WebSearch subagent with queries derived from goal + audience + channels. Search for trending topics, competitor content, best practices.

4. **Plan Generation**: Produce a structured plan with:
   - Executive summary (goal, audience, channels, timeline)
   - Per-channel content ideas (3-5 ideas per channel)
   - Publication schedule table:
     ```
     | Date | Channel | Content Idea | Type | Status | Depends On |
     |------|---------|-------------|------|--------|------------|
     ```
   - Key messages / themes

5. **Output**: Write to `comms/plans/YYYY-MM-DD-{slug}.md` and update CALENDAR.md

## Doc Preview
- commands-reference.md: add /comms-plan entry
- CHANGELOG: add under v5.1.0
