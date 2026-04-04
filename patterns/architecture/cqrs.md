---
name: cqrs
category: architecture
tags: [architecture, cqrs, read-model, write-model, event-sourcing]
languages: [all]
difficulty: advanced
---

## Intent

Separate the model used to update information (Command side) from the model used to read information (Query side). Each side is optimised independently for its specific concern.

## Problem

A single unified model serves both reads and writes poorly. Write operations require complex aggregate validation and invariant enforcement. Read operations require denormalised, joined, or projected views optimised for query patterns. Forcing both through the same model creates impedance mismatch and performance bottlenecks.

## Solution

Define separate Command and Query stacks. Commands mutate state; they go through the domain aggregate and produce events or persist via a write model. Queries read from a separate read model that may be a projection, a materialised view, or a dedicated read database. The two stacks communicate asynchronously (via events) or synchronously depending on consistency requirements.

```
Client
  │
  ├──Command──► Command Handler ──► Aggregate ──► Write Store
  │                                     │
  │                              Domain Events
  │                                     │
  │                              Event Handler ──► Read Store (projection)
  │
  └──Query───► Query Handler ──► Read Store ──► DTO ──► Client
```

## Language Implementations

### all

Command side (write):

```
CommandBus
  └── CreateOrderCommand
        └── CreateOrderHandler
              └── Order aggregate (domain logic + invariant checks)
                    └── OrderRepository (write port)
                          └── PostgreSQL (write adapter)
```

Query side (read):

```
QueryBus
  └── GetOrderSummaryQuery
        └── GetOrderSummaryHandler
              └── OrderReadModel (read port — no domain objects)
                    └── Read store adapter (Postgres view, Redis, Elasticsearch)
```

Key design choices:

- **Consistency model**: synchronous projection (strong consistency) vs. eventual (async event-driven).
- **Shared database**: write and read stores can share the same DB with separate tables/views, or use different stores.
- **Event Sourcing** (optional): store events as the source of truth; derive read models as projections of the event log.

## When to Use

- When read and write workloads have very different scaling or latency requirements.
- When the write model is complex (rich aggregates, invariants) but reads need flat, denormalised projections.
- When combined with Event Sourcing to produce auditable history and time-travel queries.

## When NOT to Use

- For simple CRUD applications where read and write models are nearly identical.
- When eventual consistency is unacceptable and the team cannot manage synchronisation complexity.
- When the team is small and the added complexity outweighs the benefit.

## Anti-Patterns

- Querying the write model from the read side — reintroduces coupling and bypasses the read store.
- Making commands return data (except a minimal acknowledgement/ID) — commands should be fire-and-notify, not query.
- Implementing CQRS "just in case" without a clear scaling or modelling driver.

## Related Patterns

- [hexagonal](hexagonal.md) — provides the port/adapter structure within which CQRS operates.
- [clean-architecture](clean-architecture.md) — command handlers live in the Use Cases ring; read models in Interface Adapters.

## References

- Martin Fowler — CQRS: https://martinfowler.com/bliki/CQRS.html
- Greg Young — CQRS Documents: https://cqrs.files.wordpress.com/2010/11/cqrs_documents.pdf
