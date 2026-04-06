---
name: contract
category: testing
tags: [testing, contract, integration, consumer-driven]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Verify that a service honours its API contract -- the agreed-upon request/response format, status codes, and behaviour -- so that consumers and producers can evolve independently with confidence.

## Problem

Integration tests that call real services are slow and brittle. Mock-based tests can drift from the actual API. When a producer changes its API, consumers break at runtime because there was no shared contract enforcement.

## Solution

Define contracts (expected request/response pairs) shared between consumer and producer. The consumer generates contract expectations; the producer verifies it satisfies them. Both sides run contract tests independently in their own CI pipelines.

## Language Implementations

### Rust

```rust
// Shared contract definition (e.g., via Pact or manual schema)
#[test]
fn user_service_honours_get_user_contract() {
    let expected = json!({
        "id": 1,
        "name": "Alice",
        "email": "alice@example.com"
    });

    let response = test_client.get("/users/1").send().unwrap();
    let body: serde_json::Value = response.json().unwrap();

    // Verify structure matches contract
    assert_eq!(body["id"].as_i64(), expected["id"].as_i64());
    assert!(body["name"].is_string());
    assert!(body["email"].is_string());
}
```

### Go

```go
func TestUserServiceContract(t *testing.T) {
    // Provider verification with Pact
    pact := dsl.Pact{
        Consumer: "order-service",
        Provider: "user-service",
    }

    pact.AddInteraction().
        Given("user alice exists").
        UponReceiving("a request for user 1").
        WithRequest(dsl.Request{Method: "GET", Path: dsl.String("/users/1")}).
        WillRespondWith(dsl.Response{
            Status:  200,
            Headers: dsl.MapMatcher{"Content-Type": dsl.String("application/json")},
            Body:    dsl.Match(User{}),
        })

    pact.Verify(t, func() { /* start provider */ })
}
```

### Python

```python
import pytest
from pact import Consumer, Provider

@pytest.fixture
def pact():
    return Consumer("order-service").has_pact_with(Provider("user-service"))

def test_get_user_contract(pact):
    (pact
        .given("user alice exists")
        .upon_receiving("a request for user 1")
        .with_request("GET", "/users/1")
        .will_respond_with(200, body={
            "id": 1,
            "name": Like("Alice"),
            "email": Like("alice@example.com"),
        }))

    with pact:
        result = user_client.get_user(1)
        assert result.name == "Alice"
```

### Typescript

```typescript
import { PactV3, MatchersV3 } from "@pact-foundation/pact";

const provider = new PactV3({ consumer: "order-service", provider: "user-service" });

test("get user contract", async () => {
  await provider
    .given("user alice exists")
    .uponReceiving("a request for user 1")
    .withRequest({ method: "GET", path: "/users/1" })
    .willRespondWith({
      status: 200,
      body: { id: MatchersV3.integer(1), name: MatchersV3.string("Alice") },
    })
    .executeTest(async (mockServer) => {
      const user = await getUser(mockServer.url, 1);
      expect(user.name).toBe("Alice");
    });
});
```

## When to Use

- When multiple teams own consumer and producer independently.
- When API compatibility must be verified without full integration tests.
- When you need to prevent breaking changes in shared APIs.

## When NOT to Use

- For internal module boundaries within a single codebase (use unit tests).
- When there is only one consumer and you control both sides.

## Anti-Patterns

- Writing contracts that are too strict (exact timestamps, random IDs).
- Not running provider verification in the producer's CI pipeline.
- Letting contracts become stale and diverge from actual usage.

## Related Patterns

- [testing/test-doubles](test-doubles.md) -- contracts validate that doubles match reality.
- [testing/testcontainers](testcontainers.md) -- run real providers for full integration verification.
- [testing/snapshot](snapshot.md) -- snapshot response shapes as a lightweight contract.

## References

- Pact Foundation: https://pact.io
- Martin Fowler, "Consumer-Driven Contracts": https://martinfowler.com/articles/consumerDrivenContracts.html
- **Rust**: `pact_consumer`, `pact_verifier`
- **Go**: `pact-go`
- **Python**: `pact-python`
- **Java/Kotlin**: `pact-jvm`, Spring Cloud Contract
- **TypeScript**: `@pact-foundation/pact`
