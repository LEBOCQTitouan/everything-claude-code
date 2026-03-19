#!/usr/bin/env bash
set -uo pipefail
[ "${ECC_WORKFLOW_BYPASS:-}" = "1" ] && exit 0

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"
STATE_FILE="$PROJECT_DIR/.claude/workflow/state.json"

# No workflow active
[ ! -f "$STATE_FILE" ] && exit 0

PHASE=$(jq -r '.phase // "done"' "$STATE_FILE" 2>/dev/null) || exit 0

# Read tool input from stdin
INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null) || exit 0

# Normalize: extract basename for matching
BASENAME=$(basename "$FILE_PATH" 2>/dev/null) || exit 0

# Detect artifact writes and determine next phase
NEXT_PHASE=""
ARTIFACT_KEY=""

case "$PHASE" in
  plan)
    [ "$BASENAME" = "plan.md" ] && NEXT_PHASE="solution" && ARTIFACT_KEY="plan"
    ;;
  solution)
    [ "$BASENAME" = "solution.md" ] && NEXT_PHASE="implement" && ARTIFACT_KEY="solution"
    ;;
  implement)
    [ "$BASENAME" = "implement-done.md" ] && NEXT_PHASE="done" && ARTIFACT_KEY="implement"
    ;;
esac

# No transition detected
[ -z "$NEXT_PHASE" ] && exit 0

# Atomic state update
TMPFILE=$(mktemp "${PROJECT_DIR}/.claude/workflow/state.XXXXXX") || exit 0

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

jq \
  --arg next_phase "$NEXT_PHASE" \
  --arg artifact_key "$ARTIFACT_KEY" \
  --arg timestamp "$TIMESTAMP" \
  --arg file_path "$FILE_PATH" \
  '.phase = $next_phase |
   .artifacts[$artifact_key] = $timestamp |
   .completed += [{ phase: $artifact_key, file: $file_path, at: $timestamp }]' \
  "$STATE_FILE" > "$TMPFILE" 2>/dev/null || { rm -f "$TMPFILE"; exit 0; }

mv "$TMPFILE" "$STATE_FILE"

echo "Phase advanced: $PHASE -> $NEXT_PHASE"
