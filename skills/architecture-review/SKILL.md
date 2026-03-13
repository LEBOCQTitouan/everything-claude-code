---
name: architecture-review
description: Audit codebase architecture for layering violations, dependency direction, coupling, cohesion, circular dependencies, and DDD/hexagonal compliance.
origin: ECC
---

# Architecture Review

Audit a codebase for structural and architectural quality. Language-agnostic — works on any project with a clear directory structure.

## When to Activate

- Onboarding to an unfamiliar codebase
- Before major releases
- Periodic health checks (monthly/quarterly)
- After significant refactoring
- When maintainability or performance degrades
- Planning a migration or rewrite

## Audit Dimensions

### 1. Dependency Direction

Imports must flow inward: adapters → application → domain. Never the reverse.

- **CRITICAL**: Domain layer imports infrastructure/framework types
- **CRITICAL**: Adapter types (ORM, HTTP, SDK) leak into domain/application
- **HIGH**: Application layer imports adapter types

Detection: Grep domain/ files for imports from adapters/, infra/, framework packages.

### 2. Layer Separation

The directory structure must reflect clear layer boundaries.

- **HIGH**: No clear layer separation (no domain/, application/, adapters/ or equivalent)
- **MEDIUM**: Mixed concerns in a single directory (business logic alongside HTTP handlers)

Detection: Analyze directory tree for standard layer directories or feature-based equivalents.

### 3. Circular Dependencies

Modules must not form import cycles.

- **CRITICAL**: Import cycles between layers (domain ↔ infrastructure)
- **MEDIUM**: Import cycles within a layer (moduleA ↔ moduleB in same layer)

Detection: Trace import chains. If A imports B and B imports A (transitively), flag it.

### 4. Coupling Analysis

Measure how tightly modules are connected.

- **HIGH**: God module — more than 20 files depend on it (fan-in > 20)
- **MEDIUM**: High fan-out — module imports more than 15 other modules

Detection: Count import fan-in and fan-out per module/file.

### 5. Cohesion Analysis

Files within a module should serve a single purpose.

- **MEDIUM**: Module with unrelated responsibilities (e.g., auth + billing in same directory)
- **MEDIUM**: Classes/files with methods that don't use shared state

Detection: Analyze naming patterns, method signatures, responsibility overlap.

### 6. Domain Model Quality

The domain layer should contain real business logic, not just data structures.

- **HIGH**: Anemic domain — entities with only getters/setters and no behavior
- **MEDIUM**: Primitive obsession — raw strings/numbers where value objects belong
- **LOW**: Missing domain events for significant state changes

Detection: Search domain model files for methods beyond accessors.

### 7. Bounded Context Boundaries

Contexts should be independent and communicate through explicit contracts.

- **HIGH**: Cross-context direct imports (OrderService importing UserRepository from another context)
- **HIGH**: Shared database/schema across contexts without explicit shared kernel

Detection: Map contexts, check for direct imports between them.

### 8. Ports & Adapters Compliance

Domain defines ports (interfaces); adapters implement them.

- **CRITICAL**: No port interfaces — domain depends on concrete implementations
- **HIGH**: Missing port interfaces for external dependencies (DB, APIs, messaging)

Detection: Check for interface definitions in domain layer and implementations in adapter layer.

### 9. File Organization

Project structure should support maintainability.

- **MEDIUM**: Files exceeding 800 lines
- **LOW**: Organized by type (controllers/, models/, views/) instead of by feature/domain

Detection: File size census, directory name analysis.

### 10. SOLID & Clean Architecture

Single Responsibility, Open/Closed, Liskov Substitution, Interface Segregation, Dependency Inversion.

- Severity varies per violation — delegated to `uncle-bob` agent

Detection: Structural analysis of class/function responsibilities, interface design, dependency direction.

### 11. Dependency Metrics

Quantitative stability and abstractness metrics per module.

- **Instability**: `I = Ce / (Ca + Ce)` where Ce = efferent coupling (outgoing deps), Ca = afferent coupling (incoming deps). Range 0 (maximally stable) to 1 (maximally unstable).
- **Abstractness**: `A = abstract_symbols / total_symbols` where abstract_symbols = interfaces, abstract classes, type definitions. Range 0 (fully concrete) to 1 (fully abstract).
- **Distance from main sequence**: `D = |A + I - 1|`. Modules should cluster near the line A + I = 1.

**Zones**:
- **Zone of Pain** (low A, low I): Concrete and stable — hard to change, everything depends on it. Flag with HIGH if D > 0.3.
- **Zone of Uselessness** (high A, high I): Abstract and unstable — interfaces nobody implements. Flag with MEDIUM if D > 0.3.

Detection: Count imports into (Ca) and out of (Ce) each top-level module. Count type/interface definitions vs concrete implementations.

### 12. Boundary Coherence

Types and state should not leak across bounded context boundaries.

- **Type leakage**: Types appearing in the public API of more than one bounded context — HIGH
- **God state**: Shared mutable state (global variables, singleton stores) accessed from multiple modules — CRITICAL
- **God DTOs**: Data transfer objects with fields from multiple contexts (e.g., `UserOrderPaymentDTO`) — HIGH
- **Missing anti-corruption layers**: Direct use of external/third-party types without a translation layer at the boundary — MEDIUM

Detection: Map types to their defining module. Search for imports of those types from other contexts. Check for shared state patterns (global stores, static mutable variables).

## Architecture Score

| Score | Criteria |
|-------|----------|
| A (HEALTHY) | 0 CRITICAL, 0 HIGH, <=3 MEDIUM |
| B (GOOD) | 0 CRITICAL, <=2 HIGH, any MEDIUM |
| C (NEEDS ATTENTION) | 0 CRITICAL, >2 HIGH |
| D (NEEDS REFACTORING) | 1+ CRITICAL or >5 HIGH |
| F (CRITICAL) | 3+ CRITICAL issues |

## Language-Specific Heuristics

Import pattern detection by language:

| Language | Import Pattern | Layer Indicators |
|----------|---------------|------------------|
| TypeScript/JS | `import`/`require` | src/domain/, src/application/, src/adapters/ |
| Python | `import`/`from` | domain/, application/, infrastructure/ |
| Go | `import` block | internal/, pkg/, cmd/ |
| Java | `import` | domain/, application/, infrastructure/, adapters/ |
| Rust | `use` | src/domain/, src/application/, src/adapters/ |

## Anti-Pattern Catalog

| Anti-Pattern | Signal | Severity |
|--------------|--------|----------|
| God class/module | >500 LOC or >20 dependents | HIGH |
| Leaky abstraction | Framework types in domain signatures | CRITICAL |
| Shotgun surgery | Single change requires touching >5 files | MEDIUM |
| Feature envy | Method uses more data from another module than its own | MEDIUM |
| Shared mutable state | Global variables, singletons with state | HIGH |
| Big ball of mud | No discernible layer structure | CRITICAL |

## Related

- Agent: `agents/arch-reviewer.md`
- Command: `commands/arch-review.md`
- Consulted agents: `agents/architect.md`, `agents/architect-module.md`, `agents/uncle-bob.md`
