---
name: testcontainers
category: testing
tags: [testing, integration, containers, docker]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Run real infrastructure dependencies (databases, message brokers, caches) in disposable Docker containers during tests, providing realistic integration testing without permanent infrastructure.

## Problem

Unit tests with mocks miss integration bugs. Shared test databases cause flaky tests and data collisions. Setting up local infrastructure manually is error-prone and differs across developer machines.

## Solution

Use the Testcontainers library to programmatically start Docker containers before tests and tear them down afterward. Each test suite gets a fresh, isolated instance of the real dependency with a random port mapping.

## Language Implementations

### Rust

```rust
use testcontainers::{clients, images::postgres::Postgres};

#[tokio::test]
async fn test_user_repository() {
    let docker = clients::Cli::default();
    let pg = docker.run(Postgres::default());
    let port = pg.get_host_port_ipv4(5432);
    let url = format!("postgres://postgres:postgres@localhost:{port}/postgres");

    let pool = PgPool::connect(&url).await.unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();

    let repo = PgUserRepository::new(pool);
    repo.create(&User::new("alice")).await.unwrap();

    let found = repo.find_by_name("alice").await.unwrap();
    assert_eq!(found.name, "alice");
}
```

### Go

```go
func TestUserRepository(t *testing.T) {
    ctx := context.Background()
    pg, err := postgres.Run(ctx, "postgres:16",
        postgres.WithDatabase("test"),
        postgres.WithUsername("test"),
        postgres.WithPassword("test"),
        testcontainers.WithWaitStrategy(
            wait.ForLog("ready to accept connections").WithOccurrence(2),
        ),
    )
    require.NoError(t, err)
    defer pg.Terminate(ctx)

    connStr, _ := pg.ConnectionString(ctx, "sslmode=disable")
    repo := NewPgUserRepository(connStr)

    err = repo.Create(ctx, &User{Name: "alice"})
    require.NoError(t, err)

    found, err := repo.FindByName(ctx, "alice")
    require.NoError(t, err)
    assert.Equal(t, "alice", found.Name)
}
```

### Python

```python
import pytest
from testcontainers.postgres import PostgresContainer

@pytest.fixture(scope="module")
def pg():
    with PostgresContainer("postgres:16") as postgres:
        yield postgres

def test_user_repository(pg):
    engine = create_engine(pg.get_connection_url())
    Base.metadata.create_all(engine)

    repo = UserRepository(engine)
    repo.create(User(name="alice"))

    found = repo.find_by_name("alice")
    assert found.name == "alice"
```

### Typescript

```typescript
import { PostgreSqlContainer } from "@testcontainers/postgresql";

let container: StartedPostgreSqlContainer;

beforeAll(async () => {
  container = await new PostgreSqlContainer("postgres:16").start();
});

afterAll(async () => { await container.stop(); });

test("user repository", async () => {
  const pool = new Pool({ connectionString: container.getConnectionUri() });
  const repo = new UserRepository(pool);

  await repo.create({ name: "alice" });
  const found = await repo.findByName("alice");

  expect(found.name).toBe("alice");
  await pool.end();
});
```

## When to Use

- For integration tests that need real database behaviour (SQL dialects, constraints, transactions).
- For testing message broker interactions (Kafka, RabbitMQ, Redis).
- When mocks cannot replicate the dependency's behaviour accurately.

## When NOT to Use

- For unit tests where a mock or fake is sufficient and faster.
- In CI environments without Docker support.
- When startup time is a bottleneck (consider reusing containers across tests).

## Anti-Patterns

- Starting a new container per test instead of per suite (very slow).
- Not cleaning up containers, leading to Docker resource exhaustion.
- Hard-coding ports instead of using dynamic port mapping.

## Related Patterns

- [testing/test-doubles](test-doubles.md) -- use doubles for unit tests, containers for integration.
- [testing/contract](contract.md) -- run provider contract tests against real containers.
- [testing/aaa](aaa.md) -- container setup is the Arrange phase of integration tests.

## References

- Testcontainers: https://testcontainers.com
- **Rust**: `testcontainers` crate
- **Go**: `testcontainers-go`
- **Python**: `testcontainers-python`
- **Kotlin/Java**: `testcontainers-java` (the original)
- **TypeScript**: `@testcontainers/postgresql`, `@testcontainers/redis`
