#!/usr/bin/env bats

load test_helper

SCRIPT_DIR="$(cd "$(dirname "$BATS_TEST_FILENAME")" && pwd)"
STATUSLINE="${SCRIPT_DIR}/../../statusline/statusline-command.sh"

# --- visible_width() unit tests ---

@test "visible_width counts ASCII characters correctly" {
  export LC_ALL=C.UTF-8
  eval "$(grep -A1 '^strip_ansi()' "$STATUSLINE")"
  eval "$(grep -A1 '^visible_width()' "$STATUSLINE")"
  result=$(visible_width "hello")
  [ "$result" -eq 5 ]
}

@test "visible_width counts Unicode ◆ Model as 7" {
  export LC_ALL=C.UTF-8
  eval "$(grep -A1 '^strip_ansi()' "$STATUSLINE")"
  eval "$(grep -A1 '^visible_width()' "$STATUSLINE")"
  result=$(visible_width "◆ Model")
  [ "$result" -eq 7 ]
}

@test "visible_width counts Unicode bar ████░░░░ as 8" {
  export LC_ALL=C.UTF-8
  eval "$(grep -A1 '^strip_ansi()' "$STATUSLINE")"
  eval "$(grep -A1 '^visible_width()' "$STATUSLINE")"
  result=$(visible_width "████░░░░")
  [ "$result" -eq 8 ]
}

@test "visible_width counts ⎇ branch as 8" {
  export LC_ALL=C.UTF-8
  eval "$(grep -A1 '^strip_ansi()' "$STATUSLINE")"
  eval "$(grep -A1 '^visible_width()' "$STATUSLINE")"
  result=$(visible_width "⎇ branch")
  [ "$result" -eq 8 ]
}

# --- Integration tests: rate limit visibility ---

@test "rate limits visible at COLUMNS=120" {
  local json='{"model":{"display_name":"Opus"},"context_window":{"used_percentage":42},"rate_limits":{"five_hour":{"used_percentage":26},"seven_day":{"used_percentage":47}},"cost":{"total_cost_usd":0,"total_duration_ms":0,"total_lines_added":0,"total_lines_removed":0}}'
  output=$(echo "$json" | COLUMNS=120 CACHE_DIR="$BATS_TEST_TMPDIR" bash "$STATUSLINE")
  clean=$(strip_ansi "$output")
  [[ "$clean" == *"5h:"* ]]
}

@test "graceful degradation at COLUMNS=50" {
  local json='{"model":{"display_name":"Opus"},"context_window":{"used_percentage":42},"rate_limits":{"five_hour":{"used_percentage":26},"seven_day":{"used_percentage":47}},"cost":{"total_cost_usd":0,"total_duration_ms":0,"total_lines_added":0,"total_lines_removed":0}}'
  output=$(echo "$json" | COLUMNS=50 CACHE_DIR="$BATS_TEST_TMPDIR" bash "$STATUSLINE")
  clean=$(strip_ansi "$output")
  [[ "$clean" == *"Opus"* ]]
}
