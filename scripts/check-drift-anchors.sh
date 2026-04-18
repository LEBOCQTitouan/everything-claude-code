#!/usr/bin/env bash
# BL-132 classifier: flow/decision diagrams require a drift-anchor comment.
#
# Rule (AC-R3.1):
#   A diagram is flow/decision if its `text` fence body contains `--Y-->` or `--N-->`.
#   Every such diagram MUST carry `<!-- keep in sync with: <test_fn_name> -->` within
#   the 3 `///` lines immediately preceding the opening fence.
#
# Usage:
#   scripts/check-drift-anchors.sh [path]      # scan path (default: crates/)
#   scripts/check-drift-anchors.sh --fixtures  # fixture self-test
#
# Exit: 0 = no violations. 1 = violations.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FIXTURE_DIR="$ROOT/scripts/fixtures/bl132"

scan_file() {
  awk '
    BEGIN {
      in_fence = 0
      fence_start_line = 0
      fence_has_flow = 0
      anchor_ok = 0
    }
    # Track last 3 `///` lines as a ring buffer.
    /^\/\/\// {
      last3[2] = last3[1]
      last3[1] = last3[0]
      last3[0] = $0
    }
    # Fence opener on a /// line
    /^\/\/\/[[:space:]]*```text/ {
      if (!in_fence) {
        in_fence = 1
        fence_start_line = NR
        fence_has_flow = 0
        # Check anchor in last 3 /// lines before the opener
        anchor_ok = 0
        for (i = 0; i < 3; i++) {
          if (match(last3[i], /<!--[[:space:]]*keep in sync with:/)) {
            anchor_ok = 1
            break
          }
        }
        next  # do not treat opener itself as inside
      }
    }
    # Fence closer on a /// line (after we were in a text fence)
    in_fence && /^\/\/\/[[:space:]]*```[[:space:]]*$/ {
      if (fence_has_flow && !anchor_ok) {
        print FILENAME ":" fence_start_line ": DRIFT_ANCHOR_MISSING (flow diagram without keep-in-sync comment)"
        found++
      }
      in_fence = 0
      fence_has_flow = 0
      next
    }
    # Inside fence body: detect flow tokens
    in_fence && (/--Y-->/ || /--N-->/) {
      fence_has_flow = 1
    }
    END { exit (found > 0 ? 1 : 0) }
  ' "$1"
}

run_fixture_selftest() {
  local failed=0
  # drift_present_ok.rs → pass
  if ! scan_file "$FIXTURE_DIR/drift_present_ok.rs" >/dev/null; then
    echo "FIXTURE REGRESSION: drift_present_ok.rs must pass but FAILED" >&2
    failed=1
  fi
  # drift_missing_fail.rs → fail
  if scan_file "$FIXTURE_DIR/drift_missing_fail.rs" >/dev/null; then
    echo "FIXTURE REGRESSION: drift_missing_fail.rs must fail but PASSED" >&2
    failed=1
  fi
  return $failed
}

if [[ "${1:-}" == "--fixtures" ]]; then
  run_fixture_selftest
  exit $?
fi

if ! run_fixture_selftest; then
  echo "Fixture self-test FAILED — classifier is broken, aborting." >&2
  exit 1
fi

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
