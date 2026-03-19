#!/usr/bin/env bash
set -uo pipefail
[ "${ECC_WORKFLOW_BYPASS:-}" = "1" ] && exit 0

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"
STATE_FILE="$PROJECT_DIR/.claude/workflow/state.json"

# No workflow active
[ ! -f "$STATE_FILE" ] && exit 0

PHASE=$(jq -r '.phase // "plan"' "$STATE_FILE" 2>/dev/null) || exit 0

# Only enforce at done phase
[ "$PHASE" != "done" ] && exit 0

SOLUTION_FILE="$PROJECT_DIR/.claude/workflow/solution.md"
IMPL_FILE="$PROJECT_DIR/.claude/workflow/implement-done.md"

# No solution file — nothing to check
[ ! -f "$SOLUTION_FILE" ] && exit 0

# If solution has E2E Test Plan, implementation must have E2E Tests
if grep -q '^## E2E Test Plan' "$SOLUTION_FILE"; then
  if [ ! -f "$IMPL_FILE" ]; then
    echo "BLOCKED: solution.md has '## E2E Test Plan' but implement-done.md not found." >&2
    exit 2
  fi

  if ! grep -q '^## E2E Tests' "$IMPL_FILE"; then
    echo "BLOCKED: solution.md has '## E2E Test Plan' but implement-done.md missing '## E2E Tests' section." >&2
    echo "Add an '## E2E Tests' section documenting E2E test results." >&2
    exit 2
  fi
fi

exit 0
