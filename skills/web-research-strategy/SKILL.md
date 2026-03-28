---
name: web-research-strategy
description: Reusable search strategy patterns for LLM-powered web research. Defines query formulation rules, source triangulation, source weighting, channel diversity, and query templates per category.
origin: ECC
---

# Web Research Strategy

Standardized patterns for effective web research using LLM-powered search agents. Apply these patterns when using WebSearch or exa-web-search to find technology findings, upgrade opportunities, or ecosystem intelligence.

## When to Apply

- Running `/audit-web` parallel search agents
- Using `/spec-dev` to research prior art and existing solutions
- Any agent that executes multiple web searches and needs reliable, cited findings

## Query Formulation Rules

### Rule 1: Short Keyword Queries

Short keyword queries outperform natural language questions for web search.

```
WRONG:  "What are the best practices for hexagonal architecture in Rust in 2026?"
CORRECT: "hexagonal architecture Rust 2026 best practices"

WRONG:  "What are the new features in tokio version 2?"
CORRECT: "tokio 2.0 new features migration 2026"
```

### Rule 2: Year-Anchored Queries

Anchor queries to the current year (or current year -1) to filter out stale results. Undated results are low-confidence.

```
WRONG:  "Rust async runtime comparison"
CORRECT: "Rust async runtime comparison 2025 2026"
```

Always append the current year to technology-specific queries. If a query yields only old results (pre-2024), note the finding as potentially stale.

### Rule 3: Pseudo-Answer Generation (pseudo-answer technique)

Before executing a search, generate a hypothetical ideal answer. Extract the most specific keywords from that hypothesis and use them as the query.

**Example:**
- Research goal: "Are there newer alternatives to the `tracing` crate for structured logging in Rust?"
- Hypothetical answer: "Yes, `tracing` 0.2 added structured span fields, and `opentelemetry-rust` now integrates natively. There's also `fastrace` for low-overhead tracing."
- Extracted keywords: `tracing 0.2 structured fields`, `opentelemetry-rust 2026`, `fastrace Rust tracing alternative`
- Resulting queries: `"tracing 0.2 Rust 2026"`, `"fastrace Rust low-overhead tracing 2026"`, `"opentelemetry-rust native integration 2026"`

This technique (inspired by HyDE — Hypothetical Document Embeddings) significantly improves recall for technical queries.

### Rule 4: Query Variation on Retry

If the first query yields no useful results (no relevant pages, or all results are older than 2 years), rephrase using one of these strategies:

| Strategy | Example |
|----------|---------|
| Broader scope | `"tokio 2.0"` → `"Rust async runtime upgrade 2026"` |
| Narrower scope | `"Rust tooling 2026"` → `"cargo nextest performance 2026"` |
| Synonym substitution | `"hexagonal architecture"` → `"ports and adapters pattern Rust"` |
| Source-specific | add `site:github.com` or `site:crates.io` |
| Exact phrase | wrap key term in quotes: `'"clean architecture" Rust 2026'` |

Do not repeat the same query if it yielded nothing. Always vary on retry.

## Source Triangulation

**A finding is only as reliable as its sources.**

Before including a finding in output:
1. Find 3+ independent sources confirming the same information
2. At least 1 source should be primary (see Source Weighting below)
3. Sources must be independent — blog posts citing the same upstream source count as 1 source

**Why triangulation matters:**
- Single-source findings are often wrong, outdated, or misleading
- LLM-generated content may hallucinate version numbers and feature names
- Release notes and changelogs are ground truth; blog posts are interpretations

If you can only find 1-2 sources, note the finding as "low confidence" and flag it for manual verification.

## Source Weighting

Apply these tiers when evaluating source quality:

| Tier | Source type | Examples | Trust |
|------|-------------|----------|-------|
| Primary | Official documentation, release notes, changelogs, GitHub releases | `docs.rs`, `crates.io`, `github.com/releases`, official project blog | Highest |
| Secondary | Technical blog posts, conference talks, tutorials from known contributors | `blog.rust-lang.org`, `rustconf talks`, major engineering blogs | Corroborating |
| Tertiary | Community posts, social media, forums | Reddit r/rust, Twitter/X threads, HN comments | Supporting evidence only |

**Primary sources trump secondary sources.** If a primary source (release notes) contradicts a secondary source (blog post), trust the primary source.

## Channel Diversity

Cast queries across multiple channel types to maximize coverage and reduce channel bias:

| Channel | Best for | Query hint |
|---------|----------|-----------|
| Official docs & releases | Version upgrades, breaking changes | `site:docs.rs`, `site:github.com releases` |
| Newsletters | Weekly ecosystem summaries | `"This Week in Rust"`, `"Rust newsletter"` |
| Conference talks | Architecture patterns, new techniques | `site:youtube.com`, `"RustConf 2025"`, `"RustNation 2026"` |
| Awesome lists | Curated alternatives and tools | `"awesome-rust 2026"`, `"awesome-cli-apps"` |
| GitHub repositories | Trending crates, community adoption | `site:github.com stars crate` |
| Postmortems & incident reports | Anti-patterns, known failure modes | `"postmortem"`, `"lessons learned"`, `"production issue"` |

## Query Templates Per Category

Use these templates as starting points. Substitute `<tech>` with actual detected dependencies or patterns.

### dep-scanner — Dependency Upgrades

```
"<dep_name> <current_version> upgrade breaking changes 2026"
"<dep_name> latest release changelog 2025 2026"
"<dep_name> alternative crate comparison 2026"
```

### arch-scout — Architecture Patterns

```
"<pattern> <language> 2026 best practices"
"<pattern> <language> crates libraries 2026"
"<pattern> <language> real-world example 2026"
```

### tool-scout — Tooling & CI/CD

```
"<language> toolchain new features 2026"
"<tool> improvements performance 2026"
"<language> CI/CD best practices 2026 GitHub Actions"
```

### ecosystem-scout — Ecosystem Trends

```
"<language> ecosystem trending 2026"
"<language> most downloaded crates 2026"
"<language> community survey results 2026"
```

### competitor-scout — Competitor Tools

```
"<domain> tools comparison 2026"
"alternatives to <tool_name> 2026"
"<tool_name> vs <competitor> 2026"
```

### user-request-miner — User Request Patterns

```
"<tool_name> feature requests GitHub issues 2025 2026"
"developer workflow pain points <domain> 2026"
"<tool_name> community feedback 2026"
```

### blog-miner — Blog Posts & Newsletters

```
"<language> newsletter 2026 <topic>"
"This Week in Rust 2026 <topic>"
"<tech_area> engineering blog post 2026"
```

### research-scout — Academic & Research Papers

```
"<technique> research paper 2025 2026"
"<domain> LLM agent architecture 2026"
"<technique> production deployment lessons 2026"
```

## Anti-Patterns

Avoid these common research mistakes:

- **Natural language queries** — web search engines respond better to keywords; natural language wastes specificity
- **Single-source findings** — one blog post is not enough; always triangulate
- **Stale results without flagging** — if the best source is from 2022, note it as potentially stale
- **Query repetition on failure** — vary the query; repeating the same query returns the same empty results
- **Raw content in output** — never pass raw search result text downstream; always summarize into structured findings
- **Undated findings** — always record when the source was published or last updated
