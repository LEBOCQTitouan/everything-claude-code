---
name: pagination
category: api-design
tags: [api-design, rest, pagination, cursor, offset]
languages: [all]
difficulty: beginner
---

## Intent

Break large collection responses into manageable pages so clients can fetch data incrementally without overwhelming server memory or network bandwidth.

## Problem

Returning an entire collection in a single response causes unbounded memory usage on the server, slow network transfers, and poor client rendering performance. Without pagination, any collection endpoint is a latent scalability bomb.

## Solution

Offer two pagination strategies: offset-based (simple, supports random access) and cursor-based (stable under concurrent writes, better for large datasets). Return pagination metadata alongside the data array.

## Language Implementations

### Offset-Based (REST)

```
GET /orders?page=2&per_page=25

Response:
{
  "data": [...],
  "meta": {
    "page": 2,
    "per_page": 25,
    "total_count": 243,
    "total_pages": 10
  }
}
```

### Cursor-Based (REST)

```
GET /orders?after=cursor_abc123&limit=25

Response:
{
  "data": [...],
  "meta": {
    "has_next_page": true,
    "next_cursor": "cursor_def456",
    "has_previous_page": true,
    "previous_cursor": "cursor_xyz789"
  }
}
```

### GraphQL Connection Pattern

```graphql
type Query {
  orders(first: Int, after: String): OrderConnection!
}

type OrderConnection {
  edges: [OrderEdge!]!
  pageInfo: PageInfo!
}

type OrderEdge {
  node: Order!
  cursor: String!
}

type PageInfo {
  hasNextPage: Boolean!
  endCursor: String
}
```

## When to Use

- Any collection endpoint that can return more than a screenful of results.
- When the dataset grows over time and unbounded responses will eventually fail.
- When clients need to display data in pages or implement infinite scroll.

## When NOT to Use

- Collections guaranteed to remain small (fewer than 50 items) and stable.
- When the entire dataset must be processed atomically (use streaming or batch export instead).

## Anti-Patterns

- Defaulting to no limit when the client omits pagination params — always enforce a server-side max.
- Using offset pagination on datasets with frequent inserts, causing items to shift between pages.
- Encoding database row IDs as cursors without opaque encoding, leaking internal identifiers.

## Related Patterns

- [rest-resources](rest-resources.md) — pagination applies to resource collection endpoints.
- [rate-limiting](rate-limiting.md) — combine with pagination to control total throughput.
- [graphql-schema](graphql-schema.md) — GraphQL uses the Relay connection spec for cursor pagination.

## References

- Relay Cursor Connections Specification: https://relay.dev/graphql/connections.htm
- Slack API Pagination: https://api.slack.com/docs/pagination
- Stripe API Pagination: https://stripe.com/docs/api/pagination
