# Performance Optimization

## Model Routing

> "Start with Sonnet, route only the most demanding to Opus."

| Model | Cost (MTok) | Use For |
|-------|-------------|---------|
| **Haiku 4.5** | $1/$5 | Diff detection, formatting, extraction, web research workers |
| **Sonnet 4.6** | $3/$15 | Code review, audits, TDD, build resolution, doc validation |
| **Opus 4.6** | $5/$25 | Architecture, security, adversarial review, planning |

## Thinking Effort Tiers

| Effort | Tokens | Model Tier |
|--------|--------|------------|
| low | 2,048 | Haiku |
| medium | 8,192 | Sonnet |
| high | 16,384 | Sonnet (complex) / Opus |
| max | 32,768 | Opus |

Set via agent frontmatter `effort` field. `SubagentStart` hook applies. Bypass: `ECC_EFFORT_BYPASS=1`.
