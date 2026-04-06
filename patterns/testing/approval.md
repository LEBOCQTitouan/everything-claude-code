---
name: approval
category: testing
tags: [testing, approval, golden-master, characterisation]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Capture the full output of a system and require explicit human approval before accepting it as the new baseline, providing a safety net for complex outputs where hand-written assertions are impractical.

## Problem

For legacy code without tests, complex report generation, or multi-line structured output, writing detailed assertions is both tedious and fragile. You need a way to lock down current behaviour and detect any deviation, with a deliberate review step before accepting changes.

## Solution

Run the code and capture its complete output. Compare against a stored "approved" file. If they differ, the test fails and presents a diff for human review. The developer explicitly approves changes by copying the "received" file to "approved", confirming the new output is intentional.

## Language Implementations

### Rust

```rust
// Using insta with explicit review workflow
use insta::assert_snapshot;

#[test]
fn test_invoice_report() {
    let report = generate_invoice_report(&sample_order());
    assert_snapshot!("invoice_report", report);
}
// Review workflow: cargo insta review
// Manually approve or reject each change
```

### Go

```go
// Using go-approval-tests
func TestInvoiceReport(t *testing.T) {
    report := GenerateInvoiceReport(SampleOrder())
    approvals.VerifyString(t, report)
    // Approved file: TestInvoiceReport.approved.txt
    // Received file: TestInvoiceReport.received.txt
}
```

### Python

```python
from approvaltests import verify

def test_invoice_report():
    report = generate_invoice_report(sample_order())
    verify(report)
    # Approved: test_invoice_report.approved.txt
    # Received: test_invoice_report.received.txt
    # Review and approve via: mv received approved
```

### Typescript

```typescript
import { verify } from "approvals";

test("invoice report", () => {
  const report = generateInvoiceReport(sampleOrder());
  verify(report);
  // Approved file stored alongside test
  // Review diff and approve explicitly
});
```

## When to Use

- When adding tests to legacy code (characterisation tests).
- For complex text output (reports, emails, CLI output, generated code).
- When the output is too complex for point assertions but too important to ignore.

## When NOT to Use

- When specific property assertions are more appropriate and maintainable.
- When output contains highly volatile data that changes every run.
- For simple values where `assert_eq` is clearer.

## Anti-Patterns

- Auto-approving all changes without reviewing the diff.
- Approval files so large that reviewers skip them in pull requests.
- Not normalising volatile fields (dates, IDs) before approval.

## Related Patterns

- [testing/snapshot](snapshot.md) -- similar mechanism, typically auto-updated rather than explicitly approved.
- [testing/aaa](aaa.md) -- approval replaces the Assert phase with file comparison.
- [testing/given-when-then](given-when-then.md) -- approval captures the Then as a golden file.

## References

- Llewellyn Falco, "Approval Tests": https://approvaltests.com
- Emily Bache, "The Approval Testing Approach": https://www.youtube.com/watch?v=OJmg9aMIHMw
- **Rust**: `insta` with manual review (`cargo insta review`)
- **Go**: `approvals` (approvals-go)
- **Python**: `approvaltests`, `pytest-approvaltests`
- **Kotlin/Java**: `approval-tests-java`
- **TypeScript**: `approvals` (npm)
