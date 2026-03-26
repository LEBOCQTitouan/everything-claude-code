#!/usr/bin/env bash
set -euo pipefail

# audit-mcp-versions.sh — Check pinned MCP server versions against npm registry (BL-026)
# Usage: ./scripts/audit-mcp-versions.sh [path/to/mcp-servers.json]

readonly NPM_REGISTRY_URL="https://registry.npmjs.org"
readonly PACKAGE_NAME_REGEX='^[@a-zA-Z0-9._/-]+$'

# --- Prerequisite check ---

check_prerequisites() {
  local missing=0
  for tool in curl jq; do
    if ! command -v "$tool" &>/dev/null; then
      echo "ERROR: required tool '$tool' not found. Please install it first." >&2
      missing=1
    fi
  done
  if [ "$missing" -eq 1 ]; then
    exit 2
  fi
}

# --- Parsing ---

parse_npx_servers() {
  local config_file="$1"
  jq -r '
    .mcpServers | to_entries[]
    | select(.value.command == "npx")
    | .value.args[1] // ""
  ' "$config_file"
}

extract_package_and_version() {
  local arg="$1"
  local package version

  # Handle scoped packages: @scope/name@version
  if [[ "$arg" == @*/*@* ]]; then
    # Last @ is the version separator for scoped packages
    package="${arg%@*}"
    version="${arg##*@}"
  elif [[ "$arg" == *@* ]] && [[ "$arg" != @* ]]; then
    # Unscoped: name@version
    package="${arg%@*}"
    version="${arg##*@}"
  else
    # No version suffix at all, or scoped without version
    package="$arg"
    version=""
  fi

  # Treat "latest" as unpinned
  if [ "$version" = "latest" ]; then
    version=""
  fi

  echo "$package"
  echo "$version"
}

# --- Registry query ---

fetch_latest_version() {
  local package="$1"

  # Validate package name against allowlist
  if [[ ! "$package" =~ $PACKAGE_NAME_REGEX ]]; then
    echo "WARN: invalid package name '$package' — skipping" >&2
    return 1
  fi

  local response
  if ! response=$(curl -sf "${NPM_REGISTRY_URL}/${package}/latest" 2>/dev/null); then
    return 1
  fi

  echo "$response" | jq -r '.version // empty'
}

# --- Output ---

print_header() {
  printf "%-50s %-15s %-15s %s\n" "PACKAGE" "PINNED" "LATEST" "STATUS"
  printf "%-50s %-15s %-15s %s\n" "-------" "------" "------" "------"
}

print_row() {
  local package="$1" pinned="$2" latest="$3" status="$4"
  printf "%-50s %-15s %-15s %s\n" "$package" "$pinned" "$latest" "$status"
}

# --- Main ---

main() {
  local config_file="${1:-mcp-configs/mcp-servers.json}"

  if [ ! -f "$config_file" ]; then
    echo "ERROR: config file not found: $config_file" >&2
    exit 2
  fi

  check_prerequisites

  local has_issues=0

  print_header

  while IFS= read -r arg; do
    [ -z "$arg" ] && continue

    local package version
    local result
    result=$(extract_package_and_version "$arg")
    mapfile -t parts <<< "$result"
    package="${parts[0]}"
    version="${parts[1]:-}"

    if [ -z "$version" ]; then
      print_row "$package" "unpinned" "-" "unpinned"
      has_issues=1
      continue
    fi

    local latest
    if ! latest=$(fetch_latest_version "$package"); then
      echo "WARN: could not fetch latest version for '$package' — skipping" >&2
      print_row "$package" "$version" "error" "skipped"
      has_issues=1
      continue
    fi

    if [ -z "$latest" ]; then
      echo "WARN: empty version response for '$package' — skipping" >&2
      print_row "$package" "$version" "error" "skipped"
      has_issues=1
      continue
    fi

    if [ "$version" = "$latest" ]; then
      print_row "$package" "$version" "$latest" "current"
    else
      print_row "$package" "$version" "$latest" "outdated"
      has_issues=1
    fi
  done < <(parse_npx_servers "$config_file")

  if [ "$has_issues" -eq 0 ]; then
    echo ""
    echo "All MCP server versions are current."
  fi

  return "$has_issues"
}

main "$@"
