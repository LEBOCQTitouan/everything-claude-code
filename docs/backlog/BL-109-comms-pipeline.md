# BL-109: Autonomous communication pipeline — multi-channel content generation from code

- **Status:** open
- **Scope:** EPIC
- **Target:** /spec
- **Created:** 2026-03-30
- **Tags:** comms, marketing, content-generation, devrel, multi-channel

## Raw Idea

Create a full communication pipeline that generates marketing and developer relations content from the codebase. The pipeline should support 4 channels (social media, blog, devblog, doc site) with per-channel strategies, produce file-based output (never auto-publish), and be a general-purpose tool with ECC as the first user. Content lives in a separate git repo; when invoked inside a code repo, comms artifacts are gitignored and pushed to the comms repo. Implemented as an agent + skill combo within ECC.

## Grill-Me Decisions

1. **Channels:** Social media (X, LinkedIn, Mastodon, Bluesky), blog, devblog, documentation site — each with predefined per-channel strategies
2. **Autonomy:** Nothing auto-published. All output is file-based (markdown, text). Optional draft-push to destination APIs (Buffer, Typefully, etc.) but always as draft. User copies/pastes or approves drafts manually
3. **Code-awareness:** Hybrid — code-aware generation for technical content (release notes, changelogs, feature highlights via git history + source analysis), templates for brand/marketing content
4. **Scope:** General-purpose tool, ECC is the first user (dogfooding)
5. **Sensitivity:** Auto-redaction of secrets/internal URLs as safety net, plus manual user review before publishing
6. **Cadence:** User-initiated only — no scheduled generation
7. **Integration:** Pluggable adapter architecture — self-contained core producing plain files, optional adapters for external tools
8. **MVP approach:** Scaffold all 4 channels with basic content generation across all, then deepen each
9. **Form factor:** Agent + skill combo (not a standalone binary or slash command)

## Web Research Findings

### Content Generation from Code
- **git-cliff** (Rust-native): structured changelog generation from conventional commits — ideal adapter for the code-aware layer
- **release-please** / **semantic-release**: automated release notes and version bumps
- **Claude API**: LLM-based transformation of changelogs into platform-specific content (tweets, articles, announcements)
- Proven pattern: `git tag → git-cliff → Claude API → platform-specific drafts`

### Static Site Generators
- **Astro Starlight**: purpose-built for docs, Markdown/MDX, fast, accessible
- **Hugo**: fastest builds, massive theme library, Go-based CLI
- **Astro**: island architecture, top SSG for 2025-2026, blog + docs hybrid

### Social Media APIs (Pluggable Adapters)
- **Buffer API**: broadest platform support (X, LinkedIn, FB, Instagram, Mastodon), simple REST, free tier
- **Typefully API**: X-focused, thread-native, developer-friendly
- **Mastodon API** + `toot` CLI: open protocol, fully scriptable
- **Bluesky AT Protocol**: open protocol, growing dev audience, multiple CLIs
- **Postiz**: open-source self-hosted social scheduler with API

### DevRel Automation
- **Postiz** (open-source): self-hosted social scheduling, API-driven
- **Crowd.dev** (open-source): community management, GitHub/Discord/Slack integration
- No dominant "devrel-as-code" tool exists — most teams roll custom pipelines

## Optimized Prompt

```
/spec

Build a general-purpose communication pipeline as an ECC agent + skill combo. The pipeline generates developer relations and marketing content from codebases, targeting 4 channels: social media (X, LinkedIn, Mastodon, Bluesky), blog, devblog, and documentation site.

**Architecture:**
- Hexagonal design with ports for content generation, code analysis, and destination adapters
- Core engine reads git history, changelogs, README, and source code to generate technical content (code-aware via git-cliff integration + Claude API)
- Template system for brand/marketing content with per-channel strategy definitions
- Pluggable adapter layer for destination APIs (Buffer, Typefully, Mastodon, Bluesky AT Protocol, Postiz)
- Auto-redaction pass scans all generated content for secrets, internal URLs, and private references before output

**Output model:**
- All content written to files (markdown, text) — never auto-published
- Comms artifacts live in a dedicated git repo (configurable path)
- When invoked inside a code repo, comms directory is gitignored; pipeline pushes to the comms repo
- Optional draft-push to destination APIs (always as draft, never published)
- Per-channel strategy files define tone, format, hashtags, audience, and posting cadence

**Agents and skills:**
- `comms-generator` agent: orchestrates content generation across channels
- `comms-strategy` skill: per-channel strategy definitions and content templates
- `comms-adapter` skill: pluggable destination adapter patterns (file, Buffer API, Typefully API, AT Protocol, Mastodon API)
- `comms-redactor` skill: secret/URL scanning and redaction rules

**MVP (v1):** Scaffold all 4 channels with basic content generation. Code-aware pipeline: git-cliff changelog → Claude API transformation → per-channel formatted drafts. File output only, no destination adapters yet.

**User-initiated only** — no scheduled generation, no auto-publishing. ECC is the first user (dogfooding).
```

## Dependencies

- None blocking (standalone feature)
- Benefits from: BL-091 (diagnostics/tracing for pipeline observability), BL-092 (structured logs), BL-093 (memory system for learning content preferences over time)
