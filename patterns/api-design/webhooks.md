---
name: webhooks
category: api-design
tags: [api-design, webhooks, events, push, async]
languages: [all]
difficulty: intermediate
---

## Intent

Push event notifications to external systems in real time via HTTP callbacks, eliminating the need for clients to poll for changes.

## Problem

Polling wastes resources: clients repeatedly hit endpoints asking "anything new?" when the answer is usually "no." Polling intervals create a latency floor — events are discovered only at the next poll cycle. High-frequency polling strains both client and server.

## Solution

Register callback URLs where the server POSTs event payloads when state changes occur. Include cryptographic signatures for authenticity verification, idempotency keys for deduplication, and retry logic with exponential backoff for failed deliveries.

## Language Implementations

### Webhook Registration

```json
POST /webhooks
{
  "url": "https://partner.example.com/hooks/orders",
  "events": ["order.created", "order.shipped", "order.cancelled"],
  "secret": "whsec_abc123def456"
}

Response:
{
  "id": "wh_789",
  "url": "https://partner.example.com/hooks/orders",
  "events": ["order.created", "order.shipped", "order.cancelled"],
  "status": "active",
  "created_at": "2026-01-15T10:00:00Z"
}
```

### Event Payload

```json
POST https://partner.example.com/hooks/orders
Content-Type: application/json
X-Webhook-ID: evt_abc123
X-Webhook-Timestamp: 1700000000
X-Webhook-Signature: sha256=a1b2c3d4e5f6...

{
  "id": "evt_abc123",
  "type": "order.created",
  "created_at": "2026-01-15T10:05:00Z",
  "data": {
    "id": "order_456",
    "customer_id": "u_789",
    "total_cents": 5000,
    "status": "pending"
  }
}
```

### Signature Verification (Receiver Side)

```
fn verify_signature(payload, timestamp, signature, secret):
    signed_content = f"{timestamp}.{payload}"
    expected = hmac_sha256(secret, signed_content)
    if not constant_time_equal(expected, signature):
        return 401 Unauthorized
    if abs(now() - timestamp) > 300:  # 5 min tolerance
        return 403 Forbidden  # replay attack
    return verified
```

### Retry Policy

```yaml
retry_policy:
  max_attempts: 5
  backoff: exponential
  schedule: [1m, 5m, 30m, 2h, 24h]
  timeout_per_attempt: 30s
  success_codes: [200, 201, 202, 204]
  disable_after: 7d_consecutive_failures
  alert_after: 3_consecutive_failures
```

## When to Use

- Notifying external systems of state changes (payments, shipments, user events).
- Integrations where partners need real-time data without API access.
- Event-driven architectures extending beyond your service boundary.

## When NOT to Use

- Internal microservice events — use a message broker (Kafka, RabbitMQ) instead.
- When the receiver requires guaranteed exactly-once delivery — webhooks are at-least-once.
- High-throughput event streams (thousands per second) — use streaming protocols.

## Anti-Patterns

- Not signing payloads — receivers cannot verify authenticity, enabling spoofed events.
- Blocking on webhook delivery in the request path — always deliver asynchronously.
- Sending the full resource in every event instead of a reference — bloats payloads and leaks data.
- Not providing a way for receivers to replay missed events (offer an event log endpoint).

## Related Patterns

- [idempotency-keys](idempotency-keys.md) — receivers use webhook event IDs for deduplication.
- [rate-limiting](rate-limiting.md) — apply rate limits to webhook registration endpoints.
- [rest-resources](rest-resources.md) — webhook management is itself a REST resource.
- [api-gateway](api-gateway.md) — gateway can route webhook delivery through egress controls.

## References

- Standard Webhooks Specification: https://www.standardwebhooks.com/
- Stripe Webhooks: https://stripe.com/docs/webhooks
- GitHub Webhooks: https://docs.github.com/en/webhooks
- Svix Webhook Best Practices: https://docs.svix.com/receiving/introduction
