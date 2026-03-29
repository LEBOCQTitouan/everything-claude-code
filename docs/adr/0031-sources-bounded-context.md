# ADR 0031: Sources as Independent Bounded Context

## Status

Accepted (2026-03-29)

## Context

BL-086 introduces a knowledge sources registry — a curated list of reference sources organized by Technology Radar quadrants. The architect review identified this as a ninth bounded context in the domain model, independent from backlog, config, and workflow.

The design replicates the backlog module pattern exactly: domain types in `ecc-domain`, app use cases in `ecc-app`, CLI wiring in `ecc-cli`. This structural consistency reduces cognitive load and enables the same testing patterns (InMemoryFileSystem, MockExecutor).

The key question: should sources share any domain types with backlog (both manage markdown-file-based registries), or remain fully independent? Knowledge duplication risk exists if both modules implement similar parsing logic.

## Decision

Sources is a **fully independent bounded context** with its own:

- **Ubiquitous language**: Technology Radar vocabulary — quadrant (Adopt/Trial/Assess/Hold), subject, source type, module mapping, stale flag
- **Data ownership**: `docs/sources.md` — a single committed Markdown file
- **Domain types**: `SourceEntry`, `Quadrant`, `SourceType`, `SourcesRegistry`, `ModuleMapping`
- **Lifecycle model**: entries move between quadrants over time, can be flagged stale or deprecated

No shared domain types with backlog. The parsing approaches are different (backlog uses YAML frontmatter per file; sources uses a structured Markdown format in a single file). Module mapping — an explicit table mapping codebase modules to source subjects — is unique to this bounded context.

Command integrations (`/spec`, `/design`, `/implement`, `/audit`, `/review`, `/catchup`) are **conformist** integrations — they read the file as-is through the same domain parser. No anti-corruption layer needed.

## Consequences

- Sources module has zero dependencies on other domain modules
- No other domain modules should depend on sources
- `docs/domain/bounded-contexts.md` gains a ninth entry
- Technology Radar vocabulary (quadrant, subject, adopt/trial/assess/hold) is the canonical naming in all layers
- Future modules following the same flat-file-registry pattern should also be independent bounded contexts
