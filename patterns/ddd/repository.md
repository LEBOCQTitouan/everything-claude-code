---
name: repository
category: ddd
tags: [ddd, domain-driven-design, repository]
languages: [rust, go, python, typescript, java]
difficulty: advanced
---

## Intent

Abstract data access behind a consistent interface, decoupling domain logic from storage details.

## Problem

Domain logic depends directly on database queries, making it impossible to test without a database and coupling business rules to SQL.

## Solution

Define a repository trait/interface per aggregate root with standard operations (find_by_id, save, delete). Implement with concrete storage adapters.

## Language Implementations

### Rust
```rust
// Idiomatic repository implementation
pub trait repositoryRepository: Send + Sync {
    fn find_by_id(&self, id: Id) -> Result<Option<Entity>>;
    fn save(&self, entity: &Entity) -> Result<()>;
}
```

### Go
```go
type repositoryRepository interface {
    FindByID(ctx context.Context, id string) (*Entity, error)
    Save(ctx context.Context, entity *Entity) error
}
```

### Python
```python
from abc import ABC, abstractmethod
class repositoryRepository(ABC):
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
