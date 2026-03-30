# Spec Preview: Autonomous Comms Pipeline (BL-109)

## How This Spec Was Understood

The user wants a general-purpose communication pipeline implemented as an ECC agent + skill combo that generates DevRel and marketing content from codebases. It targets 4 channels (social media, blog, devblog, documentation site), outputs files only (never auto-publishes), uses the agent itself as the LLM (no external API calls), and includes auto-redaction of secrets. MVP: all 4 channels scaffolded with git-cliff (graceful degradation) → agent transformation → per-channel formatted drafts. Content lives in a separate comms git repo with a CALENDAR.md tracker.

## Spec Draft

# Spec: Autonomous Comms Pipeline (BL-109)

## Problem Statement

ECC has no structured way to generate developer relations or marketing content from its codebase. Release notes, social media announcements, blog posts, and documentation updates are created manually and inconsistently. No AI coding tool offers a native content generation pipeline that transforms code changes into multi-channel communication artifacts. This is an opportunity to build a general-purpose tool (ECC as first user) that automates the DevRel content lifecycle from changelog to channel-specific drafts.

## Research Summary

- **social-changelog** (GitHub): AI-powered tool converting GitHub release notes into social posts — validates the git-to-social pipeline pattern
- **git-cliff**: Rust-native changelog generator from conventional commits — ideal for the code-aware layer, graceful degradation to `git log` if not installed
- **Multi-channel orchestration**: 2026 pattern — scrapers + curators + generators publishing to X, LinkedIn, Mastodon, YouTube simultaneously
- **Voice consistency**: 85% of marketers use AI for content (2026); successful teams use template-driven generation with brand voice guardrails and human review gates
- **Multimodal emerging**: Tools evolving beyond text to video scripts and interactive media (defer for v1)
- **Anti-pattern**: Over-reliance on single channel; multi-channel orchestration with per-platform format is essential
- **Fail-safe redaction**: Defense in depth — agent redaction + mandatory human review before publish

## Decisions Made

| # | Decision | Rationale | ADR? |
|---|----------|-----------|------|
| 1 | All 4 artifacts (agent + 3 skills) | User wants full structure even for MVP | No |
| 2 | Parameterized from start | General-purpose tool, ECC provides example strategy files | No |
| 3 | Strategy files in comms repo | Project-specific config, skill ships templates scaffolded on init | No |
| 4 | git-cliff graceful degradation | Fall back to `git log` if not installed | No |
| 5 | Agent IS the LLM | No external Claude API calls needed | No |
| 6 | Mandatory draft stage | All output → drafts/, user approval to finalize | No |
| 7 | Content calendar (CALENDAR.md) | Track generated drafts with dates and status | No |
| 8 | Agent-driven scaffolding | No CLI subcommand — agent scaffolds on first run | No |
| 9 | Comms repo isolation | Gitignored staging + separate git repo for artifacts | Yes |

## User Stories

### US-001: Comms Generator Agent

**As a** developer, **I want** a `comms-generator` agent that orchestrates multi-channel content generation from my codebase, **so that** I can produce DevRel content without manual writing.

#### Acceptance Criteria

- AC-001.1: Given a code repo with conventional commits, when the agent is invoked, then it reads git history (via git-cliff or git log fallback) and produces channel-specific drafts
- AC-001.2: Given no comms repo exists, when the agent is first invoked, then it scaffolds the comms directory structure (strategies/, drafts/{channel}/, CALENDAR.md)
- AC-001.3: Given strategy files exist in the comms repo, when generating content, then the agent follows per-channel tone, format, and audience guidelines
- AC-001.4: Given generated content, when writing output, then all drafts go to drafts/{channel}/ — never directly published
- AC-001.5: Given the agent frontmatter, when inspected, then it has name, description, tools (Read, Write, Bash, Grep, Glob, Agent), model (sonnet), skills list
- AC-001.6: Given a 4+ step workflow, when executing, then the agent uses TodoWrite with graceful degradation

#### Dependencies
- None

### US-002: Per-Channel Strategy Skill

**As a** developer, **I want** per-channel strategy definitions with tone, format, audience, and templates, **so that** generated content matches each platform's requirements.

#### Acceptance Criteria

- AC-002.1: Given the `comms-strategy` skill, when loaded, then it defines strategy schemas for social, blog, devblog, and docs channels
- AC-002.2: Given example strategy files, when scaffolded to comms repo, then each channel has: tone, format, audience, hashtags/tags, max length, and example template
- AC-002.3: Given a social media strategy, when generating, then output respects character limits (280 for X, 3000 for LinkedIn, 500 for Mastodon/Bluesky)
- AC-002.4: Given the skill file, when inspected, then it is under 500 words with proper frontmatter (name, description, origin: ECC)

#### Dependencies
- None

### US-003: Destination Adapter Skill

**As a** developer, **I want** pluggable destination adapter patterns documented, **so that** future integrations (Buffer, Typefully, AT Protocol) have a clear contract.

#### Acceptance Criteria

- AC-003.1: Given the `comms-adapter` skill, when loaded, then it documents the file-output adapter (MVP) and interface contracts for future adapters
- AC-003.2: Given the file adapter, when invoked, then drafts are written as markdown files to drafts/{channel}/YYYY-MM-DD-{slug}.md
- AC-003.3: Given the skill, when inspected, then it lists future adapter candidates (Buffer, Typefully, Mastodon API, AT Protocol) with API patterns

#### Dependencies
- None

### US-004: Secret Redaction Skill

**As a** developer, **I want** auto-redaction of secrets and internal URLs in generated content, **so that** public-facing content never leaks sensitive information.

#### Acceptance Criteria

- AC-004.1: Given the `comms-redactor` skill, when loaded, then it defines patterns for API keys (sk-, ghp_, AKIA), connection strings, bearer tokens, internal URLs, private IPs, and email addresses
- AC-004.2: Given generated content containing a CRITICAL pattern (API key), when redaction runs, then output is BLOCKED and the finding is reported
- AC-004.3: Given generated content with an internal URL, when redaction runs, then the URL is replaced with [REDACTED-URL]
- AC-004.4: Given redaction scanning fails, when error occurs, then output is blocked (fail-safe — not silently passed)
- AC-004.5: Given trufflehog or gitleaks on PATH, when redaction runs, then the external tool is used; otherwise falls back to regex patterns

#### Dependencies
- None

### US-005: Content Calendar

**As a** developer, **I want** a CALENDAR.md tracking generated drafts, **so that** I can see what's been generated, reviewed, and published.

#### Acceptance Criteria

- AC-005.1: Given the comms repo is scaffolded, when inspected, then CALENDAR.md exists with a table header
- AC-005.2: Given content is generated, when the agent writes a draft, then CALENDAR.md is updated with date, channel, title, status (draft), and file path
- AC-005.3: Given a user approves a draft, when moved to published/, then CALENDAR.md status is updated to "published"

#### Dependencies
- Depends on: US-001

### US-006: Comms Repo Isolation

**As a** developer, **I want** comms output isolated in a separate git repo, **so that** generated content doesn't pollute my code repository.

#### Acceptance Criteria

- AC-006.1: Given a code repo, when comms/ directory is created, then it is gitignored
- AC-006.2: Given a comms repo path is configured, when the agent generates content, then it writes to the comms repo and commits
- AC-006.3: Given no comms repo exists, when the agent scaffolds, then it initializes a new git repo at the configured path
- AC-006.4: Given comms output in a code repo, when `git status` is run, then comms/ files do not appear

#### Dependencies
- Depends on: US-001

## Affected Modules

| Module | Layer | Change Type |
|--------|-------|-------------|
| `agents/comms-generator.md` | Agent | Create |
| `skills/comms-strategy/SKILL.md` | Skill | Create |
| `skills/comms-adapter/SKILL.md` | Skill | Create |
| `skills/comms-redactor/SKILL.md` | Skill | Create |
| `.gitignore` | Config | Modify (add comms/) |

## Constraints

- No Rust code changes — entirely Markdown agents and skills
- All output is file-based — never auto-publish
- Mandatory draft stage — user approval required before finalizing
- Redaction is fail-safe — block on error, don't pass through
- Skills must be under 500 words for v1
- Agent must follow ECC conventions (frontmatter, TodoWrite, allowedTools)
- git-cliff graceful degradation to git log

## Non-Requirements

- External API integrations (Buffer, Typefully, etc.) — documented patterns only
- Scheduled/automated generation — user-initiated only
- Multimodal content (video, images) — text/markdown only
- CLI subcommand (`ecc comms`) — agent-driven only
- Encryption of comms artifacts
- Multi-user collaboration on comms repo

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | No Rust changes | No E2E tests needed |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New agent | CLAUDE.md | Agents section | Mention comms-generator |
| ADR | docs/adr/ | New | Comms repo isolation pattern |
| Changelog | CHANGELOG.md | Entry | Add comms pipeline entry |
| Glossary | docs/domain/glossary.md | Terms | Channel, Strategy, Redaction, Draft |

## Open Questions

None — all resolved during grill-me interview.

## Doc Preview

### CLAUDE.md changes
No changes needed — agents are auto-discovered via install. Could add a one-liner to the Project Overview mentioning comms capability.

### Other doc changes
- One ADR: comms repo isolation (gitignored staging + separate repo)
- CHANGELOG entry
- 4 glossary terms
