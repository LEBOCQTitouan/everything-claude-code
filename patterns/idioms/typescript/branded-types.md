---
name: branded-types
category: idioms
tags: [idiom, typescript]
languages: [typescript]
difficulty: advanced
---

## Intent

Create nominally distinct types from structural equivalents by intersecting with a unique symbol brand, preventing accidental interchange of values that share the same underlying type (e.g., `UserId` vs `OrderId`, both `string`).

## Problem

TypeScript uses structural typing: two types with the same shape are interchangeable. A `UserId` and an `OrderId` that are both `string` can be passed to each other's functions without error. The type system cannot catch semantic misuse of structurally identical values.

## Solution

Intersect the base type with a unique symbol property that exists only in the type system (never at runtime). Provide smart constructors that validate and return the branded type. The brand prevents assignment between structurally identical but semantically different types.

## Language Implementations

### TypeScript

```typescript
// Brand utility type
declare const __brand: unique symbol;
type Brand<T, B extends string> = T & { readonly [__brand]: B };

// Branded types -- zero runtime cost
type UserId = Brand<string, "UserId">;
type OrderId = Brand<string, "OrderId">;
type Meters = Brand<number, "Meters">;
type Seconds = Brand<number, "Seconds">;

// Smart constructors with validation
function userId(id: string): UserId {
  if (!id.match(/^usr_[a-z0-9]{12}$/)) {
    throw new Error(`Invalid user ID: ${id}`);
  }
  return id as UserId;
}

function meters(value: number): Meters {
  if (value < 0) throw new Error("Meters cannot be negative");
  return value as Meters;
}

// Type safety -- compile errors:
function fetchUser(id: UserId): Promise<User> { /* ... */ }
function fetchOrder(id: OrderId): Promise<Order> { /* ... */ }

const uid = userId("usr_abc123def456");
const oid = "ord_xyz" as OrderId;

fetchUser(uid);  // OK
// fetchUser(oid);  // Compile error: OrderId is not assignable to UserId
// fetchUser("raw-string");  // Compile error: string is not assignable to UserId
```

## When to Use

- When multiple values share a primitive type but have different domain semantics.
- When you want compile-time safety without runtime wrapper overhead.
- When smart constructors should enforce validation at creation boundaries.

## When NOT to Use

- When structural typing is sufficient and confusion is unlikely.
- When you need runtime type checking (brands exist only at compile time).
- When the overhead of smart constructors at every creation site is impractical.

## Anti-Patterns

- Using `as BrandedType` everywhere without going through the smart constructor.
- Creating brands for types that are never confused in practice.
- Adding runtime brand properties to objects (defeats the zero-cost purpose).

## Related Patterns

- [newtype](../rust/newtype.md) -- Rust's equivalent with actual distinct types at zero cost.
- [discriminated-unions](discriminated-unions.md) -- structural discrimination for variant types.
- [type-guards](type-guards.md) -- runtime narrowing that complements compile-time brands.

## References

- TypeScript GitHub -- Nominal typing discussion: https://github.com/microsoft/TypeScript/issues/202
- Effect-TS Brand module: https://effect.website/docs/other/data-types/brand
