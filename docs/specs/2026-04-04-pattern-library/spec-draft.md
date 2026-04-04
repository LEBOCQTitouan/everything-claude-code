# Spec: Pattern Library — Phase 1 Foundation

## Problem Statement

ECC agents generate, review, and refactor code across 10+ programming languages but lack a unified, structured reference for software design patterns. Existing language-specific skills (rust-patterns, python-patterns, etc.) provide idiomatic guidance but don't offer cross-language comparison, consistent schema, or machine-parseable metadata for pattern discovery. Developers and agents need a curated, searchable pattern catalog where each pattern has language-specific implementations, anti-patterns, and cross-references — enabling agents to find the right pattern at the right time.

## Research Summary

- Established catalogs use category-first hierarchy with consistent per-pattern metadata. Refactoring.guru, AgentWiki, and POSA all organize patterns into categories with fixed schema per entry.
- AI agent discovery requires machine-parseable frontmatter and semantic tags, not just prose. Vercel knowledge-agent template and MCP registry demonstrate structured YAML metadata as key to agent-accessible pattern matching.
- Composability and cross-referencing are more valuable than exhaustive individual docs. AgentWiki documents which patterns compose together; a related-patterns field enables this.
- Main pitfall: over-engineering the catalog itself. Patterns.dev warns against rigid taxonomies before having enough entries. Start with minimal schema and let categories emerge.
- Markdown with YAML frontmatter is the emerging standard for tool-integrated catalogs. Aligns with how ECC already structures agents, skills, and hooks.
- Prior art is fragmented. awesome-design-patterns, refactoring.guru, sourcemaking.com exist but none offer pattern library as code with frontmatter-based discovery for LLM agents.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Patterns as separate patterns/ content type, not skills or domain entities | Richer schema than skills. 150+ files in nested hierarchy is architecturally distinct. Existing skills serve workflow guidance, not just pattern reference. Zero domain model changes needed. | Yes (ADR-0045) |
| 2 | Pattern file schema with required sections plus full quality gate validation | Full quality gate: frontmatter + required sections + cross-reference resolution + language implementation matching + anti-patterns non-empty. | Yes (ADR-0046) |
| 3 | languages: ["all"] for universal patterns | Universal patterns declare languages: ["all"]. Language-specific patterns list exact languages. Empty list is a validation error. | No |
| 4 | Parallel validation using rayon from the start | Future-proofs for 150+ file growth. Note: rayon is a new dependency requiring cargo deny approval. Introduces crossbeam-* transitive deps. | No |
| 5 | Warn on known-unsafe code patterns in examples | Scan code blocks for eval, exec, raw SQL strings, etc. Suppressible with unsafe-examples: true frontmatter field. | No |
| 6 | Keep existing language skills, add cross-references | Existing skills serve broader purpose (workflow guidance, review checklists). Pattern library is focused reference. Cross-reference via related-skills / related-patterns fields. | No |
| 7 | Phase 1 scope: schema + validation + 2 seed categories + install + discovery | ~8 patterns, ~400-500 LOC Rust (full quality gate is substantial). Proves the format before investing in all 150+ patterns. | No |
| 8 | Four glossary terms: pattern library, pattern schema, pattern category, language matrix | Ensures consistent domain terminology across agents and docs. | No |

## User Stories

### US-001: Pattern File Schema and Directory Structure

**As a** pattern author, **I want** a well-defined schema for pattern markdown files and a standard directory layout, **so that** all patterns follow a consistent format that agents can parse reliably.

#### Acceptance Criteria

- AC-001.1: Given the patterns/ directory, when I create a new pattern file, then it follows the schema: YAML frontmatter with required fields (name, category, tags, languages, difficulty) and optional fields (related-patterns, related-skills, unsafe-examples), followed by mandatory sections: Intent, Problem, Solution, Language Implementations, When to Use, When NOT to Use, Anti-Patterns, Related Patterns, References.
- AC-001.2: Given the directory structure, when I navigate patterns/, then I find subdirectories for the 17 defined categories (creational/, structural/, behavioral/, architecture/, concurrency/, error-handling/, resilience/, testing/, ddd/, api-design/, security/, observability/, cicd/, agentic/, functional/, data-access/, idioms/). In Phase 1, only creational/ and architecture/ contain pattern files; the remaining 15 directories are created empty as placeholders for future phases.
- AC-001.3: Given a pattern file, when the languages frontmatter field lists specific languages, then only those languages appear in the Language Implementations section.
- AC-001.4: Given the patterns/ directory, when I look at patterns/index.md, then I find a hand-maintained master index listing all populated categories with pattern counts and a language coverage summary. Validation checks that every pattern file is listed in the index. Note: hand-maintained index is appropriate for Phase 1 (8 patterns); Phase 2+ should introduce automated index generation when pattern count exceeds ~30.
- AC-001.5: Given the pattern schema, the canonical language identifiers are lowercase strings: "rust", "go", "python", "typescript", "java", "kotlin", "csharp", "cpp", "swift", "shell". The special value "all" denotes universal applicability. No other values are accepted by validation.
- AC-001.6: Given the pattern schema, the canonical difficulty values are: "beginner", "intermediate", "advanced". No other values are accepted by validation.

#### Dependencies

- Depends on: none

### US-002: Pattern Validation Infrastructure

**As an** ECC developer, **I want** `ecc validate patterns` to check all pattern files against the schema with full quality gate, **so that** malformed patterns are caught before they ship.

#### Acceptance Criteria

- AC-002.1: Given the CLI, when I run `ecc validate patterns`, then each pattern file in patterns/ is checked for: valid YAML frontmatter, required fields present and non-empty (name, category, tags, languages, difficulty), category matches its parent directory name, tags is a list, languages is a list containing recognized language identifiers or "all".
- AC-002.2: Given a pattern file missing the category field, when I run `ecc validate patterns`, then I see ERROR on stderr and exit code 1.
- AC-002.3: Given a pattern file where category does not match the parent directory, when I run `ecc validate patterns`, then I see an error about the mismatch.
- AC-002.4: Given all pattern files are valid, when I run `ecc validate patterns`, then I see "Validated N pattern files across M categories" on stdout and exit code 0.
- AC-002.5: Given a pattern file, when validation runs, then it checks that all required sections exist (Intent, Problem, Solution, When to Use, When NOT to Use, Anti-Patterns, Related Patterns, References).
- AC-002.6: Given a pattern file with related-patterns references, when validation runs, then it verifies each reference resolves to an existing pattern file.
- AC-002.7: Given a pattern file with language-specific implementations, when validation runs, then it verifies each implementation language matches an entry in the languages frontmatter field.
- AC-002.8: Given a pattern file with code blocks containing known-unsafe patterns, when unsafe-examples frontmatter is not true, then validation emits a warning. The initial unsafe-pattern deny-list is defined as a constant (UNSAFE_CODE_PATTERNS) and is extensible without schema changes. Initial set includes: eval, exec, system, raw SQL concatenation, innerHTML.
- AC-002.9: Given the CI pipeline, when `ecc validate patterns` is added to the validate job in ci.yml, then malformed patterns block PRs. If the patterns/ directory does not exist, the step succeeds gracefully.
- AC-002.10: Given pattern validation, when scanning files, then it uses rayon for parallel file processing.
- AC-002.11: Given an empty patterns/ directory with no pattern files, when `ecc validate patterns` runs, then it outputs "Validated 0 pattern files across 0 categories" and exits 0.
- AC-002.12: Given a pattern file with an empty required section (heading present but no content below it), when validation runs, then it emits an error.
- AC-002.13: Given a pattern file that references itself in related-patterns, when validation runs, then it emits a warning about self-reference.
- AC-002.14: Given a pattern file with `languages: ["all"]`, when validation runs, then the language-implementation section matching check is skipped (all language headings are valid).
- AC-002.15: Given a pattern file with `languages: []` (empty list), when validation runs, then it emits an error.
- AC-002.16: Given pattern frontmatter with list-valued fields (tags, languages), when parsed, then the validator correctly handles YAML list syntax (both flow `[a, b]` and block `- a\n- b` forms).
- AC-002.17: Given a pattern file at the root of patterns/ (not inside a category subdirectory), when validation runs, then it is ignored with a warning.

#### Dependencies

- Depends on: US-001 (schema definition)

### US-003: Seed Patterns — Creational Category

**As an** agent consuming the pattern library, **I want** the Creational patterns category fully populated with all 5 GoF creational patterns, **so that** I can reference idiomatic implementations when generating or reviewing code involving object/value creation.

#### Acceptance Criteria

- AC-003.1: Given the patterns/creational/ directory, when I list files, then I find: factory-method.md, abstract-factory.md, builder.md, prototype.md, singleton.md.
- AC-003.2: Given any creational pattern file, when I read it, then it contains valid frontmatter, all required sections, and Language Implementations for at least Rust, Go, Python, and TypeScript.
- AC-003.3: Given builder.md, when I read the Rust implementation, then it demonstrates the typestate builder pattern (compile-time field enforcement), not just a simple fluent builder. (Content-review criterion — verified by human/agent review, not by `ecc validate patterns`.)
- AC-003.4: Given singleton.md, when I read the Anti-Patterns section, then it warns about testability problems and recommends dependency injection as an alternative. (Content-review criterion — verified by human/agent review, not by `ecc validate patterns`.)
- AC-003.5: Given all 5 creational patterns, when I run `ecc validate patterns`, then all pass full quality gate validation.

#### Dependencies

- Depends on: US-001 (directory structure), US-002 (validation for verification)

### US-004: Seed Patterns — Architecture Category

**As an** agent consuming the pattern library, **I want** the Architecture patterns category populated with Hexagonal, Clean Architecture, and CQRS, **so that** agents can reference these during system design and /design workflows.

#### Acceptance Criteria

- AC-004.1: Given the patterns/architecture/ directory, when I list files, then I find: hexagonal.md, clean-architecture.md, cqrs.md.
- AC-004.2: Given hexagonal.md, when I read it, then it cross-references ECC's own architecture as a real-world example (ports in ecc-ports, adapters in ecc-infra, domain in ecc-domain).
- AC-004.3: Given any architecture pattern, when I read the Language Implementations section, then it shows structural examples (directory layout, dependency direction, trait/interface definitions) rather than just code snippets.
- AC-004.4: Given all architecture patterns, when I run `ecc validate patterns`, then all pass full quality gate validation.

#### Dependencies

- Depends on: US-001 (directory structure), US-002 (validation for verification)

### US-005: Pattern Install Integration

**As an** ECC user running `ecc install`, **I want** the patterns/ directory to be installed alongside skills, agents, and rules, **so that** patterns are available in my Claude workspace after installation.

#### Acceptance Criteria

- AC-005.1: Given the ECC repository with patterns/, when I run `ecc install`, then patterns are copied to ~/.claude/patterns/.
- AC-005.2: Given an existing patterns/ installation, when I run `ecc install` again, then updated patterns are merged (new added, existing updated, removed cleaned).
- AC-005.3: Given the install manifest (Artifacts struct), when I inspect it after installation, then a patterns field lists all installed pattern directories. The `is_ecc_managed` function recognizes artifact_type "patterns".
- AC-005.4: Given `ecc audit`, when patterns are installed, then the audit reports pattern count and any schema violations.
- AC-005.5: Given an existing manifest without a patterns field (from older ECC version), when deserialized by the updated code, then it defaults to an empty list (backward compatible via `#[serde(default)]`).

#### Dependencies

- Depends on: US-001 (directory structure)

### US-006: Agent Discovery via Frontmatter

**As an** agent author, **I want** agents to declare `patterns: ["creational", "architecture"]` in their frontmatter, **so that** relevant pattern categories are discoverable and validatable.

#### Acceptance Criteria

- AC-006.1: Given an agent with `patterns: ["creational"]` in frontmatter, when `ecc validate agents` is run, then the pattern category reference is validated against existing patterns/ directories.
- AC-006.2: Given an agent with no `patterns:` field, when `ecc validate agents` is run, then no pattern-related warnings are emitted (backward compatible).
- AC-006.3: Given an agent referencing a non-existent pattern category, when `ecc validate agents` is run, then a warning is emitted (not an error, for forward compatibility).
- AC-006.4: Given the `architect` agent, when its frontmatter is updated, then it declares `patterns: ["architecture"]` alongside its existing skills.

#### Dependencies

- Depends on: US-001 (directory structure), US-002 (validation infrastructure)

## Affected Modules

| Module | Layer | Change Type |
|--------|-------|-------------|
| crates/ecc-app/src/validate/mod.rs | App | Add Patterns variant to ValidateTarget enum |
| crates/ecc-app/src/validate/patterns.rs | App | New file — pattern validation logic (~250-350 LOC) |
| crates/ecc-cli/src/commands/validate.rs | CLI/Adapter | Add Patterns to CliValidateTarget enum + mapping |
| crates/ecc-domain/src/config/manifest.rs | Domain | Add patterns: Vec<String> to Artifacts struct + update is_ecc_managed() |
| crates/ecc-app/src/install/helpers/artifacts.rs | App | Add pattern collection to collect_installed_artifacts() |
| crates/ecc-app/src/validate/agents.rs | App | Add optional patterns: frontmatter validation |
| crates/ecc-app/src/audit/ | App | Report pattern count and schema violations in ecc audit |
| .github/workflows/ci.yml | CI | Add ecc validate patterns step |
| agents/architect.md | Content | Add patterns: ["architecture"] to frontmatter |
| patterns/ | Content | New directory with schema, index, and 8 seed pattern files |
| docs/domain/glossary.md | Docs | Add 4 new terms |
| docs/adr/0045-patterns-as-content-type.md | Docs | ADR: Patterns as separate content type |
| docs/adr/0046-pattern-file-schema.md | Docs | ADR: Pattern file schema design |

## Constraints

- Zero changes to ecc-domain core logic (business rules) — patterns are content, not domain entities
- Zero changes to ecc-ports — no new port traits needed
- Zero changes to ecc-infra — existing FileSystem trait sufficient
- Pattern validation must follow existing validation module architecture (same function signature, same error reporting)
- Existing language skills (rust-patterns, python-patterns, etc.) remain unchanged
- CI validate step must include patterns after this feature ships

## Non-Requirements

- Not in Phase 1: Context injection in /spec-* and /implement commands (deferred to Phase 4)
- Not in Phase 1: Review integration (code reviewers auto-referencing patterns) (deferred to Phase 4)
- Not in Phase 1: Full 150+ pattern catalog (13 remaining categories deferred to Phases 2-5)
- Not in Phase 1: Pattern search CLI (ecc patterns search) (YAGNI, revisit if needed)
- Not in Phase 1: Database-backed pattern indexing (YAGNI)
- Not in Phase 1: Migration of existing language skills into pattern library
- Not in scope: Runtime pattern loading by Claude Code skill loader (outside ECC control)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| FileSystem (read) | Extended usage | Pattern files read during validation — covered by existing InMemoryFileSystem |
| TerminalIO (write) | Extended usage | Validation output formatting — covered by existing test doubles |
| CLI validate command | New subcommand | Integration test needed: ecc validate patterns against fixture directory |
| Install pipeline | New artifact type | Integration test needed: patterns installed to target directory |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New feature | CLAUDE.md | CLI command list | Add ecc validate patterns |
| New concept | Glossary | docs/domain/glossary.md | Add 4 terms |
| Architecture decision | ADR | docs/adr/0045-patterns-as-content-type.md | Create |
| Schema decision | ADR | docs/adr/0046-pattern-file-schema.md | Create |
| New content type | Architecture | docs/ARCHITECTURE.md | Note patterns/ in project structure |

## Open Questions

None — all resolved during grill-me interview.
