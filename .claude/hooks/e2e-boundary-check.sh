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

# No implement-done file — nothing to check
[ ! -f "$IMPL_FILE" ] && exit 0

# If implement-done.md exists but has no E2E Tests section, warn
if ! grep -q '^## E2E Tests' "$IMPL_FILE"; then
  echo "WARNING: implement-done.md is missing '## E2E Tests' section." >&2
  echo "Add an '## E2E Tests' section documenting E2E test results (or 'No E2E tests required by solution')." >&2
  exit 0
fi

exit 0
