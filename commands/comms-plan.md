---
description: Content ideation and scheduling — structured interview to clarify goals, then web research and plan generation with per-channel ideas and publication schedule.
allowed-tools: Read, Write, Edit, Bash, Grep, Glob, Agent, AskUserQuestion, WebSearch, TodoWrite
---

# /comms-plan

Plan your content strategy with a structured interview, web trend research, and a publication schedule. This command produces a planning document — not generated content. Use `/comms-generate` after planning to produce actual drafts.

## What This Command Does

> **Tracking**: Create a TodoWrite checklist. If TodoWrite is unavailable, proceed without tracking.

TodoWrite items:
- "Phase 1: Content strategy interview (4 stages)"
- "Phase 2: Trend research via WebSearch"
- "Phase 3: Plan generation"
- "Phase 4: Write output files"

Mark each item complete as the phase finishes.

---

## Phase 1: Content Strategy Interview

This is a purpose-built conversational interview — NOT grill-me. It is collaborative and adaptive, not adversarial. Ask one question per stage and wait for the answer before proceeding to the next stage.

### Stage 1 — Goal

Ask via AskUserQuestion:

> "What's your content goal?"

Options:
- Launch announcement
- Weekly developer engagement
- Awareness campaign
- Conference/event promotion
- Other

Record the answer as `interview.goal`.

### Stage 2 — Audience

Based on `interview.goal`, suggest relevant audiences. Ask via AskUserQuestion:

> "Who is your target audience?"

Adapt the options based on the goal:
- If goal = **launch announcement** → developers, technical decision-makers, open source community, general tech enthusiasts
- If goal = **weekly developer engagement** → existing users, contributors, broader developer community, technical leads
- If goal = **awareness campaign** → CTOs/engineering managers, developers discovering the tool, technical bloggers/influencers
- If goal = **conference/event promotion** → conference attendees, speakers, developer community, potential sponsors
- If goal = **other** → offer: developers, business decision-makers, general public, technical community

Options should be `multiSelect: false` (single primary audience). Always include "Other" as a fallback.

Record the answer as `interview.audience`.

### Stage 3 — Channels

Based on `interview.audience`, suggest channel priorities. Ask via AskUserQuestion:

> "Which channels do you want to prioritize?"

Options (multiSelect: true):
- Social media (X/LinkedIn/Mastodon/Bluesky)
- Blog
- Devblog
- Documentation site

Recommend based on audience:
- If audience = **developers / contributors** → Devblog first, then social media, then blog
- If audience = **technical decision-makers / CTOs** → Blog first, then social media
- If audience = **general public / awareness** → Social media first, then blog
- If audience = **conference/event** → Social media first, then blog, then devblog

Record the answer as `interview.channels` (list).

### Stage 4 — Timeline

Ask via AskUserQuestion:

> "What's your planning horizon?"

Options:
- This week (7 days)
- Next 2 weeks
- This month (30 days)
- 60-day plan (Recommended for strategic content)

Record the answer as `interview.timeline`. Note the recommended option is 60 days for strategic depth.

---

## Phase 2: Trend Research

After the interview completes, launch a web research subagent to find trending topics relevant to the stated goal.

If WebSearch is unavailable, skip this phase and add a note in the plan: "Trend research unavailable — WebSearch not accessible."

Research steps:

1. **Derive 2-3 search queries** from `interview.goal` + `interview.audience`. Examples:
   - If goal = "launch v5.0" → "developer tool launch announcement best practices 2026", "open source release marketing developer community"
   - If goal = "weekly engagement" → "developer community engagement content strategy 2026", "devrel weekly content ideas"
   - If goal = "awareness campaign" → "developer tool awareness marketing 2026", "technical content marketing trends"

2. **Launch a WebSearch subagent** with the derived queries. Collect:
   - Trending topics in the goal's domain
   - Competitor or peer content patterns (format, cadence, themes)
   - Relevant hashtags and keywords
   - Best practices for the selected channels

3. **Summarize research** into 3-5 bullet points to include in the plan under "Research Insights."

---

## Phase 3: Plan Generation

Generate a structured content plan containing these sections:

### 1. Executive Summary

| Field | Value |
|-------|-------|
| Goal | `interview.goal` |
| Audience | `interview.audience` |
| Channels | `interview.channels` (comma-separated) |
| Timeline | `interview.timeline` |
| Generated | today's date |

### 2. Key Themes

Derive 3-5 overarching themes from the goal and research insights. These are the narrative threads that run across all content. Example themes:
- "Reliability and developer trust" (for a launch)
- "Community growth and contribution" (for engagement)

### 3. Per-Channel Ideas

For each channel in `interview.channels`, generate 3-5 specific content ideas. Each idea includes:

| Field | Description |
|-------|-------------|
| Title/Concept | Short descriptive title |
| Format | thread, article, tutorial, reference doc, changelog post, etc. |
| Key Talking Points | 2-3 bullet points of what to cover |
| Estimated Effort | low / medium / high |

### 4. Publication Schedule

Generate a schedule table spanning the full planning horizon, distributing content across channels with logical pacing and dependency awareness.

```markdown
| Date | Channel | Content Idea | Type | Status | Depends On |
|------|---------|--------------|------|--------|------------|
| YYYY-MM-DD | social | Launch teaser thread | thread | planned | — |
| YYYY-MM-DD | blog | Feature deep-dive | article | planned | social teaser |
| YYYY-MM-DD | devblog | Technical walkthrough | tutorial | planned | blog article |
```

Rules for scheduling:
- Spread content evenly across the timeline — avoid clustering
- If a blog article announces a feature, schedule social teasers 1-2 days before
- Mark dependencies explicitly in the "Depends On" column
- All entries start with status `planned`

### 5. Research Insights

Include the summarized findings from Phase 2. If research was skipped, note why.

---

## Phase 4: Output

1. **Generate a slug** from the goal. Lowercase, hyphenated, max 40 characters.
   - "Launch announcement for v5.0" → `v5-launch-announcement`
   - "Weekly developer engagement" → `weekly-developer-engagement`
   - "Awareness campaign Q2" → `awareness-campaign-q2`

2. **Determine the output path**: `comms/plans/YYYY-MM-DD-{slug}.md` using today's date.

3. **Check for `comms/` directory**. If it does not exist, create `comms/plans/` directory structure:
   ```bash
   mkdir -p comms/plans
   ```

4. **Write the plan** to `comms/plans/YYYY-MM-DD-{slug}.md` with YAML front-matter:
   ```yaml
   ---
   goal: <interview.goal>
   audience: <interview.audience>
   channels: [<interview.channels>]
   timeline: <interview.timeline>
   status: draft
   created: YYYY-MM-DD
   ---
   ```

5. **Update CALENDAR.md** if it exists at `comms/CALENDAR.md`:
   - Append each scheduled entry from the publication schedule table
   - Use status `planned` for all new entries
   - Do not overwrite existing entries

6. **Display the full plan** inline in the conversation for immediate review.

---

## Notes

- This command produces a **planning document**, not generated content. Use `/comms-generate` to produce actual drafts based on this plan.
- The interview is **conversational and collaborative** — adapt wording naturally based on what the user has told you.
- Web research is **optional** — the plan is generated with or without it, but research improves idea quality.
- All plan files go to `comms/plans/` — separate from `comms/drafts/` which holds generated content.
- No Plan Mode — this is a guided workflow, not a pipeline with design review.
