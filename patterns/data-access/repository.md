---
name: repository
category: data-access
tags: [data-access, persistence, repository]
languages: [rust, go, python, typescript, java]
difficulty: intermediate
---

## Intent

Abstract data access behind a consistent interface so domain logic is decoupled from storage.

## Problem

Domain objects depend directly on database APIs, making testing require a real database and coupling business rules to specific storage technology.

## Solution

Define a repository interface per aggregate root. Implement with concrete adapters (SQL, NoSQL, in-memory). Domain code depends only on the interface.

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
