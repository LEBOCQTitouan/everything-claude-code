---
description: Content ideation and scheduling — structured interview to clarify goals, then web research and plan generation with per-channel ideas and publication schedule.
allowed-tools: Read, Write, Edit, Bash, Grep, Glob, Agent, AskUserQuestion, WebSearch, TodoWrite
---

# /comms-plan

Plan content strategy with structured interview, web research, and publication schedule. Produces a planning document. Use `/comms-generate` after for actual drafts.

> **Tracking**: TodoWrite checklist below. If unavailable, proceed without tracking.

TodoWrite: Phase 1-4 (interview, research, plan generation, output).

## Phase 1: Content Strategy Interview

Purpose-built conversational interview (NOT grill-me). Collaborative. One question per stage.

### Stage 1 — Goal
Ask: "What's your content goal?" Options: Launch announcement, Weekly developer engagement, Awareness campaign, Conference/event promotion, Other. Record as `interview.goal`.

### Stage 2 — Audience
Suggest audiences based on goal. Single primary audience. Record as `interview.audience`.

### Stage 3 — Channels
Based on audience, suggest channel priorities (multiSelect: true): Social media, Blog, Devblog, Documentation site. Record as `interview.channels`.

### Stage 4 — Timeline
Options: This week (7d), Next 2 weeks, This month (30d), 60-day plan (recommended). Record as `interview.timeline`.

## Phase 2: Trend Research

If WebSearch unavailable, skip with note. Derive 2-3 queries from goal + audience. Run searches. Summarize 3-5 bullets: trending topics, competitor patterns, hashtags, best practices.

## Phase 3: Plan Generation

### 1. Executive Summary
Table: Goal, Audience, Channels, Timeline, Generated date.

### 2. Key Themes
3-5 overarching narrative threads.

### 3. Per-Channel Ideas
3-5 ideas per channel: Title/Concept, Format, Key Talking Points (2-3), Estimated Effort.

### 4. Publication Schedule
Table spanning full horizon: Date, Channel, Content Idea, Type, Status (planned), Depends On. Rules: spread evenly, social teasers before blog, mark dependencies.

### 5. Research Insights
Phase 2 findings.

## Phase 4: Output

1. Generate slug from goal (lowercase, hyphenated, max 40 chars)
2. Write to `comms/plans/YYYY-MM-DD-{slug}.md` with YAML frontmatter (goal, audience, channels, timeline, status: draft, created)
3. Create `comms/plans/` if needed
4. Update `comms/CALENDAR.md` if exists (append entries, status: planned)
5. Display full plan inline

## Notes

- Planning document only — `/comms-generate` for actual drafts
- Interview is collaborative, not adversarial
- Web research optional but improves quality
- Plans go to `comms/plans/`, drafts to `comms/drafts/`
