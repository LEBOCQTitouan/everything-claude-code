---
name: behaviour-extraction
description: Extract runtime behaviour, side effects, error paths, and interaction protocols from source code for documentation purposes.
origin: ECC
---

# Behaviour Extraction

Atomic extraction skill for understanding what code *does* at runtime — side effects, error conditions, state transitions, and interaction protocols. Complements symbol-extraction (which captures *what* is exported) by capturing *how* things behave.

## When to Activate

- After symbol-extraction, to enrich API documentation with behavioural context
- When generating runbooks that need failure mode understanding
- When documenting complex workflows or multi-step protocols
- Before writing integration test documentation

## Methodology

### 1. Side Effect Analysis

For each public function/method, identify:

| Side Effect Type | Detection Pattern |
|-----------------|-------------------|
| File I/O | `fs.*`, `open()`, `os.*`, file path arguments |
| Network | `fetch`, `http.*`, `net.*`, URL arguments |
| Database | SQL strings, ORM calls, connection objects |
| Environment | `process.env`, `os.environ`, `os.Getenv` |
| Console/Logging | `console.*`, `log.*`, `println`, `fmt.Print*` |
| Process | `exec`, `spawn`, `subprocess`, `os/exec` |
| State mutation | `this.*=`, `self.*=`, global variable writes |

Record: `function → [side_effect_type, target, is_conditional]`

### 2. Error Path Extraction

For each public function, trace error conditions:

1. **Explicit throws/returns**: `throw`, `raise`, `return error`, `Err()`
2. **Error propagation**: `?` operator (Rust), uncaught promise rejections, re-throws
3. **Validation guards**: Early returns on invalid input
4. **Panic paths**: `panic!()`, `process.exit()`, `os.Exit()`, `assert`

Record per error path:
- Trigger condition (what input/state causes it)
- Error type/class
- Whether it's recoverable
- Whether it's documented in the existing doc comment

### 3. Protocol Detection

Identify multi-step interaction patterns:

- **Lifecycle protocols**: `init → configure → start → stop → cleanup`
- **Request/response**: `validate → process → respond`
- **Builder patterns**: chained method calls before a terminal `.build()`
- **State machines**: enum-driven branching, status field transitions
- **Retry/backoff**: loop with delay patterns around fallible operations

Record: `protocol_name → [step1, step2, ...] → terminal_condition`

### 4. Concurrency Behaviour

Detect concurrent execution patterns:

- **Thread safety**: mutex/lock usage, atomic operations, channel usage
- **Async patterns**: `async/await`, `Promise.all`, goroutines, `tokio::spawn`
- **Race conditions**: shared mutable state without synchronization
- **Cancellation**: context cancellation, abort signals, timeout handling

### 5. Invariant Detection

Identify conditions that must hold:

- **Preconditions**: validation checks at function entry
- **Postconditions**: assertions/checks before return
- **Loop invariants**: conditions maintained across iterations
- **Type narrowing**: runtime type checks that refine behaviour

## Output Format

Per-function behavioural summary:

```
function mergeDirectory(src, dest, opts?)
  Side effects:
    - File I/O: reads src directory, writes to dest directory
    - Console: logs progress when opts.verbose is true
  Error paths:
    - throws ENOENT if src does not exist
    - throws EACCES if dest is not writable
    - returns partial result if individual file copy fails (non-fatal)
  Protocol: validate-inputs → scan-source → resolve-conflicts → copy-files → verify
  Concurrency: not thread-safe (concurrent calls with same dest may conflict)
  Invariants:
    - src must be an existing directory
    - dest is created if it does not exist
```

## Depth Control

To avoid excessive analysis time on large codebases:

- **Shallow** (default): Side effects + error paths only, no protocol/concurrency analysis
- **Deep**: All five analysis steps, limited to modules flagged by doc-analyzer as high-priority
- **Targeted**: Full analysis on a single function/module specified by `--target`

## Related

- Symbol extraction: `skills/symbol-extraction/SKILL.md`
- Failure modes: `skills/failure-modes/SKILL.md`
- API reference generation: `skills/api-reference-gen/SKILL.md`
- Runbook generation: `skills/runbook-gen/SKILL.md`
