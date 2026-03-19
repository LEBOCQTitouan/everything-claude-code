#!/usr/bin/env bash
set -uo pipefail
[ "${ECC_WORKFLOW_BYPASS:-}" = "1" ] && exit 0

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"
STATE_FILE="$PROJECT_DIR/.claude/workflow/state.json"

# No workflow active
[ ! -f "$STATE_FILE" ] && exit 0

PHASE=$(jq -r '.phase // "plan"' "$STATE_FILE" 2>/dev/null) || exit 0

# Only track during implement phase
[ "$PHASE" != "implement" ] && exit 0

TDD_STATE_FILE="$PROJECT_DIR/.claude/workflow/.tdd-state"

# Read tool input from stdin
INPUT=$(cat)
TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // empty' 2>/dev/null) || exit 0
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null) || true
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null) || true

# Detect test file writes
is_test_file() {
  local path="$1"
  case "$path" in
    *_test.go|*_test.rs|*test_*.py|*_test.py|*.test.ts|*.test.js|*.spec.ts|*.spec.js) return 0 ;;
    */tests/*|*/test/*|*/__tests__/*) return 0 ;;
  esac
  return 1
}

# Detect source file writes (non-test, non-config)
is_source_file() {
  local path="$1"
  is_test_file "$path" && return 1
  case "$path" in
    *.rs|*.go|*.py|*.ts|*.js|*.java|*.kt|*.cs|*.cpp|*.c|*.swift) return 0 ;;
  esac
  return 1
}

case "$TOOL_NAME" in
  Write|Edit)
    if [ -n "$FILE_PATH" ]; then
      if is_test_file "$FILE_PATH"; then
        echo "red" > "$TDD_STATE_FILE"
        echo "TDD: test written (RED) — $FILE_PATH"
      elif is_source_file "$FILE_PATH"; then
        CURRENT_STATE=$(cat "$TDD_STATE_FILE" 2>/dev/null || echo "unknown")
        if [ "$CURRENT_STATE" = "red" ]; then
          echo "green" > "$TDD_STATE_FILE"
          echo "TDD: source written after test (GREEN) — $FILE_PATH"
        else
          echo "TDD: source written without prior test — $FILE_PATH"
        fi
      fi
    fi
    ;;
  Bash)
    if [ -n "$COMMAND" ]; then
      case "$COMMAND" in
        *"cargo test"*|*"go test"*|*"pytest"*|*"npm test"*|*"npx jest"*|*"npx vitest"*|*"bats"*)
          # Check exit code from tool output (informational only)
          TOOL_EXIT=$(echo "$INPUT" | jq -r '.tool_output.exit_code // empty' 2>/dev/null) || true
          CURRENT_STATE=$(cat "$TDD_STATE_FILE" 2>/dev/null || echo "unknown")
          if [ "$TOOL_EXIT" = "0" ]; then
            echo "TDD: tests passed"
            if [ "$CURRENT_STATE" = "green" ]; then
              echo "refactor" > "$TDD_STATE_FILE"
              echo "TDD: ready to refactor (REFACTOR)"
            fi
          else
            echo "TDD: tests ran (check results)"
          fi
          ;;
      esac
    fi
    ;;
esac

# Informational only — never block
exit 0
