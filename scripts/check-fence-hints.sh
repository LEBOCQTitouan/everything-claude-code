#!/usr/bin/env bash
# BL-132 classifier: enforce language-tag on every opening fence inside `///` doc-comments.
#
# Rule (AC-R1.2):
#   Every `///` fenced code block opener MUST specify one of:
#     text | rust | ignore | no_run | compile_fail
#   Bare ``` openers trigger rustdoc to compile content as Rust → doctest failures on ASCII art.
#   Closing fences are ignored by design (only the opener carries the language tag).
#
# Usage:
#   scripts/check-fence-hints.sh [path]       # scan path (default: crates/)
#   scripts/check-fence-hints.sh --fixtures   # fixture self-test (exit 1 if any fixture wrong)
#
# Exit: 0 = no violations. 1 = violations (or fixture self-test failure).

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FIXTURE_DIR="$ROOT/scripts/fixtures/bl132"

scan_file() {
  # Print "<file>:<line>: BARE_FENCE_OPENER" for each violation. Exit 0 if none.
  awk '
    # Match a /// line whose content after /// is exactly a ``` fence (possibly with lang tag).
    /^\/\/\/[[:space:]]*```/ {
      fence_count[FILENAME]++
      if (fence_count[FILENAME] % 2 == 1) {
        # Opener. Require a known language tag immediately after ```.
        if (!match($0, /^\/\/\/[[:space:]]*```(text|rust|ignore|no_run|compile_fail)[[:space:]]*$/)) {
          print FILENAME ":" NR ": BARE_FENCE_OPENER"
          found++
        }
      }
    }
    END { exit (found > 0 ? 1 : 0) }
  ' "$1"
}

run_fixture_selftest() {
  # Expectations:
  #   fence_text_ok.rs     → pass (0)
  #   fence_bare_fail.rs   → fail (1)
  #   fence_closing_ok.rs  → pass (0)
  local failed=0
  for name in fence_text_ok fence_closing_ok; do
    if ! scan_file "$FIXTURE_DIR/$name.rs" >/dev/null; then
      echo "FIXTURE REGRESSION: $name.rs must pass but FAILED" >&2
      failed=1
    fi
  done
  if scan_file "$FIXTURE_DIR/fence_bare_fail.rs" >/dev/null; then
    echo "FIXTURE REGRESSION: fence_bare_fail.rs must fail but PASSED" >&2
    failed=1
  fi
  return $failed
}

if [[ "${1:-}" == "--fixtures" ]]; then
  run_fixture_selftest
  exit $?
fi

# Always run fixture self-test first
if ! run_fixture_selftest; then
  echo "Fixture self-test FAILED — classifier is broken, aborting." >&2
  exit 1
fi

# Real scan: default crates/ unless overridden
TARGET="${1:-$ROOT/crates}"
if [[ ! -e "$TARGET" ]]; then
  echo "Target path not found: $TARGET" >&2
  exit 1
fi

violations=0
while IFS= read -r -d '' f; do
  scan_file "$f" || violations=1
done < <(find "$TARGET" -name '*.rs' -not -path '*/target/*' -print0)

exit $violations
