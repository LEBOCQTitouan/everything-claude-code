---
name: snapshot
category: testing
tags: [testing, snapshot, regression, golden-file]
languages: [rust, go, python, typescript]
difficulty: beginner
---

## Intent

Capture the output of a function or component and compare it against a stored reference (snapshot) to detect unintended changes in behaviour or rendering.

## Problem

Manually writing expected output for complex structures (HTML, JSON, error messages, CLI output) is tedious and error-prone. When the output changes intentionally, updating dozens of hand-written assertions is costly.

## Solution

On the first run, the test framework serialises the actual output and saves it as a snapshot file. On subsequent runs, it compares the current output against the stored snapshot. If they differ, the test fails. If the change is intentional, the developer updates the snapshot.

## Language Implementations

### Rust

```rust
use insta::assert_snapshot;

#[test]
fn test_error_message_format() {
    let err = validate_config("{}");
    assert_snapshot!(err.to_string());
    // Snapshot stored in snapshots/test_error_message_format.snap
}

#[test]
fn test_json_serialization() {
    let order = Order::new("widget", 42);
    insta::assert_json_snapshot!(order);
}
// Review: cargo insta review
```

### Go

```go
func TestErrorMessageFormat(t *testing.T) {
    err := ValidateConfig("{}")
    // Using cupaloy for snapshot testing
    cupaloy.SnapshotT(t, err.Error())
    // Snapshot stored in .snapshots/TestErrorMessageFormat
}
// Update: UPDATE_SNAPSHOTS=true go test ./...
```

### Python

```python
def test_error_message_format(snapshot):
    err = validate_config("{}")
    assert str(err) == snapshot
    # Using syrupy or inline-snapshot

# Or with syrupy:
def test_json_output(snapshot):
    result = generate_report()
    assert result == snapshot
# Update: pytest --snapshot-update
```

### Typescript

```typescript
test("error message format", () => {
  const err = validateConfig("{}");
  expect(err.message).toMatchSnapshot();
  // Snapshot stored in __snapshots__/
});

test("component rendering", () => {
  const tree = render(<UserCard name="Alice" />);
  expect(tree).toMatchSnapshot();
});
// Update: vitest --update or jest --updateSnapshot
```

## When to Use

- For serialisation output (JSON, YAML, TOML).
- For error message formats and CLI output.
- For component rendering (React, HTML templates).
- When expected output is complex and hand-writing assertions is impractical.

## When NOT to Use

- When output contains non-deterministic values (timestamps, random IDs) -- normalise first.
- For business logic validation where specific assertions are more expressive.
- When snapshots become so large they are never meaningfully reviewed.

## Anti-Patterns

- Blindly updating snapshots without reviewing the diff.
- Snapshot files so large that reviewers skip them in PRs.
- Snapshotting unstable output without normalising volatile fields.

## Related Patterns

- [testing/approval](approval.md) -- similar concept with explicit human approval workflow.
- [testing/aaa](aaa.md) -- snapshot replaces the Assert phase with file comparison.
- [testing/contract](contract.md) -- snapshots as lightweight API shape contracts.

## References

- **Rust**: `insta` (mitsuhiko/insta) -- the gold standard for Rust snapshots
- **Go**: `cupaloy`, `go-snaps`
- **Python**: `syrupy`, `inline-snapshot`, `pytest-snapshot`
- **Kotlin**: Kotest `shouldMatchSnapshot`
- **TypeScript**: `vitest` toMatchSnapshot, `jest` snapshot testing
