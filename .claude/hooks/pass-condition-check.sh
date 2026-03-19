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

IMPL_FILE="$PROJECT_DIR/.claude/workflow/implement-done.md"

if [ ! -f "$IMPL_FILE" ]; then
  echo "BLOCKED: implement-done.md not found." >&2
  exit 2
fi

# Check for ## Pass Condition Results heading
if ! grep -q '^## Pass Condition Results' "$IMPL_FILE"; then
  echo "BLOCKED: implement-done.md missing '## Pass Condition Results' section." >&2
  exit 2
fi

# Block on any failures
if grep -q '❌' "$IMPL_FILE"; then
  echo "BLOCKED: Pass condition failures found in implement-done.md:" >&2
  grep '❌' "$IMPL_FILE" >&2
  echo "" >&2
  echo "Fix all failing conditions before completing the workflow." >&2
  exit 2
fi

# Check for summary line
if ! grep -qE 'All pass conditions:.*✅' "$IMPL_FILE"; then
  echo "BLOCKED: Missing 'All pass conditions: ... ✅' summary line." >&2
  echo "Add a summary confirming all pass conditions passed." >&2
  exit 2
fi

exit 0
