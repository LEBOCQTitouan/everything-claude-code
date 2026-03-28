# Solution: Ordering Violation Fixture

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-003 | unit | Third test first | AC-001.1 | cargo test | PASS |
| PC-001 | unit | First test second | AC-001.2 | cargo test | PASS |
| PC-002 | unit | Second test third | AC-001.1 | cargo test | PASS |

## File Changes

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `src/lib.rs` | Create | Core logic | AC-001.1 |
| 2 | `src/main.rs` | Modify | Entry point | AC-001.2 |
