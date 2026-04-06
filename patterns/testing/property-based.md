---
name: property-based
category: testing
tags: [testing, property-based, generative, fuzzing]
languages: [rust, go, python, typescript]
difficulty: advanced
---

## Intent

Verify that a property holds for all valid inputs by generating hundreds of random test cases automatically, finding edge cases that hand-written examples miss.

## Problem

Example-based tests only cover the specific inputs the developer thought of. Edge cases, boundary conditions, and unusual combinations slip through. Writing enough examples to cover the input space thoroughly is impractical.

## Solution

Define properties (invariants) that must hold for all valid inputs. Let the framework generate random inputs, run the function, and check the property. When a failure is found, the framework shrinks the input to a minimal reproducing case.

## Language Implementations

### Rust

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn encode_decode_roundtrip(input in "\\PC*") {
        let encoded = encode(&input);
        let decoded = decode(&encoded).unwrap();
        prop_assert_eq!(decoded, input);
    }

    #[test]
    fn sort_preserves_length(mut vec in prop::collection::vec(any::<i32>(), 0..100)) {
        let original_len = vec.len();
        vec.sort();
        prop_assert_eq!(vec.len(), original_len);
    }
}
```

### Go

```go
func TestEncodeDecodeRoundtrip(t *testing.T) {
    f := func(input string) bool {
        encoded := Encode(input)
        decoded, err := Decode(encoded)
        return err == nil && decoded == input
    }
    if err := quick.Check(f, nil); err != nil {
        t.Error(err)
    }
}
```

### Python

```python
from hypothesis import given
from hypothesis import strategies as st

@given(st.text())
def test_encode_decode_roundtrip(input_str: str):
    encoded = encode(input_str)
    decoded = decode(encoded)
    assert decoded == input_str

@given(st.lists(st.integers()))
def test_sort_preserves_length(xs: list[int]):
    assert len(sorted(xs)) == len(xs)
```

### Typescript

```typescript
import fc from "fast-check";

test("encode/decode roundtrip", () => {
  fc.assert(
    fc.property(fc.string(), (input) => {
      const encoded = encode(input);
      const decoded = decode(encoded);
      expect(decoded).toBe(input);
    }),
  );
});
```

## When to Use

- For serialisation/deserialisation roundtrip properties.
- For sorting, filtering, and transformation invariants.
- For parser and codec correctness.
- When you need to discover edge cases in complex input spaces.

## When NOT to Use

- When the property is trivial or tautological.
- When generating valid inputs requires impractically complex custom generators.
- For integration tests where setup cost per run is high.

## Anti-Patterns

- Writing properties that re-implement the function under test.
- Ignoring shrunk failure cases instead of fixing the root cause.
- Not seeding the random generator for reproducible CI failures.

## Related Patterns

- [testing/table-driven](table-driven.md) -- complements property tests with specific known examples.
- [testing/mutation](mutation.md) -- verifies that tests catch injected faults.
- [testing/aaa](aaa.md) -- property tests follow the same logical structure, with generated Arrange.

## References

- John Hughes, "QuickCheck: A Lightweight Tool for Random Testing of Haskell Programs".
- **Rust**: `proptest`, `quickcheck`, `arbitrary`
- **Go**: `testing/quick`, `rapid` (pgregory/rapid)
- **Python**: `hypothesis`
- **Kotlin**: Kotest property testing, `jqwik`
- **TypeScript**: `fast-check`
