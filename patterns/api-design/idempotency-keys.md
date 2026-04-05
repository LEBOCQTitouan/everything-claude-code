---
name: idempotency-keys
category: api-design
tags: [api-design, idempotency, reliability, retry-safety]
languages: [all]
difficulty: intermediate
---

## Intent

Make non-idempotent operations (POST, PATCH) safely retryable by associating each request with a unique client-generated key, preventing duplicate side effects.

## Problem

Network failures, timeouts, and client retries can cause the same request to arrive multiple times. Without idempotency protection, a retried payment creates a double charge, a retried order creates duplicate items, and a retried email sends twice.

## Solution

Require clients to send a unique `Idempotency-Key` header with every mutating request. The server stores the key alongside the response for a retention window. On duplicate keys, the server returns the stored response without re-executing the operation.

## Language Implementations

### HTTP Contract

```
POST /payments
Idempotency-Key: idk_a1b2c3d4e5f6
Content-Type: application/json

{ "amount": 5000, "currency": "usd" }

# First request: processes payment, stores result keyed by idk_a1b2c3d4e5f6
# Retry with same key: returns stored result, 200 OK (no double charge)
# Different key: processes as new payment
```

### Server-Side Flow (Pseudocode)

```
fn handle_request(key, request):
    # Check for existing result
    existing = store.get(key)
    if existing is not None:
        if existing.status == "processing":
            return 409 Conflict  # concurrent duplicate
        return existing.response

    # Lock the key
    store.set(key, { status: "processing", created_at: now() })

    try:
        result = process(request)
        store.set(key, { status: "complete", response: result, created_at: now() })
        return result
    except error:
        store.set(key, { status: "failed", error: error, created_at: now() })
        raise error
```

### Key Generation (Client-Side)

```
# UUIDv4 — simple and unique
Idempotency-Key: 550e8400-e29b-41d4-a716-446655440000

# Deterministic — derived from business context (preferred for retries)
Idempotency-Key: sha256(user_id + order_id + amount + timestamp_bucket)
```

### Storage Schema

```yaml
idempotency_keys:
  key: string (primary key)
  status: enum(processing, complete, failed)
  request_hash: string   # detect mismatched payloads for same key
  response_code: int
  response_body: blob
  created_at: timestamp
  ttl: 24h              # auto-expire after retention window
```

## When to Use

- Payment processing, order creation, and any operation with financial side effects.
- Webhook delivery where the receiver must deduplicate events.
- Any POST endpoint where retries are expected (mobile clients, unreliable networks).

## When NOT to Use

- GET, PUT, and DELETE requests that are already idempotent by HTTP semantics.
- Internal synchronous calls where the caller can verify success before retrying.

## Anti-Patterns

- Accepting the same idempotency key with a different request body — always hash-check the payload.
- Using server-generated keys instead of client-generated keys — defeats retry safety.
- Setting TTL too short — clients that retry after a long timeout will trigger duplicate processing.
- Storing idempotency keys only in memory — server restart causes lost deduplication state.

## Related Patterns

- [rest-resources](rest-resources.md) — idempotency keys apply to resource creation endpoints.
- [rate-limiting](rate-limiting.md) — retried requests with the same key should not count against rate limits.
- [webhooks](webhooks.md) — webhook receivers use idempotency keys to deduplicate deliveries.

## References

- Stripe Idempotent Requests: https://stripe.com/docs/api/idempotent_requests
- IETF Idempotency-Key Header: https://datatracker.ietf.org/doc/draft-ietf-httpapi-idempotency-key-header/
- Brandur, Implementing Stripe-like Idempotency Keys: https://brandur.org/idempotency-keys
