#!/usr/bin/env bash
set -uo pipefail
[ "${ECC_WORKFLOW_BYPASS:-}" = "1" ] && exit 0

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"
WORKFLOW_DIR="$PROJECT_DIR/.claude/workflow"
STATE_FILE="$WORKFLOW_DIR/state.json"

# Usage: bash .claude/hooks/workflow-init.sh <concern> "<feature>"
CONCERN="${1:-}"
FEATURE="${2:-}"

if [ -z "$CONCERN" ] || [ -z "$FEATURE" ]; then
  echo "Usage: workflow-init.sh <concern> \"<feature description>\"" >&2
  echo "  concern: dev, refactor, security, docs" >&2
  exit 1
fi

# Create workflow directory
mkdir -p "$WORKFLOW_DIR"

# Clean previous artifacts
rm -f "$WORKFLOW_DIR/plan.md" \
      "$WORKFLOW_DIR/solution.md" \
      "$WORKFLOW_DIR/implement-done.md" \
      "$WORKFLOW_DIR/.tdd-state"

# Build state JSON
STARTED_AT=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Atomic write via mktemp+mv
TMPFILE=$(mktemp "${WORKFLOW_DIR}/state.XXXXXX") || exit 1

if command -v jq >/dev/null 2>&1; then
  jq -n \
    --arg concern "$CONCERN" \
    --arg phase "plan" \
    --arg feature "$FEATURE" \
    --arg started_at "$STARTED_AT" \
    '{
      concern: $concern,
      phase: $phase,
      feature: $feature,
      started_at: $started_at,
      artifacts: { plan: null, solution: null, implement: null },
      completed: []
    }' > "$TMPFILE"
else
  printf '{"concern":"%s","phase":"plan","feature":"%s","started_at":"%s","artifacts":{"plan":null,"solution":null,"implement":null},"completed":[]}\n' \
    "$CONCERN" "$FEATURE" "$STARTED_AT" > "$TMPFILE"
fi

mv "$TMPFILE" "$STATE_FILE"

echo "Workflow initialized: concern=$CONCERN, feature=\"$FEATURE\""
echo "Phase: plan -> solution -> implement -> done"
echo "State: $STATE_FILE"
