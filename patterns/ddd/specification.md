---
name: specification
category: ddd
tags: [ddd, domain-driven-design, specification]
languages: [rust, go, python, java]
difficulty: advanced
---

## Intent

Encapsulate business rules as composable, reusable predicate objects that can be combined with AND, OR, NOT.

## Problem

Complex query filters and validation rules are duplicated across services, scattered in repository methods and service logic.

## Solution

Create a Specification trait/interface with is_satisfied_by(entity) method. Compose specifications using combinators (and, or, not).

## Language Implementations

### Rust
```rust
// Idiomatic specification implementation
pub trait specificationRepository: Send + Sync {
    fn find_by_id(&self, id: Id) -> Result<Option<Entity>>;
    fn save(&self, entity: &Entity) -> Result<()>;
}
```

### Go
```go
type specificationRepository interface {
    FindByID(ctx context.Context, id string) (*Entity, error)
    Save(ctx context.Context, entity *Entity) error
}
```

### Python
```python
from abc import ABC, abstractmethod
class specificationRepository(ABC):
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
