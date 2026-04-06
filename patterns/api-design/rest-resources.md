---
name: rest-resources
category: api-design
tags: [api-design, rest, resources, http, crud]
languages: [all]
difficulty: beginner
---

## Intent

Model API endpoints around domain resources using standard HTTP methods, giving clients a uniform, predictable interface for data manipulation.

## Problem

APIs that expose actions as RPC-style verbs (`/getUser`, `/createOrder`) lack consistency. Each endpoint invents its own contract, making the API hard to learn and impossible to navigate with generic tooling. Clients must memorize bespoke URLs and payloads for every operation.

## Solution

Map domain entities to URI-addressable resources. Use plural nouns for collections (`/orders`) and hierarchical paths for relationships (`/orders/{id}/items`). Let HTTP methods convey the operation: GET reads, POST creates, PUT replaces, PATCH updates partially, DELETE removes.

## Language Implementations

### HTTP Contract (Protocol-Agnostic)

```
GET    /users          # List collection (filterable via query params)
POST   /users          # Create resource (server assigns ID)
GET    /users/{id}     # Retrieve single resource
PUT    /users/{id}     # Full replacement
PATCH  /users/{id}     # Partial update (JSON Merge Patch or JSON Patch)
DELETE /users/{id}     # Remove resource

# Sub-resources for relationships
GET    /users/{id}/orders
POST   /users/{id}/orders
```

### Response Envelope

```json
{
  "data": { "id": "u_123", "name": "Alice", "email": "alice@example.com" },
  "meta": { "request_id": "req_abc" }
}
```

### Error Envelope

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Email is required",
    "details": [{ "field": "email", "reason": "must not be blank" }]
  }
}
```

## When to Use

- Public or partner-facing APIs where discoverability and tooling compatibility matter.
- CRUD-dominated domains where entities map naturally to resources.
- When HTTP caching via standard headers provides significant performance gains.

## When NOT to Use

- When the domain is action-oriented (e.g., batch ETL triggers) and forcing CRUD semantics distorts intent.
- Real-time bidirectional communication (prefer WebSockets or gRPC streaming).
- Internal microservice-to-microservice calls where RPC semantics and binary protocols are more efficient.

## Anti-Patterns

- Using verbs in URIs (`/createUser`) instead of relying on HTTP methods.
- Returning 200 for every response and encoding the real status in the body.
- Exposing database schema directly as API resources without a domain mapping layer.
- Ignoring plural/singular consistency (`/user` vs `/users`).

## Related Patterns

- [pagination](pagination.md) — paginate collection endpoints returned by resource listings.
- [hateoas](hateoas.md) — embed navigation links in resource representations.
- [versioning](versioning.md) — evolve resource schemas without breaking clients.
- [idempotency-keys](idempotency-keys.md) — make POST creation requests safely retryable.

## References

- Fielding, R. Architectural Styles and the Design of Network-Based Software Architectures (2000), Chapter 5.
- Microsoft REST API Guidelines: https://github.com/microsoft/api-guidelines
- Google API Design Guide: https://cloud.google.com/apis/design
