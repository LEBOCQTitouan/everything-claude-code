---
name: graphql-schema
category: api-design
tags: [api-design, graphql, schema-first, type-system]
languages: [all]
difficulty: intermediate
---

## Intent

Define a strongly-typed, self-documenting API schema that lets clients request exactly the data they need in a single round trip, eliminating over-fetching and under-fetching.

## Problem

REST APIs force the server to decide what data each endpoint returns. Clients either get too much data (over-fetching) or must call multiple endpoints (under-fetching). Adding fields for one client bloats responses for all others. Mobile and web clients have vastly different data needs from the same domain.

## Solution

Define a GraphQL schema using SDL (Schema Definition Language) with types, queries, mutations, and subscriptions. Clients compose queries selecting exactly the fields they need. The schema serves as the contract and documentation simultaneously.

## Language Implementations

### Schema Definition (SDL)

```graphql
type User {
  id: ID!
  email: String!
  displayName: String!
  orders(first: Int, after: String): OrderConnection!
  createdAt: DateTime!
}

type Query {
  user(id: ID!): User
  users(filter: UserFilter, first: Int, after: String): UserConnection!
}

type Mutation {
  createUser(input: CreateUserInput!): CreateUserPayload!
  updateUser(input: UpdateUserInput!): UpdateUserPayload!
}

input CreateUserInput {
  email: String!
  displayName: String!
}

type CreateUserPayload {
  user: User
  errors: [UserError!]!
}

type UserError {
  field: String!
  message: String!
}
```

### Naming Conventions

```graphql
# Inputs: <Action><Type>Input
input CreateOrderInput { ... }

# Payloads: <Action><Type>Payload (include errors field)
type CreateOrderPayload { order: Order, errors: [UserError!]! }

# Connections: <Type>Connection with edges and pageInfo
type OrderConnection { edges: [OrderEdge!]!, pageInfo: PageInfo! }

# Enums: SCREAMING_SNAKE_CASE values
enum OrderStatus { PENDING, CONFIRMED, SHIPPED, DELIVERED }
```

### Query Complexity Limiting

```graphql
# Server-side depth and complexity limits
# Max depth: 7 levels
# Max complexity: 1000 points
# Field costs: scalar=0, object=1, connection=10

query TooDeep {
  user(id: "1") {          # depth 1
    orders(first: 10) {    # depth 2, cost 10
      edges { node {       # depth 4
        items { edges {    # depth 6, cost 10
          node { ... }     # depth 7 — at limit
        }}
      }}
    }
  }
}
```

## When to Use

- When multiple client types (web, mobile, internal tools) consume the same API with different data needs.
- When the domain has deeply nested, interconnected entities.
- When you want a self-documenting API with built-in introspection.

## When NOT to Use

- Simple CRUD APIs with few clients and uniform data needs — REST is simpler.
- File upload/download heavy APIs — GraphQL adds unnecessary complexity.
- When team lacks GraphQL expertise and the learning curve outweighs benefits.

## Anti-Patterns

- Exposing database schema directly as GraphQL types without a domain mapping layer.
- Allowing unbounded query depth without complexity analysis — enables DoS via deep queries.
- Using GraphQL mutations for every operation when some are better modeled as REST endpoints (file uploads).
- Returning null for errors instead of using a typed error payload.

## Related Patterns

- [pagination](pagination.md) — GraphQL uses Relay connection spec for cursor-based pagination.
- [rest-resources](rest-resources.md) — alternative API style for simpler domains.
- [rate-limiting](rate-limiting.md) — rate limit by query complexity, not just request count.
- [api-gateway](api-gateway.md) — gateway can federate multiple GraphQL subgraphs.

## References

- GraphQL Specification: https://spec.graphql.org/
- Relay Connection Specification: https://relay.dev/graphql/connections.htm
- Apollo Federation: https://www.apollographql.com/docs/federation/
- Shopify GraphQL Design Tutorial: https://github.com/Shopify/graphql-design-tutorial
