#!/usr/bin/env bash
set -uo pipefail
[ "${ECC_WORKFLOW_BYPASS:-}" = "1" ] && exit 0

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"
STATE_FILE="$PROJECT_DIR/.claude/workflow/state.json"

# No workflow active — allow stop
[ ! -f "$STATE_FILE" ] && exit 0

PHASE=$(jq -r '.phase // "done"' "$STATE_FILE" 2>/dev/null) || exit 0

# Done phase — allow stop
[ "$PHASE" = "done" ] && exit 0

FEATURE=$(jq -r '.feature // "unknown"' "$STATE_FILE" 2>/dev/null) || true

echo "BLOCKED: Cannot stop — workflow is in '$PHASE' phase." >&2
echo "Feature: $FEATURE" >&2
echo "Remaining: $PHASE -> ... -> done" >&2
echo "" >&2
echo "Escape hatch: rm $STATE_FILE" >&2
exit 2
