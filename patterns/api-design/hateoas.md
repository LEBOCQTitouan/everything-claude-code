---
name: hateoas
category: api-design
tags: [api-design, rest, hateoas, hypermedia, discoverability]
languages: [all]
difficulty: advanced
---

## Intent

Embed navigational links in API responses so clients can discover available actions and traverse the API dynamically, without hardcoding URL structures.

## Problem

Clients that hardcode API URLs are tightly coupled to the server's URL structure. Any URL change breaks clients. Clients must consult external documentation to know which actions are available for a given resource state, leading to stale assumptions and invalid requests.

## Solution

Include hypermedia links in every response that describe available transitions from the current resource state. Clients follow links rather than constructing URLs, making the API self-describing and allowing the server to evolve URLs freely.

## Language Implementations

### HAL Format (Hypertext Application Language)

```json
{
  "id": "order_123",
  "status": "pending",
  "total_cents": 5000,
  "_links": {
    "self":    { "href": "/orders/order_123" },
    "confirm": { "href": "/orders/order_123/confirm", "method": "POST" },
    "cancel":  { "href": "/orders/order_123/cancel", "method": "POST" },
    "customer": { "href": "/users/u_456" }
  }
}
```

### State-Dependent Links

```json
// Pending order — can confirm or cancel
{
  "status": "pending",
  "_links": {
    "self": { "href": "/orders/order_123" },
    "confirm": { "href": "/orders/order_123/confirm", "method": "POST" },
    "cancel": { "href": "/orders/order_123/cancel", "method": "POST" }
  }
}

// Confirmed order — can only ship (confirm/cancel links removed)
{
  "status": "confirmed",
  "_links": {
    "self": { "href": "/orders/order_123" },
    "ship": { "href": "/orders/order_123/ship", "method": "POST" }
  }
}
```

### Collection with Pagination Links

```json
{
  "data": [{ "id": "order_1" }, { "id": "order_2" }],
  "_links": {
    "self":  { "href": "/orders?page=2&per_page=25" },
    "first": { "href": "/orders?page=1&per_page=25" },
    "prev":  { "href": "/orders?page=1&per_page=25" },
    "next":  { "href": "/orders?page=3&per_page=25" },
    "last":  { "href": "/orders?page=10&per_page=25" }
  }
}
```

### Link Relations (IANA Registry)

```
# Standard relations
self       — canonical URL of this resource
next/prev  — pagination
related    — related resource
collection — parent collection

# Custom relations (use URI namespace)
https://api.example.com/rels/confirm
https://api.example.com/rels/ship
```

## When to Use

- APIs where resource state drives available actions (order workflows, approval chains).
- Long-lived APIs where URL structure will evolve independently of clients.
- When building APIs consumed by generic hypermedia clients or crawlers.

## When NOT to Use

- Internal APIs where clients are deployed alongside the server and URL coupling is acceptable.
- Simple CRUD APIs with no state-dependent transitions.
- When clients are written in languages/frameworks that do not have hypermedia client libraries.

## Anti-Patterns

- Including all possible links regardless of state — links must reflect current valid transitions.
- Clients ignoring links and hardcoding URLs anyway — defeats the purpose entirely.
- Overloading `_links` with dozens of relations, making responses bloated and confusing.
- Using HATEOAS without a documented link relation registry.

## Related Patterns

- [rest-resources](rest-resources.md) — HATEOAS extends REST resources with navigational links.
- [pagination](pagination.md) — pagination links are a common HATEOAS application.
- [versioning](versioning.md) — HATEOAS reduces the need for versioning since clients follow links.

## References

- Fielding, R. REST APIs must be hypertext-driven: https://roy.gbiv.com/untangled/2008/rest-apis-must-be-hypertext-driven
- HAL Specification: https://datatracker.ietf.org/doc/html/draft-kelly-json-hal
- IANA Link Relations: https://www.iana.org/assignments/link-relations/
- JSON:API Specification: https://jsonapi.org/
