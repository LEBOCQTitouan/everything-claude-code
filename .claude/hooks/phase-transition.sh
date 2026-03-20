#!/usr/bin/env bash
set -uo pipefail
[ "${ECC_WORKFLOW_BYPASS:-}" = "1" ] && exit 0

# Usage: bash .claude/hooks/phase-transition.sh <target-phase> [artifact-name]
# Examples:
#   bash .claude/hooks/phase-transition.sh solution plan
#   bash .claude/hooks/phase-transition.sh implement solution
#   bash .claude/hooks/phase-transition.sh done implement

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"
WORKFLOW_DIR="$PROJECT_DIR/.claude/workflow"
STATE_FILE="$WORKFLOW_DIR/state.json"

TARGET="${1:-}"
ARTIFACT="${2:-}"

if [ -z "$TARGET" ]; then
  echo "Usage: phase-transition.sh <target-phase> [artifact-name]" >&2
  echo "  target-phase: solution, implement, done" >&2
  echo "  artifact-name: plan, solution, implement (optional — sets artifacts.<name> timestamp)" >&2
  exit 1
fi

if [ ! -f "$STATE_FILE" ]; then
  echo "ERROR: No workflow active — $STATE_FILE not found." >&2
  echo "Run a /plan-* command first to initialize the workflow." >&2
  exit 1
fi

# Read current phase
CURRENT=$(jq -r '.phase // "unknown"' "$STATE_FILE" 2>/dev/null)
if [ "$CURRENT" = "unknown" ]; then
  echo "ERROR: Cannot read current phase from $STATE_FILE." >&2
  exit 1
fi

# Validate transition is legal
VALID=false
TRANSITION="${CURRENT}_${TARGET}"
case "$TRANSITION" in
  plan_solution)      VALID=true ;;
  solution_implement) VALID=true ;;
  implement_done)     VALID=true ;;
  # Allow re-entry (idempotent)
  "${TARGET}_${TARGET}") VALID=true ;;
esac

if [ "$VALID" != "true" ]; then
  echo "ERROR: Illegal transition '$CURRENT' -> '$TARGET'." >&2
  echo "Valid transitions: plan->solution, solution->implement, implement->done." >&2
  exit 1
fi

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Build jq expression
JQ_EXPR=".phase = \"$TARGET\""

if [ -n "$ARTIFACT" ]; then
  JQ_EXPR="$JQ_EXPR | .artifacts.$ARTIFACT = \"$TIMESTAMP\""
fi

if [ "$TARGET" = "done" ]; then
  JQ_EXPR="$JQ_EXPR | .completed += [{\"phase\": \"$ARTIFACT\", \"file\": \"implement-done.md\", \"at\": \"$TIMESTAMP\"}]"
fi

# Atomic write via mktemp+mv
TMPFILE=$(mktemp "${WORKFLOW_DIR}/state.XXXXXX") || exit 1
jq "$JQ_EXPR" "$STATE_FILE" > "$TMPFILE" || { rm -f "$TMPFILE"; exit 1; }
mv "$TMPFILE" "$STATE_FILE"

echo "Phase transition: $CURRENT -> $TARGET"
[ -n "$ARTIFACT" ] && echo "Artifact stamped: $ARTIFACT = $TIMESTAMP"
[ "$TARGET" = "done" ] && echo "Workflow complete."
