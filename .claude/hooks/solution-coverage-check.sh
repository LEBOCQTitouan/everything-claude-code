#!/usr/bin/env bash
set -uo pipefail
[ "${ECC_WORKFLOW_BYPASS:-}" = "1" ] && exit 0

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"
STATE_FILE="$PROJECT_DIR/.claude/workflow/state.json"

# No workflow active
[ ! -f "$STATE_FILE" ] && exit 0

# Read tool input from stdin (Claude hook protocol)
INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null) || exit 0
BASENAME=$(basename "$FILE_PATH" 2>/dev/null) || exit 0

# Only check when solution.md is written
[ "$BASENAME" != "solution.md" ] && exit 0

PLAN_FILE="$PROJECT_DIR/.claude/workflow/plan.md"
SOLUTION_FILE="$PROJECT_DIR/.claude/workflow/solution.md"

# Need both files
[ ! -f "$PLAN_FILE" ] && exit 0
[ ! -f "$SOLUTION_FILE" ] && exit 0

# Extract all AC-NNN.N from plan.md
PLAN_ACS=$(grep -oE 'AC-[0-9]+\.[0-9]+' "$PLAN_FILE" | sort -u)

# Extract all AC-NNN.N from solution.md (from PC table "Verifies AC" column)
SOLUTION_ACS=$(grep -oE 'AC-[0-9]+\.[0-9]+' "$SOLUTION_FILE" | sort -u)

# Find uncovered ACs
UNCOVERED=$(comm -23 <(echo "$PLAN_ACS") <(echo "$SOLUTION_ACS"))

if [ -n "$UNCOVERED" ]; then
  echo "BLOCKED: Uncovered acceptance criteria in solution.md:" >&2
  echo "$UNCOVERED" | while read -r ac; do
    echo "  - $ac has no covering pass condition" >&2
  done
  echo "" >&2
  echo "Every AC from plan.md must appear in at least one PC's 'Verifies AC' column." >&2

  # Rollback state to solution phase
  TMPFILE=$(mktemp "${PROJECT_DIR}/.claude/workflow/state.XXXXXX") || exit 2
  jq '.phase = "solution"' "$STATE_FILE" > "$TMPFILE" 2>/dev/null || { rm -f "$TMPFILE"; exit 2; }
  mv "$TMPFILE" "$STATE_FILE"
  echo "State rolled back to 'solution' phase." >&2
  exit 2
fi

# Check for Adversarial Review: PASS
if ! grep -q 'Adversarial Review: PASS' "$SOLUTION_FILE"; then
  echo "BLOCKED: solution.md missing 'Adversarial Review: PASS'." >&2
  echo "Run the adversarial review in the /solution command before proceeding." >&2

  # Rollback state to solution phase
  TMPFILE=$(mktemp "${PROJECT_DIR}/.claude/workflow/state.XXXXXX") || exit 2
  jq '.phase = "solution"' "$STATE_FILE" > "$TMPFILE" 2>/dev/null || { rm -f "$TMPFILE"; exit 2; }
  mv "$TMPFILE" "$STATE_FILE"
  echo "State rolled back to 'solution' phase." >&2
  exit 2
fi

exit 0
