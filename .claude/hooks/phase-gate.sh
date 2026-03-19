#!/usr/bin/env bash
set -uo pipefail
[ "${ECC_WORKFLOW_BYPASS:-}" = "1" ] && exit 0

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"
STATE_FILE="$PROJECT_DIR/.claude/workflow/state.json"

# No workflow active — allow everything
[ ! -f "$STATE_FILE" ] && exit 0

PHASE=$(jq -r '.phase // "done"' "$STATE_FILE" 2>/dev/null) || exit 0

# implement and done phases — no gating
[ "$PHASE" = "done" ] || [ "$PHASE" = "implement" ] && exit 0

# Stale workflow warning (> 86400 seconds = 24 hours)
STARTED_AT=$(jq -r '.started_at // empty' "$STATE_FILE" 2>/dev/null) || true
if [ -n "$STARTED_AT" ]; then
  # Parse ISO 8601 timestamp to epoch
  STARTED_EPOCH=$(date -jf "%Y-%m-%dT%H:%M:%SZ" "$STARTED_AT" +%s 2>/dev/null || date -d "$STARTED_AT" +%s 2>/dev/null || echo "")
  if [ -n "$STARTED_EPOCH" ]; then
    NOW_EPOCH=$(date +%s)
    AGE=$((NOW_EPOCH - STARTED_EPOCH))
    if [ "$AGE" -gt 86400 ]; then
      HOURS=$((AGE / 3600))
      echo "WARNING: Workflow started ${HOURS} hours ago. Consider whether this workflow is still relevant." >&2
      echo "To reset: rm $STATE_FILE" >&2
    fi
  fi
fi

# Read tool input from stdin (Claude hook protocol)
INPUT=$(cat)
TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // empty' 2>/dev/null) || exit 0
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null) || true
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null) || true

# --- Write/Edit/MultiEdit: allow workflow and docs paths only ---
case "$TOOL_NAME" in
  Write|Edit|MultiEdit)
    case "$FILE_PATH" in
      */.claude/workflow/*) exit 0 ;;
      */docs/audits/*) exit 0 ;;
      */docs/backlog/*) exit 0 ;;
      */docs/user-stories/*) exit 0 ;;
      .claude/workflow/*) exit 0 ;;
      docs/audits/*) exit 0 ;;
      docs/backlog/*) exit 0 ;;
      docs/user-stories/*) exit 0 ;;
    esac
    echo "BLOCKED: Cannot write to '$FILE_PATH' during $PHASE phase." >&2
    echo "Only .claude/workflow/*, docs/audits/*, docs/backlog/*, docs/user-stories/* are allowed." >&2
    echo "Advance to implement phase by completing the $PHASE artifact." >&2
    exit 2
    ;;
esac

# --- Bash: block destructive commands ---
case "$TOOL_NAME" in
  Bash)
    if [ -n "$COMMAND" ]; then
      case "$COMMAND" in
        *"rm -rf"*|*"git reset --hard"*|*"git clean"*|*"git checkout --"*|*"cargo publish"*)
          echo "BLOCKED: Destructive command not allowed during $PHASE phase." >&2
          echo "Command: $COMMAND" >&2
          exit 2
          ;;
      esac
    fi
    exit 0
    ;;
esac

# All other tools (Read, Glob, Grep, etc.) — allow
exit 0
