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
ARTIFACT_PATH="${3:-}"

if [ -z "$TARGET" ]; then
  echo "Usage: phase-transition.sh <target-phase> [artifact-name]" >&2
  echo "  target-phase: solution, implement, done" >&2
  echo "  artifact-name: plan, solution, implement (optional — sets artifacts.<name> timestamp)" >&2
  exit 1
fi

if [ ! -f "$STATE_FILE" ]; then
  echo "ERROR: No workflow active — $STATE_FILE not found." >&2
  echo "Run a /spec-* command first to initialize the workflow." >&2
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
  # Spec/design aliases (backward compatible)
  plan_design)        VALID=true; TARGET="solution" ;;
  spec_solution)      VALID=true; TARGET="solution" ;;
  spec_design)        VALID=true; TARGET="solution" ;;
  design_implement)   VALID=true; TARGET="implement" ;;
  # Allow re-entry (idempotent)
  "${TARGET}_${TARGET}") VALID=true ;;
esac

if [ "$VALID" != "true" ]; then
  echo "ERROR: Illegal transition '$CURRENT' -> '$TARGET'." >&2
  echo "Valid transitions: spec->design, plan->solution, solution->implement, implement->done." >&2
  exit 1
fi

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Build jq expression
JQ_EXPR=".phase = \"$TARGET\""

if [ -n "$ARTIFACT" ]; then
  JQ_EXPR="$JQ_EXPR | .artifacts.$ARTIFACT = \"$TIMESTAMP\""
fi

# Store spec/design file paths in state.json (BL-029)
if [ -n "$ARTIFACT_PATH" ]; then
  case "$ARTIFACT" in
    plan)      JQ_EXPR="$JQ_EXPR | .artifacts.spec_path = \"$ARTIFACT_PATH\"" ;;
    solution)  JQ_EXPR="$JQ_EXPR | .artifacts.design_path = \"$ARTIFACT_PATH\"" ;;
    implement) JQ_EXPR="$JQ_EXPR | .artifacts.tasks_path = \"$ARTIFACT_PATH\"" ;;
  esac
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

# Write to cross-session memory (BL-027)
MEMORY_WRITER="$PROJECT_DIR/.claude/hooks/memory-writer.sh"
if [ -x "$MEMORY_WRITER" ] && [ -n "$ARTIFACT" ]; then
  FEATURE=$(jq -r '.feature // "unknown"' "$STATE_FILE" 2>/dev/null)
  CONCERN=$(jq -r '.concern // "unknown"' "$STATE_FILE" 2>/dev/null)
  # Map artifact name to work-item phase
  case "$ARTIFACT" in
    plan)           WI_PHASE="plan" ;;
    solution)       WI_PHASE="solution" ;;
    implement)      WI_PHASE="implementation" ;;
    *)              WI_PHASE="" ;;
  esac
  # Write action log entry
  "$MEMORY_WRITER" action "$ARTIFACT" "$FEATURE" "success" "[]" 2>/dev/null || true
  # Write work item file
  if [ -n "$WI_PHASE" ]; then
    "$MEMORY_WRITER" work-item "$WI_PHASE" "$FEATURE" "$CONCERN" 2>/dev/null || true
  fi
  # Write to daily memory file (BL-047)
  "$MEMORY_WRITER" daily "$ARTIFACT" "$FEATURE" "$CONCERN" 2>/dev/null || true
  "$MEMORY_WRITER" memory-index 2>/dev/null || true
fi
