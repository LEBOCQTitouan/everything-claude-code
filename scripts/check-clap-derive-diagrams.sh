#!/usr/bin/env bash
# BL-132 classifier: ban inline `text` diagrams on `///` attached to clap-derive items.
#
# Rule (AC-R1.1):
#   In files containing #[derive(Parser|Subcommand|Args|ValueEnum)], a `///.*```text` line
#   MUST live inside either:
#     - a `//!` module-header doc block, OR
#     - an `impl` block scope
#   Diagrams on `///` directly above a struct/variant/field in a clap-derive file would be
#   promoted into clap's `--help --long` output and corrupt user-visible CLI.
#
# Usage:
#   scripts/check-clap-derive-diagrams.sh [path]      # scan path (default: crates/ecc-cli crates/ecc-workflow)
#   scripts/check-clap-derive-diagrams.sh --fixtures  # fixture self-test
#
# Exit: 0 = no violations. 1 = violations.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FIXTURE_DIR="$ROOT/scripts/fixtures/bl132"

# Scan a single file. Assumes caller has already verified the file contains a clap derive.
scan_file() {
  awk '
    BEGIN {
      impl_depth = 0
      in_impl = 0
      brace_depth = 0
      # We count braces OUTSIDE impl to detect when we leave the impl block.
    }
    # Track impl entry: look for "impl" at start of a non-doc line, optionally with generics.
    /^impl([[:space:]<]|$)/ {
      if (!in_impl) {
        in_impl = 1
        impl_depth = brace_depth  # record the brace depth at which we entered impl
      }
    }
    # Count braces on every non-comment line.
    {
      # Strip // line comments before counting braces.
      line = $0
      sub(/\/\/.*/, "", line)
      n_open = gsub(/\{/, "{", line)
      n_close = gsub(/\}/, "}", line)
      brace_depth += n_open - n_close
      if (in_impl && brace_depth <= impl_depth) {
        in_impl = 0
      }
    }
    # A `///.*```text` line is a violation unless inside //! block or impl scope.
    /^\/\/\/[[:space:]]*```text/ {
      if (!in_impl) {
        print FILENAME ":" NR ": CLAP_DERIVE_DIAGRAM"
        found++
      }
    }
    END { exit (found > 0 ? 1 : 0) }
  ' "$1"
}

has_clap_derive() {
  grep -qE '#\[derive\((Parser|Subcommand|Args|ValueEnum)\)' "$1" 2>/dev/null
}

run_fixture_selftest() {
  local failed=0
  # impl-block diagram → pass
  if ! scan_file "$FIXTURE_DIR/clap_impl_ok.rs" >/dev/null; then
    echo "FIXTURE REGRESSION: clap_impl_ok.rs must pass but FAILED" >&2
    failed=1
  fi
  # module-level diagram → pass (the diagram is in //!, not ///, so the rule doesn't fire)
  if ! scan_file "$FIXTURE_DIR/clap_module_ok.rs" >/dev/null; then
    echo "FIXTURE REGRESSION: clap_module_ok.rs must pass but FAILED" >&2
    failed=1
  fi
  # struct-level diagram in clap-derive file → fail
  if scan_file "$FIXTURE_DIR/clap_struct_fail.rs" >/dev/null; then
    echo "FIXTURE REGRESSION: clap_struct_fail.rs must fail but PASSED" >&2
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

TARGETS=()
if [[ $# -eq 0 ]]; then
  [[ -d "$ROOT/crates/ecc-cli" ]] && TARGETS+=("$ROOT/crates/ecc-cli")
  [[ -d "$ROOT/crates/ecc-workflow" ]] && TARGETS+=("$ROOT/crates/ecc-workflow")
else
  TARGETS=("$@")
fi

if [[ ${#TARGETS[@]} -eq 0 ]]; then
  echo "No targets found" >&2
  exit 1
fi

violations=0
for t in "${TARGETS[@]}"; do
  while IFS= read -r -d '' f; do
    if has_clap_derive "$f"; then
      scan_file "$f" || violations=1
    fi
  done < <(find "$t" -name '*.rs' -not -path '*/target/*' -print0)
done

exit $violations
