---
name: connection-pool
category: data-access
tags: [data-access, persistence, connection, pool]
languages: [rust, go, python, java]
difficulty: intermediate
---

## Intent

Maintain a pool of reusable database connections to avoid the overhead of establishing new connections per request.

## Problem

Opening a new database connection for every request is expensive (TCP handshake, TLS, authentication). Under load, this creates thousands of short-lived connections that exhaust server resources.

## Solution

Pre-allocate a fixed number of connections at startup. Requests borrow a connection from the pool and return it when done. The pool handles lifecycle, health checks, and idle timeout.

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
