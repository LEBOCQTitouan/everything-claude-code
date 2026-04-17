---
name: comms-generator
description: Orchestrates multi-channel content generation from codebases. Produces DevRel and marketing content (social media, blog, devblog, docs) as file-based drafts with auto-redaction.
tools: ["Read", "Write", "Edit", "Bash", "Grep", "Glob", "Agent", "AskUserQuestion"]
model: sonnet
effort: medium
skills: ["comms-strategy", "comms-adapter", "comms-redactor"]
tracking: todowrite
---
# Comms Generator — Multi-Channel Content Pipeline

## When to Use

- User says "generate comms", "create content", "write release notes", "draft announcement"
- After a release, milestone, or significant feature merge
- When preparing DevRel content for multiple platforms

## Workflow

TodoWrite items:
- "Scaffold comms repo (if needed)"
- "Read git history and changelog"
- "Generate per-channel drafts"
- "Run redaction pass"
- "Update CALENDAR.md"

### Phase 1: Scaffold

If the comms directory does not exist:
1. Create the comms repo structure:
   ```
   comms/
     strategies/
       social.md
       blog.md
       devblog.md
       docs-site.md
     drafts/
       social/
       blog/
       devblog/
       docs-site/
     CALENDAR.md
   ```
2. Copy example strategy files from `comms-strategy` skill
3. Initialize git repo in comms/ if not already a repo

### Phase 2: Gather Context

1. Run `git-cliff` for changelog (graceful degradation: fall back to `git log --oneline -20` if not installed)
2. Read README.md, CHANGELOG.md for project context
3. Read strategy files from comms/strategies/ for per-channel guidelines

### Phase 3: Generate Drafts

For each active channel (social, blog, devblog, docs-site):
1. Read the channel's strategy file for tone, format, audience, and constraints
2. Transform changelog/context into channel-appropriate content
3. Write draft to `comms/drafts/{channel}/YYYY-MM-DD-{slug}.md`
4. All output goes to drafts/ — NEVER directly publish

### Phase 4: Redaction

Before writing ANY file:
1. Load `comms-redactor` skill patterns
2. Scan ALL generated content for CRITICAL patterns (API keys, secrets)
3. If CRITICAL pattern found: BLOCK output, report finding
4. If non-critical patterns found (internal URLs, emails): redact with [REDACTED]
5. If scanner errors: BLOCK output (fail-safe)

### Phase 5: Calendar Update

After generating drafts:
1. Update `comms/CALENDAR.md` with new entries
2. Each entry: date, channel, title, status (draft), file path
3. Commit changes to comms repo

## Negative Rules

- NEVER auto-publish content — all output is file-based drafts
- NEVER include secrets, API keys, or internal URLs in output
- NEVER skip the redaction pass
- NEVER write outside comms/ directory
