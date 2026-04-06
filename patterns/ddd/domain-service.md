---
name: domain-service
category: ddd
tags: [ddd, domain-driven-design, domain, service]
languages: [rust, go, python, typescript, java]
difficulty: advanced
---

## Intent

Encapsulate domain logic that does not naturally belong to any single entity or value object.

## Problem

Some business operations span multiple aggregates or require coordination that would violate single-entity responsibility.

## Solution

Create a stateless service in the domain layer that orchestrates operations across entities. It depends only on domain types and repository interfaces.

## Language Implementations

### Rust
```rust
// Idiomatic domain service implementation
pub trait domainserviceRepository: Send + Sync {
    fn find_by_id(&self, id: Id) -> Result<Option<Entity>>;
    fn save(&self, entity: &Entity) -> Result<()>;
}
```

### Go
```go
type domainserviceRepository interface {
    FindByID(ctx context.Context, id string) (*Entity, error)
    Save(ctx context.Context, entity *Entity) error
}
```

### Python
```python
from abc import ABC, abstractmethod
class domainserviceRepository(ABC):
    @abstractmethod
    def find_by_id(self, id: str) -> Entity | None: ...
```

## When to Use

- Domain-driven design projects with complex business logic.
- Systems with multiple bounded contexts requiring clean separation.

## When NOT to Use

- Simple CRUD applications with minimal business logic.
- Prototypes where speed matters more than architecture.

## Anti-Patterns

- Anemic domain model where all logic lives in services.
- Repositories that expose raw SQL or ORM queries.

## Related Patterns

- ddd/aggregate-root
- ddd/entity
- ddd/value-object

## References

- Eric Evans, Domain-Driven Design (2003).
- Vaughn Vernon, Implementing Domain-Driven Design (2013).
