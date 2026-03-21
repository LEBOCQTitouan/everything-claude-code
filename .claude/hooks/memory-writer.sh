#!/usr/bin/env bash
set -uo pipefail

# Memory writer for cross-session persistence (BL-027)
# Usage:
#   memory-writer.sh action <action_type> <description> <outcome> <artifacts_json>
#   memory-writer.sh work-item <phase> <description> <concern>
#
# Action types: plan, solution, implement, verify, fix, audit, review, other
# Phases: plan, solution, implementation

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"
MEMORY_DIR="$PROJECT_DIR/docs/memory"
ACTION_LOG="$MEMORY_DIR/action-log.json"
WORK_ITEMS_DIR="$MEMORY_DIR/work-items"

# Ensure memory directory exists
ensure_dirs() {
  mkdir -p "$MEMORY_DIR" "$WORK_ITEMS_DIR"
  if [ ! -f "$ACTION_LOG" ]; then
    echo "[]" > "$ACTION_LOG"
  fi
}

# Generate slug from description: lowercase, hyphenated, max 40 chars, alphanumeric only
make_slug() {
  local desc="$1"
  echo "$desc" \
    | tr '[:upper:]' '[:lower:]' \
    | sed 's/[^a-z0-9 ]//g' \
    | sed 's/  */ /g' \
    | sed 's/ /-/g' \
    | cut -c1-40 \
    | sed 's/-$//'
}

# Write an action log entry (append-only, atomic)
write_action() {
  local action_type="$1"
  local description="$2"
  local outcome="$3"
  local artifacts_json="$4"
  local timestamp
  local session_id

  timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  session_id="${CLAUDE_SESSION_ID:-unknown}"

  ensure_dirs

  local entry
  entry=$(jq -n \
    --arg ts "$timestamp" \
    --arg sid "$session_id" \
    --arg at "$action_type" \
    --arg desc "$description" \
    --arg out "$outcome" \
    --argjson arts "$artifacts_json" \
    '{
      timestamp: $ts,
      session_id: $sid,
      action_type: $at,
      description: $desc,
      artifacts: $arts,
      outcome: $out,
      tags: []
    }')

  # Atomic append via mktemp+mv
  local tmpfile
  tmpfile=$(mktemp "${MEMORY_DIR}/action-log.XXXXXX") || return 1
  jq ". += [$entry]" "$ACTION_LOG" > "$tmpfile" || { rm -f "$tmpfile"; return 1; }
  mv "$tmpfile" "$ACTION_LOG"
}

# Write a work item file (plan.md, solution.md, or implementation.md)
write_work_item() {
  local phase="$1"
  local description="$2"
  local concern="$3"
  local today
  local slug
  local item_dir
  local target_file

  today=$(date -u +"%Y-%m-%d")
  slug=$(make_slug "$description")
  item_dir="$WORK_ITEMS_DIR/${today}-${slug}"

  ensure_dirs
  mkdir -p "$item_dir"

  target_file="$item_dir/${phase}.md"

  local timestamp
  timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

  # If file exists (re-entry), append a revision block instead of overwriting
  if [ -f "$target_file" ]; then
    cat >> "$target_file" <<EOF

## Revision

Date: $timestamp

Re-entry detected. This phase was re-executed.
EOF
    return 0
  fi

  # Write new file with deterministic H2 sections
  case "$phase" in
    plan)
      cat > "$target_file" <<EOF
# Plan: $description

## Context

Concern: $concern
Created: $timestamp

## Decisions

(Populated by /plan-* command output)

## User Stories

(Populated by /plan-* command output)

## Outcome

Phase completed at $timestamp
EOF
      ;;
    solution)
      cat > "$target_file" <<EOF
# Solution: $description

## Context

Concern: $concern
Created: $timestamp

## File Changes

(Populated by /solution command output)

## Pass Conditions

(Populated by /solution command output)

## Outcome

Phase completed at $timestamp
EOF
      ;;
    implementation)
      cat > "$target_file" <<EOF
# Implementation: $description

## Context

Concern: $concern
Created: $timestamp

## Changes Made

(Populated from implement-done.md)

## Test Results

(Populated from implement-done.md)

## Outcome

Phase completed at $timestamp
EOF
      ;;
    *)
      echo "ERROR: Unknown phase '$phase'. Use: plan, solution, implementation" >&2
      return 1
      ;;
  esac
}

# Main dispatch
CMD="${1:-}"
shift || true

case "$CMD" in
  action)
    write_action "${1:-}" "${2:-}" "${3:-}" "${4:-[]}"
    ;;
  work-item)
    write_work_item "${1:-}" "${2:-}" "${3:-}"
    ;;
  *)
    echo "Usage: memory-writer.sh action|work-item ..." >&2
    exit 1
    ;;
esac
