# ADR 0031: Knowledge Sources Registry

## Status

Accepted (2026-03-29)

## Context

ECC commands do ad-hoc web research with no persistent record of authoritative sources. Research effort is duplicated across sessions. There is no shared vocabulary of trusted references for the project.

## Decision

Introduce a persistent knowledge sources registry at `docs/sources.md`, organized by Technology Radar quadrants (Adopt/Trial/Assess/Hold):

- **Format**: Markdown tables per quadrant with structured metadata (URL, title, type, subject, dates)
- **Inbox flow**: Humans add to Inbox section; `ecc sources reindex` classifies into quadrants
- **Consultation**: On-demand only — commands load metadata (URL + title + quadrant), never full content. No automatic context preloading.
- **Lifecycle**: Stale sources flagged for review (never silently removed). Deprecated sources remain with reason.
- **Auto-enrichment**: `/spec` web research proposes high-quality findings to Inbox
- **CLI**: `ecc sources list/add/check/reindex` subcommands

## Consequences

- Sources become a first-class project artifact committed to git
- 7 commands gain source consultation capabilities (spec, implement, design, audit-web, audit-evolution, catchup, review)
- New bounded context in ecc-domain: `sources` (entry parsing, registry, staleness detection)
- Registry grows organically through auto-propose from web research
- Subject matching uses free-text keywords (flexible but imprecise)
- URL reachability checked via curl through ShellExecutor port (no new HTTP dependency)
