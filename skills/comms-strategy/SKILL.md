---
name: comms-strategy
description: Per-channel content strategy definitions for the comms pipeline. Defines tone, format, audience, and templates for social media, blog, devblog, and documentation channels.
origin: ECC
---

# Comms Strategy — Per-Channel Content Guidelines

## Channel Definitions

### Social Media
- **Platforms**: X (280 chars), LinkedIn (3000 chars), Mastodon (500 chars), Bluesky (300 chars)
- **Tone**: Concise, engaging, developer-friendly. Use technical terms but explain briefly.
- **Format**: Single post or thread. Include relevant hashtags. Link to blog/docs for details.
- **Audience**: Developers, DevOps engineers, AI practitioners
- **Cadence**: Per release or milestone

### Blog
- **Format**: 800-1500 word article with code examples, diagrams, and clear sections
- **Tone**: Educational, detailed, thought-leadership
- **Audience**: Technical decision-makers, senior developers
- **Structure**: Problem → Solution → Implementation → Results

### Devblog
- **Format**: 300-800 word technical deep-dive with code snippets
- **Tone**: Direct, implementation-focused, peer-to-peer
- **Audience**: Contributors, power users, framework developers
- **Structure**: What changed → Why → How to use it → Migration notes

### Documentation Site
- **Format**: Reference-style with examples, API signatures, configuration options
- **Tone**: Precise, scannable, task-oriented
- **Audience**: Users following docs to accomplish a task

## Strategy File Schema

Each channel has a strategy file in `comms/strategies/{channel}.md`:

```yaml
channel: social
platforms: [x, linkedin, mastodon, bluesky]
tone: concise, developer-friendly
max_length: {x: 280, linkedin: 3000, mastodon: 500, bluesky: 300}
hashtags: ["#rustlang", "#devtools", "#AI"]
audience: developers
```
