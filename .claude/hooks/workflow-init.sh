#!/usr/bin/env bash
set -uo pipefail
[ "${ECC_WORKFLOW_BYPASS:-}" = "1" ] && exit 0

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"
WORKFLOW_DIR="$PROJECT_DIR/.claude/workflow"
STATE_FILE="$WORKFLOW_DIR/state.json"

# Usage: bash .claude/hooks/workflow-init.sh <concern> ["<feature>"]
# Feature is optional — defaults to "(pending)" when omitted (defined during grill-me interview).
CONCERN="${1:-}"
FEATURE="${2:-(pending)}"

if [ -z "$CONCERN" ]; then
  echo "Usage: workflow-init.sh <concern> [\"<feature description>\"]" >&2
  echo "  concern: dev, refactor, security, docs" >&2
  echo "  feature: optional — defaults to \"(pending)\"" >&2
  exit 1
fi

# Create workflow directory
mkdir -p "$WORKFLOW_DIR"

# Warn and archive stale workflow state (> 7 days old)
if [ -f "$STATE_FILE" ]; then
  STALE_PHASE=$(jq -r '.phase // "unknown"' "$STATE_FILE" 2>/dev/null)
  if [ "$STALE_PHASE" != "done" ]; then
    echo "WARNING: Previous workflow stuck at phase '$STALE_PHASE' — archiving stale state."
  fi
  ARCHIVE_DIR="$WORKFLOW_DIR/archive"
  mkdir -p "$ARCHIVE_DIR"
  ARCHIVE_TS=$(date -u +"%Y%m%d-%H%M%S")
  mv "$STATE_FILE" "$ARCHIVE_DIR/state-${ARCHIVE_TS}.json" 2>/dev/null || true
fi

# Clean previous artifacts
rm -f "$WORKFLOW_DIR/implement-done.md" \
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
echo "Phase: spec -> design -> implement -> done"
echo "State: $STATE_FILE"
