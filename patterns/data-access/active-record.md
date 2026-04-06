---
name: active-record
category: data-access
tags: [data-access, persistence, active, record]
languages: [python, ruby]
difficulty: intermediate
---

## Intent

Wrap a database row in a domain object that knows how to persist itself.

## Problem

Simple CRUD applications need a quick way to map objects to database rows without the ceremony of separate repository and mapper layers.

## Solution

Each model class inherits from a base that provides save(), delete(), and query methods. The object IS the row — fields map directly to columns.

## Language Implementations

### Rust
```rust
// sqlx / diesel / sea-orm
let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", id)
    .fetch_optional(&pool).await?;
```

### Python
```python
# SQLAlchemy / Django ORM
user = session.query(User).filter_by(id=id).first()
```

### Java
```java
// Hibernate / JPA / Spring Data
Optional<User> user = userRepository.findById(id);
```

## When to Use

- Applications with database persistence requirements.
- Systems needing testable data access layers.

## When NOT to Use

- In-memory-only applications or CLI tools without persistence.
- Prototypes where direct SQL is faster to write.

## Anti-Patterns

- Exposing ORM internals in domain layer.
- N+1 query problems from lazy loading.

## Related Patterns

- data-access/repository
- ddd/aggregate-root

## References

- Martin Fowler, Patterns of Enterprise Application Architecture.
- sqlx (Rust). SQLAlchemy (Python). Hibernate (Java). Prisma (TypeScript).
