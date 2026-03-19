#!/usr/bin/env bash
set -uo pipefail
[ "${ECC_WORKFLOW_BYPASS:-}" = "1" ] && exit 0

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"
STATE_FILE="$PROJECT_DIR/.claude/workflow/state.json"

# No workflow active
[ ! -f "$STATE_FILE" ] && exit 0

PHASE=$(jq -r '.phase // "done"' "$STATE_FILE" 2>/dev/null) || exit 0

# Only active during solution phase
[ "$PHASE" != "solution" ] && exit 0

# Read tool input from stdin (Claude hook protocol)
INPUT=$(cat)
TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // empty' 2>/dev/null) || exit 0

# Only check Write operations
[ "$TOOL_NAME" != "Write" ] && exit 0

PLAN_FILE="$PROJECT_DIR/.claude/workflow/plan.md"

# 1. File exists
if [ ! -f "$PLAN_FILE" ]; then
  echo "BLOCKED: plan.md not found. Cannot proceed to solution phase without a spec." >&2
  exit 2
fi

# 2. Contains ## User Stories
if ! grep -q '^## User Stories' "$PLAN_FILE"; then
  echo "BLOCKED: plan.md missing '## User Stories' section." >&2
  exit 2
fi

# 3. Contains at least one ### US- heading
if ! grep -q '^### US-' "$PLAN_FILE"; then
  echo "BLOCKED: plan.md has no user stories (no '### US-' headings found)." >&2
  exit 2
fi

# 4. Contains AC- patterns
if ! grep -q 'AC-' "$PLAN_FILE"; then
  echo "BLOCKED: plan.md has no acceptance criteria (no 'AC-' patterns found)." >&2
  exit 2
fi

# 5. Contains ## Doc Impact Assessment
if ! grep -q '^## Doc Impact Assessment' "$PLAN_FILE"; then
  echo "BLOCKED: plan.md missing '## Doc Impact Assessment' section." >&2
  exit 2
fi

# 6. Contains Adversarial Review: PASS
if ! grep -q 'Adversarial Review: PASS' "$PLAN_FILE"; then
  echo "BLOCKED: plan.md missing 'Adversarial Review: PASS'." >&2
  echo "Run the adversarial review in the /plan-* command before proceeding to /solution." >&2
  exit 2
fi

exit 0
