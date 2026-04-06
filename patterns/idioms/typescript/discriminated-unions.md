---
name: discriminated-unions
category: idioms
tags: [idiom, typescript]
languages: [typescript]
difficulty: intermediate
---

## Intent

Model a value that can be one of several distinct shapes, using a shared literal discriminant field to enable exhaustive type narrowing in `switch` statements, catching missing cases at compile time.

## Problem

Union types like `Shape` that could be a circle, rectangle, or triangle lack a reliable way to narrow the type. Using `instanceof` requires classes. Type guards for each variant are verbose. Without exhaustiveness checking, adding a new variant silently skips handling.

## Solution

Add a literal `type` (or `kind`) field to each variant interface. TypeScript narrows the union automatically in `switch`/`if` blocks based on the discriminant. Use the `never` type in a default branch to get compile errors when a new variant is added but not handled.

## Language Implementations

### TypeScript

```typescript
interface Circle {
  readonly kind: "circle";
  readonly radius: number;
}

interface Rectangle {
  readonly kind: "rectangle";
  readonly width: number;
  readonly height: number;
}

interface Triangle {
  readonly kind: "triangle";
  readonly base: number;
  readonly height: number;
}

type Shape = Circle | Rectangle | Triangle;

// Exhaustive switch with never guard
function area(shape: Shape): number {
  switch (shape.kind) {
    case "circle":
      return Math.PI * shape.radius ** 2;
    case "rectangle":
      return shape.width * shape.height;
    case "triangle":
      return 0.5 * shape.base * shape.height;
    default: {
      // Compile error if a variant is missing above
      const _exhaustive: never = shape;
      return _exhaustive;
    }
  }
}

// Result type -- common for error handling
type Result<T, E = Error> =
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly error: E };

function divide(a: number, b: number): Result<number, string> {
  if (b === 0) return { ok: false, error: "division by zero" };
  return { ok: true, value: a / b };
}
```

## When to Use

- When modeling domain events, commands, or API responses with distinct shapes.
- When you need exhaustive handling that catches new variants at compile time.
- When replacing class hierarchies with plain data objects.

## When NOT to Use

- When variants share most fields (use a single interface with optional fields).
- When the set of variants is open-ended and defined by external consumers.
- When runtime type information (`instanceof`) is already available and sufficient.

## Anti-Patterns

- Omitting the `never` exhaustiveness check in the default branch.
- Using string enums as discriminants when literal types are clearer.
- Nesting discriminated unions deeply, making narrowing complex.

## Related Patterns

- [sealed-classes](../kotlin/sealed-classes.md) -- Kotlin's equivalent with `when` exhaustiveness.
- [enum-dispatch](../rust/enum-dispatch.md) -- Rust's equivalent using `match` on enum variants.
- [type-guards](type-guards.md) -- custom narrowing for cases beyond discriminated unions.

## References

- TypeScript Handbook -- Discriminated Unions: https://www.typescriptlang.org/docs/handbook/2/narrowing.html#discriminated-unions
- TypeScript Handbook -- Exhaustiveness checking: https://www.typescriptlang.org/docs/handbook/2/narrowing.html#exhaustiveness-checking
