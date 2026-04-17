---
id: BL-120
title: "Pattern Library for Agent-Assisted Development"
status: implemented
scope: EPIC
target: /spec-dev
tags: [patterns, knowledge, agents, multi-language, library]
created: 2026-04-04
---

## Raw Idea

"I want to have a store of patterns, code, agents, ... and I want the agents to use that 'library'. Look on internet for all those patterns, be extensive, and put language-specific recommendations for each pattern if needed (some patterns might not be relevant for some languages)."

## Challenge Notes

### What is a "Pattern Library"?

A curated, structured collection of markdown files (one per pattern or pattern group) containing:
- Pattern name, intent, and when to use
- Language-agnostic description with diagrams
- Language-specific implementations with idiomatic code snippets
- Anti-patterns and common mistakes
- References to existing ECC skills that already cover the pattern

This is distinct from BL-086 (Knowledge Sources Registry), which curates external URLs. The pattern library is an internal, self-contained knowledge base that agents can read and apply directly during code generation, review, and refactoring.

### How would agents discover and use patterns?

- **Skill-based discovery**: Each pattern category becomes an ECC skill (or skill group) that agents can preload via frontmatter `skills:` declarations
- **Search-based discovery**: Agents use `ecc memory search` or file glob to find patterns by tag, language, or category
- **Context injection**: The `/spec-*` and `/implement` commands could auto-inject relevant patterns based on detected language and architectural context
- **Review integration**: Code reviewers reference the library to flag deviations from established patterns

### Scope assessment

This is an EPIC because:
- 15+ pattern categories, each with multiple patterns
- Language-specific variants for 8+ languages
- Integration with agent discovery, review, and code generation workflows
- Cross-references to existing ECC skills (rust-patterns, python-patterns, etc.)
- Needs a schema for pattern files and an index/registry mechanism

### Relationship to existing skills

ECC already has language-specific skills (rust-patterns, python-patterns, golang-patterns, kotlin-patterns, etc.) and domain skills (api-design, security-review, tdd-workflow). The pattern library should:
1. Consolidate and extend these into a unified, searchable catalog
2. Add cross-language comparison tables
3. Add patterns not yet covered (resilience, DDD tactical, concurrency, agent patterns)
4. Provide a discovery layer so agents find the right pattern at the right time

## Research

### 1. GoF Design Patterns (23 patterns)

**Creational (5)**: Factory Method, Abstract Factory, Builder, Prototype, Singleton

| Pattern | Rust | Go | Python | TypeScript | Java/Kotlin | C#/C++ |
|---------|------|-----|--------|------------|-------------|--------|
| Factory Method | Trait + associated fn | Interface + constructor fn | `__init__` subclass / classmethod | Class + static create | Standard OOP | Standard OOP |
| Abstract Factory | Trait family | Interface family | ABC | Generic factory | Standard OOP | Standard OOP |
| Builder | Typestate builder (compile-time safety) | Functional options | `__init__` kwargs / fluent | Fluent builder | Lombok @Builder / DSL | Fluent builder |
| Prototype | Clone trait | Deep copy via encoding | `copy.deepcopy` | Spread operator / structuredClone | Cloneable | ICloneable |
| Singleton | `once_cell::Lazy` / `std::sync::OnceLock` | `sync.Once` | Module-level / `__new__` | Module scope / class static | Enum singleton | `Lazy<T>` |

**Structural (7)**: Adapter, Bridge, Composite, Decorator, Facade, Flyweight, Proxy

| Pattern | Rust | Go | Python | TypeScript | Java/Kotlin |
|---------|------|-----|--------|------------|-------------|
| Adapter | Trait impl wrapper | Interface embedding | Duck typing / adapter class | Class adapter | Standard OOP |
| Decorator | Trait wrapping / newtype | Middleware chain | `@decorator` / `functools.wraps` | Class decorator / HOF | Decorator pattern / Kotlin delegated properties |
| Composite | Enum variants (algebraic) | Interface + slice | List of same-typed objects | Union types | Composite interface |
| Proxy | Smart pointer (`Deref`) | Interface wrapper | `__getattr__` proxy | Proxy class / ES6 Proxy | Dynamic proxy / Kotlin delegation |

**Behavioral (11)**: Strategy, Template Method, Observer, Mediator, Command, Iterator, State, Chain of Responsibility, Memento, Visitor, Interpreter

| Pattern | Rust | Go | Python | TypeScript |
|---------|------|-----|--------|------------|
| Strategy | Trait object / enum dispatch | Function parameter / interface | First-class functions / protocol | Interface + injection |
| Observer | Channel-based / callback Vec | Channel / goroutine | Signals / event emitter | EventEmitter / RxJS |
| Iterator | `Iterator` trait (native) | Range / channel | `__iter__` / generator | Symbol.iterator / generator |
| State | Enum + match (typestate) | Interface per state | Class per state | Discriminated union |
| Visitor | Enum match (no need for classic visitor) | Interface | Double dispatch / `functools.singledispatch` | Pattern matching |
| Command | Closure / trait object | Function value | Callable / class | Function / class |

### 2. Architecture Patterns

| Pattern | Description | Relevant Languages |
|---------|-------------|-------------------|
| Hexagonal (Ports & Adapters) | Core logic isolated from I/O via port traits and adapter impls | All (Rust, Go, Java, TS, Python) |
| Clean Architecture | Concentric layers: entities, use cases, interface adapters, frameworks | All |
| CQRS | Separate read/write models | All (especially event-driven stacks) |
| Event Sourcing | Persist state as immutable event stream | All (Kotlin/Java + Axon, TS + EventStoreDB) |
| Microservices | Independent deployable services via APIs | All |
| Modular Monolith | Single deployment, strict module boundaries | All |
| Saga Pattern | Distributed transaction coordination via compensating actions | Java/Kotlin, Go, TS (orchestration frameworks) |
| Outbox Pattern | Reliable event publishing via transactional outbox table | Java/Kotlin, Go, Python (DB-heavy stacks) |
| Strangler Fig | Incremental migration from legacy to new system | All |
| Backend for Frontend (BFF) | API gateway per client type | TS/Go/Java |

### 3. Concurrency Patterns

| Pattern | Rust | Go | Python | TypeScript | Java/Kotlin |
|---------|------|-----|--------|------------|-------------|
| Async/Await | `tokio`/`async-std` | N/A (goroutines) | `asyncio` | Native promises | Kotlin coroutines / Java CompletableFuture |
| CSP (Channels) | `tokio::sync::mpsc` | Goroutines + channels (native) | `asyncio.Queue` | N/A | Kotlin Channels |
| Actor Model | `actix` | N/A (use channels) | `pykka` | N/A | Akka (Java/Scala) |
| Thread Pool | `rayon` | Goroutine pool | `concurrent.futures` | Worker threads | `ExecutorService` |
| Read-Write Lock | `RwLock` | `sync.RWMutex` | `threading.RLock` | N/A (single-threaded) | `ReentrantReadWriteLock` |
| Fan-out/Fan-in | `tokio::spawn` + join | Goroutine fan-out | `asyncio.gather` | `Promise.all` | Coroutine scope |

### 4. Error Handling Patterns

| Pattern | Rust | Go | Python | TypeScript | Java/Kotlin |
|---------|------|-----|--------|------------|-------------|
| Result/Either | `Result<T, E>` (native) | `(value, error)` tuple | N/A (exceptions) | `Result` type (custom/neverthrow) | Kotlin `Result` / Arrow Either |
| Railway Oriented | `?` operator chaining | Error wrapping chain | N/A | `fp-ts` Either chain | Arrow monad comprehensions |
| Error Wrapping | `thiserror` / `anyhow` | `fmt.Errorf("%w")` | Exception chaining | Custom error classes | Exception cause chain |
| Structured Errors | Enum variants with context | Sentinel errors + `errors.Is` | Custom exception hierarchy | Discriminated union errors | Sealed class hierarchy |
| Panic/Recover | `panic!` + `catch_unwind` | `panic` + `recover` | N/A | N/A | N/A |

### 5. Resilience Patterns

| Pattern | Description | Libraries by Language |
|---------|-------------|----------------------|
| Circuit Breaker | Stop calling failing services | Rust: custom / `circuitbreaker`, Go: `gobreaker`, Java: Resilience4j, Python: `pybreaker`, TS: `opossum` |
| Retry + Backoff | Exponential retry on transient failures | Rust: `backoff`, Go: built-in / `retry-go`, Java: Resilience4j, Python: `tenacity`, TS: `p-retry` |
| Bulkhead | Isolate failure domains | Rust: `tokio::Semaphore`, Go: bounded channels, Java: Resilience4j, TS: custom |
| Timeout | Fail fast on slow calls | Rust: `tokio::time::timeout`, Go: `context.WithTimeout`, Python: `asyncio.wait_for`, TS: `AbortController` |
| Fallback | Degrade gracefully | All: pattern, not library |
| Rate Limiter | Throttle request volume | Rust: `governor`, Go: `golang.org/x/time/rate`, Java: Resilience4j, Python: `ratelimit`, TS: `bottleneck` |

### 6. Testing Patterns

| Pattern | Description | When to Use |
|---------|-------------|-------------|
| Arrange-Act-Assert | Structure every test in 3 phases | All unit tests |
| Given-When-Then | BDD-style test structure | Acceptance/integration tests |
| Test Doubles (Mock/Stub/Spy/Fake) | Replace dependencies | Unit/integration isolation |
| Property-Based Testing | Generate random inputs, verify invariants | Domain logic, parsers, serialization |
| Mutation Testing | Verify test quality by injecting faults | After achieving coverage targets |
| Contract Testing | Verify API compatibility between services | Microservice boundaries |
| Snapshot Testing | Detect unintended output changes | UI components, serialized output |
| Table-Driven Tests | Parametric test cases | Go (native), Rust, all via parametrize |
| Testcontainers | Real dependencies in containers | Integration tests with DB/queue/cache |
| Approval Testing | Human-reviewed golden files | Complex output, reports |

### 7. DDD Tactical Patterns

| Pattern | Description | Language Notes |
|---------|-------------|---------------|
| Aggregate Root | Consistency boundary for entity clusters | Rust: struct with private fields + methods. Go: struct + constructor. Java/Kotlin: class with invariant checks |
| Value Object | Immutable, equality by value | Rust: `#[derive(Eq, Hash)]` struct. Python: `@dataclass(frozen=True)`. Kotlin: `data class`. TS: `readonly` |
| Entity | Identity-based domain object | All languages, use newtype for ID |
| Domain Event | Immutable record of something that happened | All, pair with event sourcing |
| Repository | Abstract data access interface | Rust: trait. Go: interface. Java: interface. Python: ABC |
| Specification | Encapsulate query/validation rules | Rust: trait + combinator. Java: Specification<T> |
| Anti-Corruption Layer | Translate between bounded contexts | All, adapter pattern at boundary |
| Domain Service | Stateless logic that doesn't belong to a single entity | All |

### 8. API Design Patterns

| Pattern | Description | Relevant Protocols |
|---------|-------------|-------------------|
| REST Resource Design | Noun-based URLs, HTTP verb semantics | REST |
| Pagination (cursor vs offset) | Handle large collections | REST, GraphQL |
| Rate Limiting + Quota | Protect APIs from abuse | REST, gRPC |
| Versioning (URL vs header) | Evolve APIs without breaking clients | REST |
| Idempotency Keys | Safe retries for non-idempotent operations | REST, gRPC |
| GraphQL Schema Design | Type-first, resolver pattern | GraphQL |
| gRPC Service Definition | Protobuf IDL, streaming modes | gRPC |
| API Gateway | Single entry point, routing, auth | All protocols |
| HATEOAS | Hypermedia-driven API navigation | REST |
| Webhook Design | Event delivery via HTTP callbacks | REST |

### 9. Security Patterns

| Pattern | Description | Language-Specific Notes |
|---------|-------------|------------------------|
| Input Validation | Schema-based validation at boundaries | Rust: `serde` + `validator`. Go: `go-playground/validator`. Python: Pydantic. TS: Zod. Java: Bean Validation |
| Authentication (OIDC/OAuth2) | Identity verification | Library per language/framework |
| Authorization (RBAC/ABAC) | Access control policies | All, framework-specific middleware |
| Secrets Management | No hardcoded secrets, env/vault | All, use `dotenv` or vault client |
| CSRF Protection | Token-based cross-site request forgery prevention | Web frameworks (Django, Express, Spring) |
| SQL Injection Prevention | Parameterized queries only | All DB-accessing code |
| XSS Prevention | Output encoding/sanitization | TS/JS (DOMPurify), Python (bleach), Java (OWASP encoder) |
| Content Security Policy | Browser-side restriction headers | Web-serving languages |

### 10. Observability Patterns

| Pattern | Description | Libraries |
|---------|-------------|-----------|
| Structured Logging | JSON/key-value log format | Rust: `tracing`. Go: `slog`/`zerolog`. Python: `structlog`. TS: `pino`. Java: Logback+JSON |
| Distributed Tracing | End-to-end request tracking | OpenTelemetry (all languages) |
| Metrics Collection | Counters, gauges, histograms | Prometheus client (all languages) |
| Correlation ID | Unique request identifier propagated across services | Middleware pattern, all languages |
| Health Checks | Liveness/readiness probes | All, endpoint-based |
| Log Aggregation | Centralized log collection and search | ELK, Loki, CloudWatch |

### 11. CI/CD & DevOps Patterns

| Pattern | Description |
|---------|-------------|
| Blue-Green Deployment | Two identical environments, instant cutover |
| Canary Release | Gradual traffic shift to new version |
| Feature Flags | Runtime feature toggling without deploy |
| Rolling Update | Incremental instance replacement |
| Infrastructure as Code | Declarative infra provisioning (Terraform, Pulumi) |
| GitOps | Git as single source of truth for infra state |
| Trunk-Based Development | Short-lived branches, frequent merges |
| Pipeline as Code | CI/CD defined in repo (GitHub Actions, Jenkinsfile) |

### 12. Agentic AI Patterns

| Pattern | Description | ECC Relevance |
|---------|-------------|---------------|
| ReAct (Reason + Act) | Alternate thought-action-observation loops | Core agent loop |
| Reflection | Self-evaluation and revision cycle | Adversary agents, /review |
| Tool Use | Structured tool calling with schema validation | All ECC agents |
| Planning | Decompose complex tasks into structured roadmaps | /spec, /design, /implement |
| Multi-Agent Collaboration | Specialized agents coordinating on shared tasks | Wave dispatch, team coordination |
| Human-in-the-Loop | Strategic checkpoints for human review | Plan Mode, grill-me |
| Memory & Context | Persistent knowledge across sessions | BL-093 memory system |
| Guardrails | Safety constraints and output validation | Hooks, adversary agents |

### 13. Rust-Specific Patterns (beyond GoF)

| Pattern | Description |
|---------|-------------|
| Newtype | Zero-cost wrapper for type safety (`struct Meters(f64)`) |
| Typestate | Compile-time state machine via phantom types |
| Builder (typestate) | Enforce required fields at compile time |
| Enum Dispatch | Replace visitor/strategy with `match` on enum variants |
| Interior Mutability | `RefCell`, `Cell`, `Mutex` for controlled mutation |
| RAII / Drop | Resource cleanup via `Drop` trait |
| Deref Coercion | Smart pointer ergonomics |
| Extension Trait | Add methods to foreign types |
| Error Enum | `thiserror`-derived error type hierarchies |
| `From`/`Into` Conversion | Idiomatic type conversion |

### 14. Functional Programming Patterns

| Pattern | Description | Languages |
|---------|-------------|-----------|
| Map/Filter/Reduce | Collection transformation pipeline | All (iterators, streams, LINQ) |
| Monad (Option/Result/Either) | Composable computation with context | Rust, Haskell, Scala, Kotlin (Arrow), TS (fp-ts) |
| Functor/Applicative | Lift functions over wrapped values | FP-heavy languages |
| Algebraic Data Types | Sum types (enum) + product types (struct) | Rust, Haskell, Scala, Kotlin sealed, TS union |
| Pattern Matching | Destructure and branch on data shape | Rust, Scala, Kotlin when, Python match, TS narrowing |
| Immutable Data Structures | Persistent data structures | All (Rust by default, Immer.js, PCollections) |
| Currying/Partial Application | Function specialization | JS/TS, Python functools, Haskell native |
| Lenses/Optics | Composable nested data access/update | Haskell, Scala (Monocle), TS (monocle-ts) |

### 15. Data Access Patterns

| Pattern | Description | Languages |
|---------|-------------|-----------|
| Repository | Abstract interface over data store | All |
| Unit of Work | Track changes, commit atomically | Java (JPA), Python (SQLAlchemy), C# (EF) |
| Active Record | Object wraps a database row | Ruby (Rails), Python (Django ORM) |
| Data Mapper | Separate domain objects from DB schema | Java (Hibernate), TS (TypeORM/MikroORM) |
| Query Builder | Programmatic query construction | All (SQLx, GORM, Prisma, QueryDSL) |
| Connection Pool | Reuse database connections | All (HikariCP, pgbouncer, sqlx pool) |

## Ready-to-Paste Prompt

```
Build an internal Pattern Library for agent-assisted development in ECC.

## Objective

Create a structured, searchable collection of software patterns as markdown files
that ECC agents can discover and reference during code generation, review,
refactoring, and architecture work. Each pattern includes language-specific
idiomatic implementations where relevant.

## Pattern Categories (15 categories, ~150+ patterns total)

1. **GoF Design Patterns** (23): Creational (5), Structural (7), Behavioral (11)
   - Language variants: Rust, Go, Python, TypeScript, Java/Kotlin, C#, C++
   - Highlight idiomatic alternatives (e.g., Rust enum dispatch vs Visitor, Go functional options vs Builder)

2. **Architecture Patterns** (10+): Hexagonal, Clean, CQRS, Event Sourcing, Microservices, Modular Monolith, Saga, Outbox, Strangler Fig, BFF
   - Cross-reference with ECC's own hexagonal architecture

3. **Concurrency Patterns** (6+): Async/Await, CSP/Channels, Actor Model, Thread Pool, RwLock, Fan-out/Fan-in
   - Language matrix: Rust (tokio), Go (goroutines), Python (asyncio), Kotlin (coroutines), Java (virtual threads)

4. **Error Handling Patterns** (5+): Result/Either, Railway Oriented, Error Wrapping, Structured Errors, Panic/Recover
   - Language matrix with idiomatic approaches per language

5. **Resilience Patterns** (6): Circuit Breaker, Retry+Backoff, Bulkhead, Timeout, Fallback, Rate Limiter
   - Include recommended library per language

6. **Testing Patterns** (10+): AAA, GWT, Test Doubles, Property-Based, Mutation, Contract, Snapshot, Table-Driven, Testcontainers, Approval
   - Framework recommendations per language

7. **DDD Tactical Patterns** (8): Aggregate Root, Value Object, Entity, Domain Event, Repository, Specification, Anti-Corruption Layer, Domain Service
   - Idiomatic implementation per language

8. **API Design Patterns** (10+): REST Resources, Pagination, Rate Limiting, Versioning, Idempotency, GraphQL Schema, gRPC, API Gateway, HATEOAS, Webhooks

9. **Security Patterns** (8+): Input Validation, AuthN/AuthZ, Secrets Management, CSRF, SQLi Prevention, XSS, CSP
   - OWASP-aligned, framework-specific implementations

10. **Observability Patterns** (6): Structured Logging, Distributed Tracing, Metrics, Correlation ID, Health Checks, Log Aggregation
    - OpenTelemetry integration per language

11. **CI/CD & DevOps Patterns** (8): Blue-Green, Canary, Feature Flags, Rolling Update, IaC, GitOps, Trunk-Based Dev, Pipeline as Code

12. **Agentic AI Patterns** (8): ReAct, Reflection, Tool Use, Planning, Multi-Agent, Human-in-the-Loop, Memory, Guardrails
    - ECC-specific integration points

13. **Language-Specific Idioms**: Patterns unique to a language's type system or runtime
    - Rust: Newtype, Typestate, Enum Dispatch, Interior Mutability, RAII, Extension Trait
    - Go: Functional Options, Error Wrapping, Context Propagation, Table Tests
    - Python: Decorators, Context Managers, Generators, Metaclasses, Descriptors
    - TypeScript: Discriminated Unions, Branded Types, Type Guards, Conditional Types
    - Kotlin: Sealed Classes, DSL Builders, Coroutine Scope, Delegation

14. **Functional Programming Patterns** (8): Map/Filter/Reduce, Monads, Algebraic Data Types, Pattern Matching, Immutable Data, Currying, Lenses
    - Relevance matrix by language

15. **Data Access Patterns** (6): Repository, Unit of Work, Active Record, Data Mapper, Query Builder, Connection Pool

## File Structure

```
patterns/
  index.md                     # Master index with categories and tags
  creational/
    factory-method.md
    builder.md
    ...
  structural/
    adapter.md
    ...
  behavioral/
    strategy.md
    ...
  architecture/
    hexagonal.md
    cqrs.md
    ...
  concurrency/
    async-await.md
    channels-csp.md
    ...
  error-handling/
    result-either.md
    ...
  resilience/
    circuit-breaker.md
    ...
  testing/
    property-based.md
    ...
  ddd/
    aggregate-root.md
    ...
  api-design/
    rest-resources.md
    ...
  security/
    input-validation.md
    ...
  observability/
    structured-logging.md
    ...
  cicd/
    blue-green.md
    ...
  agentic/
    react-pattern.md
    ...
  functional/
    monads.md
    ...
  data-access/
    repository.md
    ...
  idioms/
    rust/
    go/
    python/
    typescript/
    kotlin/
```

## Pattern File Schema

Each pattern file follows this structure:
- YAML frontmatter: name, category, tags, languages, difficulty, related-patterns
- Intent: one-sentence purpose
- Problem: what problem it solves
- Solution: language-agnostic description
- Language Implementations: code snippets per language (only languages where pattern is idiomatic)
- When to Use / When NOT to Use
- Anti-Patterns: common mistakes
- Related Patterns: cross-references
- References: links to authoritative sources

## Agent Discovery Integration

- Each pattern category maps to an ECC skill
- Agents declare `skills: ["patterns/resilience"]` in frontmatter to preload relevant patterns
- `/spec-*` commands auto-detect language and inject relevant pattern skills
- Code reviewers reference patterns when flagging issues
- Pattern index is searchable via `ecc memory search` or direct file glob

## Existing ECC Skills to Consolidate

The library should subsume and extend: rust-patterns, python-patterns, golang-patterns,
kotlin-patterns, csharp-patterns, cpp-coding-standards, api-design, security-review,
tdd-workflow, clean-craft, architecture-review, error-handling-audit, observability-audit,
deployment-patterns, docker-patterns, ci-cd-workflows, agentic-engineering

## Languages Covered

Primary: Rust, Go, Python, TypeScript, Java, Kotlin
Secondary: C#, C++, Swift, Shell
Per-pattern: only include languages where the pattern is idiomatic or has a meaningful variant
```
