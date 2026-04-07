# Spec Adversary Report

## Summary
Verdict: PASS (avg: 82/100)
Rounds: 2 of 3

## Dimension Results
| # | Dimension | Score | Verdict | Critical Findings |
|---|-----------|-------|---------|-------------------|
| 1 | Ambiguity | 80 | PASS | "valid directory" now covered by AC-001.7; atomic write mechanism still underspecified but non-blocking |
| 2 | Edge Cases | 78 | PASS | Corrupt, stale, write ordering, dir creation all addressed; benign TOCTOU acknowledged implicitly |
| 3 | Scope Creep Risk | 88 | PASS | No change from round 1 -- tight non-requirements, US-002 justified |
| 4 | Dependency Gaps | 90 | PASS | No change from round 1 -- clean |
| 5 | Testability | 72 | PASS | AC-001.5 remains a regression gate (filler), but all new ACs are deterministically testable |
| 6 | Decision Completeness | 82 | PASS | Three missing decisions now present; all rationales are substantive |
| 7 | Rollback & Failure | 80 | PASS | Write ordering explicit, reset best-effort explicit, partial-failure sequences documented |

## Detailed Findings

### 1. Ambiguity

- **Finding**: The atomic write mechanism for `.state-dir` is still underspecified. The constraint says "mktemp + mv" but AC-001.8 says "If `.state-dir` write fails, init still succeeds." An implementer could use a simple `std::fs::write` (not atomic) since the file is best-effort. The constraint and the AC contradict on strictness.
- **Evidence**: Constraint: "`.state-dir` must be written atomically (mktemp + mv)". AC-001.8: "If `.state-dir` write fails, init still succeeds (best-effort optimization, not correctness requirement)."
- **Recommendation**: Minor. Clarify: atomic write is preferred to avoid partial reads, but since the file is best-effort, a simple `write` + `rename` in the same directory suffices. This will not cause divergent implementations -- both approaches produce correct behavior given the fallback in AC-001.7. No spec change required.

- **Finding**: AC-001.2 still says "points to a valid directory" without explicit definition, but AC-001.7 now covers the gap by specifying what happens when the content "does not resolve to an existing directory." Combined, these two ACs establish that "valid" means "existing directory on the filesystem." Closed.
- **Evidence**: AC-001.7: "does not resolve to an existing directory, `resolve_state_dir()` falls back..."
- **Recommendation**: None.

### 2. Edge Cases

- **Finding**: The TOCTOU race between init writing `.state-dir` and another process calling `resolve_state_dir()` is not explicitly documented as benign. However, AC-001.7 (corrupt/stale fallback) and AC-001.8 (write ordering with state.json first) together make this race harmless: if `.state-dir` is missed or half-written, git resolution kicks in and finds the already-written `state.json`. The spec implicitly covers this.
- **Evidence**: AC-001.8 establishes write ordering (state.json first). AC-001.7 establishes fallback for any non-resolving content. Together they make the race benign.
- **Recommendation**: None. Documenting TOCTOU benignity would be nice but is not load-bearing.

- **Finding**: AC-001.8 includes `create_dir_all` for `.claude/workflow/`. This closes the fresh-worktree gap from round 1.
- **Evidence**: AC-001.8: "Init must `create_dir_all` for `.claude/workflow/` if needed."
- **Recommendation**: None.

- **Finding**: Symlink edge case: if `.state-dir` content points to a symlinked directory, `is_dir()` follows symlinks on all platforms. The spec does not mention this but the behavior is correct by default in Rust's `std::path::Path::is_dir()`. Not a gap.
- **Evidence**: Rust stdlib: `Path::is_dir()` follows symlinks.
- **Recommendation**: None.

### 3. Scope Creep Risk

- **Finding**: No change from round 1. US-002 is appropriately bundled. Non-requirements fence out env var and hook config approaches. The four new ACs (001.6-001.9) all directly address the `.state-dir` mechanism and do not expand scope.
- **Evidence**: All new ACs reference `.state-dir` lifecycle operations within the already-scoped US-001.
- **Recommendation**: None.

### 4. Dependency Gaps

- **Finding**: No change from round 1. The new ACs do not introduce new module dependencies. `create_dir_all` is already available on the `FileSystem` trait (used by `migrate_if_needed` at state_resolver.rs:119). `is_dir` is also on the trait (ecc-ports/src/fs.rs:17) and implemented by `InMemoryFileSystem` (in_memory_fs.rs:104).
- **Evidence**: Verified in codebase: `fn is_dir(&self, path: &Path) -> bool` at ecc-ports/src/fs.rs:17, InMemoryFileSystem at in_memory_fs.rs:104.
- **Recommendation**: None.

### 5. Testability

- **Finding**: AC-001.5 ("All existing state_resolver + init + reset tests still pass") remains a regression gate rather than a behavior spec. It adds no information that CI does not already enforce. Still filler, but not blocking.
- **Evidence**: AC-001.5 is the only AC that does not describe a new behavior.
- **Recommendation**: Low priority. Could be removed or replaced with a specific backward-compat test case, but this is cosmetic.

- **Finding**: All four new ACs (001.6-001.9) are deterministically testable with `InMemoryFileSystem`:
  - AC-001.6: Write `.state-dir` with known content, read back, verify trim behavior.
  - AC-001.7: Write `.state-dir` pointing to non-existent path, verify fallback to git resolution and warning.
  - AC-001.8: Verify init creates `state.json` before `.state-dir`; verify init succeeds even if `.state-dir` write is mocked to fail.
  - AC-001.9: Verify reset succeeds even if `.state-dir` deletion is mocked to fail.
- **Evidence**: `InMemoryFileSystem` supports `read_to_string`, `write`, `exists`, `is_dir`, `create_dir_all`, `remove_file` -- all operations needed.
- **Recommendation**: None.

- **Finding**: AC-002.1 (`git ls-files` returns empty) is still a one-time repo operation, not a recurring test. This was noted in round 1 and is inherent to the AC's nature. Not blocking.
- **Evidence**: AC-002.1 describes a git index state, not a code behavior.
- **Recommendation**: None.

### 6. Decision Completeness

- **Finding**: Decisions #5, #6, #7 close the three gaps identified in round 1:
  - #5 (anchor format): "single-line UTF-8 absolute path, trimmed on read" -- specific and implementable.
  - #6 (corrupt handling): "treated as absent -- fall back to git resolution" with rationale "fail-open to avoid blocking developer work" -- substantive.
  - #7 (write ordering): "state.json first, .state-dir second" with rationale explaining both failure modes -- substantive.
- **Evidence**: Decision table rows 5-7 in the spec.
- **Recommendation**: None.

- **Finding**: Round 1 noted the question of whether `transition` can create state without `init`. Decision #2 says "Written by `ecc-workflow init` only." Checking the codebase: `transition.rs` reads existing state but never creates it -- if `state.json` does not exist, transition fails with "No active workflow." The decision is consistent with the code.
- **Evidence**: Decision #2 + codebase behavior confirmed.
- **Recommendation**: None.

### 7. Rollback & Failure

- **Finding**: AC-001.8 explicitly documents write ordering (state.json first, .state-dir second) and specifies that `.state-dir` write failure does not block init. This closes the round 1 gap about partial-failure sequences.
- **Evidence**: AC-001.8: "If `.state-dir` write fails, init still succeeds (best-effort optimization, not correctness requirement)."
- **Recommendation**: None.

- **Finding**: AC-001.9 explicitly specifies best-effort deletion during reset with warning on failure. This closes the round 1 gap about reset deletion failure.
- **Evidence**: AC-001.9: "Deletion failure emits a warning but does not block the reset."
- **Recommendation**: None.

- **Finding**: Rollback path remains trivially safe: deploying the previous binary version ignores `.state-dir` entirely. The file is inert to older versions.
- **Evidence**: `.state-dir` is a new file read by new code only. Constraint: "No changes to the public API of `resolve_state_dir()`."
- **Recommendation**: None.

## Suggested ACs

None. All gaps from round 1 have been addressed.

## Verdict Rationale

Round 1 was CONDITIONAL (69/100) due to three dimensions below 70: Ambiguity (60), Edge Cases (55), Decision Completeness (55). The spec author added exactly the four ACs and three decisions recommended in round 1.

Round 2 evaluation:

- **Ambiguity (80)**: "Valid directory" is now operationally defined by the combination of AC-001.2 and AC-001.7. Content format is explicit in AC-001.6 and Decision #5. The only remaining ambiguity is the atomic-write mechanism (constraint vs. best-effort AC), which is minor because both implementations are correct given the fallback.

- **Edge Cases (78)**: Corrupt content (AC-001.7), stale anchor (AC-001.7), write ordering (AC-001.8), directory creation (AC-001.8), and reset failure (AC-001.9) are all covered. The TOCTOU race is implicitly benign. No critical edge cases remain unaddressed.

- **Decision Completeness (82)**: All three missing decisions are present with substantive rationales. No obvious decisions remain unlisted.

- **Rollback & Failure (80)**: Write ordering documented, partial failure modes analyzed, reset tolerance specified.

All dimensions are above 70. No dimension has a critical finding. The spec is ready for `/design`.
