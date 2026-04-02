# Spec: BL-064 Sub-Spec B — Element Registry and Cross-Reference Matrix

## Problem Statement

Sub-Spec A shipped journeys and flows but the third cartography layer — the element registry — is missing. Developers cannot navigate the codebase by element role (which commands exist, which agents handle which flows, which ports connect which adapters). The cross-reference matrix linking elements to journeys and flows does not exist, so there is no single view showing how the system's components relate to its behaviors.

## Research Summary

- **Central Hub Pattern**: Registry architectures use a master record hub with globally unique identifiers and cross-references to source systems
- **CNCF xRegistry 1.0**: REST API and document format for software architecture metadata with multi-group resource relationships
- **Documentation Generation Automation**: Tools like Stencil.js auto-generate component docs from source code, reducing manual maintenance
- **Structured Element Description**: Enterprise architecture docs capture identity, description, relations — enabling Service Catalogues
- **Cross-Reference Mechanisms**: Enterprise Architect supports bidirectional cross-references between elements and project browser
- **Two-tier approach**: Minimal universal set + domain-specific extension is a proven pattern for extensible taxonomies

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Element generation is post-loop (after all journey/flow deltas processed) | Cross-reference matrix needs complete journey/flow state | Yes |
| 2 | Two-tier element types: universal (Module, Interface, Config, Unknown) + ECC overlay (Command, Agent, Skill, Hook, Rule, Crate, Port, Adapter, DomainEntity) | Language-agnostic core with project-specific extensions | Yes |
| 3 | INDEX.md is fully regenerated, not delta-merged | Computed view — determinism over flexibility | Yes |
| 4 | GAP markers for deleted/unknown elements, no auto-pruning | Safe default — auto-pruning risks false positives on renames | No |
| 5 | Extend cartographer agent, not separate INDEX agent | Cartographer already handles commit logic; adding INDEX step is simpler | No |

## User Stories

### US-010: Element Domain Types

**As a** cartography system, **I want** typed representations of application elements, **so that** generators, validators, and the matrix share a single source of truth.

#### Acceptance Criteria
- AC-010.1: Given the domain crate, when `ElementType` is used, then it includes universal variants (Module, Interface, Config, Unknown) and ECC variants (Command, Agent, Skill, Hook, Rule, Crate, Port, Adapter, DomainEntity)
- AC-010.2: Given an `ElementEntry`, when serialized to JSON, then it round-trips with: slug, element_type, purpose, uses, used_by, participating_flows, participating_journeys, sources, last_updated
- AC-010.3: Given `ecc-domain`, when any `std::fs`/`std::process`/`std::net` import appears in `cartography/`, then the build fails
- AC-010.4: Given `ElementType::Unknown`, when serialized, then it produces `"unknown"` and deserializes back
- AC-010.5: Given an element slug is needed, when `derive_slug()` from Sub-Spec A's `cartography::slug` module is called with the element name, then it produces a lowercase-hyphenated string (max 60 chars) — reuses the existing function, no new slug logic
- AC-010.6: Given a Rust crate name, when element type is inferred, then the mapping is: ecc-domain→DomainEntity, ecc-ports→Port, ecc-infra→Adapter, ecc-app→Module, ecc-cli→Module, ecc-workflow→Module, ecc-test-support→Module, ecc-integration-tests→Module, ecc-flock→Module
- AC-010.7: Given Sub-Spec A cartography modules exist (`merge_section`, `validate_journey`, `validate_flow`, `check_staleness`, `calculate_coverage`, `derive_slug`, `SessionDelta`, `ProjectType`), then Sub-Spec B imports and reuses them — no re-implementation

#### Dependencies
- Depends on: Sub-Spec A (cartography bounded context must be implemented)

### US-011: Element Schema Validation

**As a** cartography validator, **I want** `validate_element()` checking required sections, **so that** malformed entries are caught.

#### Acceptance Criteria
- AC-011.1: Given an element file with all sections (Overview, Relationships, Participating Flows, Participating Journeys), when validated, then Ok(())
- AC-011.2: Given missing sections, when validated, then Err listing missing names
- AC-011.3: Given `ecc validate cartography` scans `elements/`, when any file fails, then CLI prints error and exits 1
- AC-011.4: Given sections in any order, when validated, then Ok (order-independent)

#### Dependencies
- Depends on: US-010

### US-012: Element File Generator Agent

**As a** cartography system, **I want** an agent that produces per-element markdown files, **so that** each element is documented.

#### Acceptance Criteria
- AC-012.1: Given a delta references element source files, when the generator runs, then it creates/updates `docs/cartography/elements/<slug>.md` passing validation
- AC-012.2: Given existing element file, when updated, then delta-merge using merge_section() preserves manual content
- AC-012.3: Given element created, then Relationships lists uses/used_by as relative links
- AC-012.4: Given element created, then Participating Flows lists relative links to flow files
- AC-012.5: Given element created, then Participating Journeys lists relative links to journey files
- AC-012.6: Given unknown element type, then file has `element_type: unknown` and GAP marker
- AC-012.7: Given element type inference, then path-based classification: `agents/`→Agent, `commands/`→Command, `skills/`→Skill, `hooks/`→Hook, `rules/`→Rule; crate role: `ecc-ports`→Port, `ecc-domain`→DomainEntity; fallback→Unknown
- AC-012.8: Given `elements/` doesn't exist, then directory created on first write

#### Dependencies
- Depends on: US-010

### US-013: Cartographer — Element Dispatch

**As a** cartography system, **I want** the cartographer agent to dispatch the element generator after journey/flow processing, **so that** elements are updated in the same pass.

#### Acceptance Criteria
- AC-013.1: Given pending deltas processed, when cartographer runs, then element generator dispatched AFTER journey+flow generators complete
- AC-013.2: Given element generator succeeds, then `git add docs/cartography/` stages element files (existing scoped-add)
- AC-013.3: Given element generator fails, then same failure path: git reset, no archive
- AC-013.4: Given no element targets in delta, then element generator not dispatched

#### Dependencies
- Depends on: US-012

### US-014: Cross-Reference Matrix (INDEX.md)

**As a** developer, **I want** INDEX.md with element×journey×flow matrix, **so that** I can see participation at a glance.

#### Acceptance Criteria
- AC-014.1: Given element files exist, when INDEX regenerated, then INDEX.md has table: rows=elements, columns=journeys+flows, cells=Y/blank
- AC-014.2: Given existing INDEX.md, when regenerated, then fully replaced (not delta-merged)
- AC-014.3: Given new element added, when cartographer runs, then INDEX.md regenerated after element generators
- AC-014.4: Given INDEX rendered as Markdown, then journey columns before flow columns
- AC-014.5: Given `ecc validate cartography` finds `docs/cartography/elements/INDEX.md` absent, then warning (not error)
- AC-014.6: Given `docs/cartography/elements/INDEX.md` is stale (missing slugs), then validator warns with missing list
- AC-014.7: Given INDEX.md is generated, then it is written to `docs/cartography/elements/INDEX.md` (canonical path)

#### Dependencies
- Depends on: US-012, US-013

### US-015: Element Staleness and Coverage

**As a** developer, **I want** staleness and coverage to include elements, **so that** the dashboard reflects full cartography scope.

#### Acceptance Criteria
- AC-015.1: Given element file with CARTOGRAPHY-META marker, when source modified after last_updated, then staleness reported
- AC-015.2: Given `--coverage` runs, when element files exist, then coverage includes element-referenced files
- AC-015.3: Given coverage below 50%, then priority gaps include all sources (journeys+flows+elements)

#### Dependencies
- Depends on: US-011

### US-016: Element Directory Scaffold

**As a** developer, **I want** `docs/cartography/elements/` created automatically, **so that** the scaffold is complete.

#### Acceptance Criteria
- AC-016.1: Given start_cartography runs and `elements/` missing, then directory + README created
- AC-016.2: Given `elements/` exists, then left untouched
- AC-016.3: Given validate runs and `elements/` missing, then exits cleanly (no error)

#### Dependencies
- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `ecc-domain/src/cartography/element_types.rs` | Domain | New: ElementEntry, ElementType, CrossReferenceMatrix |
| `ecc-domain/src/cartography/element_validation.rs` | Domain | New: validate_element() |
| `ecc-domain/src/cartography/cross_reference.rs` | Domain | New: build_cross_reference_matrix() |
| `ecc-domain/src/cartography/mod.rs` | Domain | Modify: add 3 new modules |
| `ecc-app/src/validate_cartography.rs` | Application | Modify: add element scan, staleness, coverage |
| `ecc-app/src/hook/handlers/tier3_session/cartography.rs` | Application | Modify: add elements/ scaffold, post-loop element generation |
| `agents/cartography-element-generator.md` | Agent | New |
| `agents/cartographer.md` | Agent | Modify: add element dispatch + INDEX generation |

## Constraints

- Element generation runs AFTER journey/flow processing (post-loop)
- INDEX.md is fully regenerated, never delta-merged
- Two-tier element types: universal + ECC overlay
- Reuse existing merge engine, validation pattern, staleness, coverage from Sub-Spec A
- No auto-pruning of deleted element files
- Handler file must stay under 800 lines

## Non-Requirements

- Auto-pruning of element files for deleted source code (GAP markers instead)
- `--strict` mode for INDEX.md validation
- Interactive HTML navigation
- Full-project element backfill scan
- Separate INDEX generator agent (extend cartographer instead)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| FileSystem | Used by element generator | E2E: element files written correctly |
| ShellExecutor | Used for git operations | E2E: commit includes element files |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New ADRs | docs/adr/ | 3 ADRs (post-loop, two-tier types, INDEX regen) | Create |
| Bounded context | docs/domain/bounded-contexts.md | Add element types to cartography context | Edit |
| Feature entry | CHANGELOG.md | Add Sub-Spec B entry | Edit |
| Test count | CLAUDE.md | Update test count | Edit |

## Open Questions

None — all resolved during grill-me.
