---
name: anti-corruption-layer
category: ddd
tags: [ddd, domain-driven-design, anti, corruption, layer]
languages: [rust, go, python, typescript]
difficulty: advanced
---

## Intent

Translate between bounded contexts to prevent one model from corrupting another.

## Problem

When integrating with legacy systems or external APIs, their data models leak into your domain, creating tight coupling and model degradation.

## Solution

Place an adapter layer at the boundary that translates external models into your domain language. The ACL isolates your domain from upstream changes.

## Language Implementations

### Rust
```rust
// Idiomatic anti corruption layer implementation
pub trait anticorruptionlayerRepository: Send + Sync {
    fn find_by_id(&self, id: Id) -> Result<Option<Entity>>;
    fn save(&self, entity: &Entity) -> Result<()>;
}
```

### Go
```go
type anticorruptionlayerRepository interface {
    FindByID(ctx context.Context, id string) (*Entity, error)
    Save(ctx context.Context, entity *Entity) error
}
```

### Python
```python
from abc import ABC, abstractmethod
class anticorruptionlayerRepository(ABC):
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
