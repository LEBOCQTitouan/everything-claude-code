#!/usr/bin/env bats

# Test suite for scripts/audit-mcp-versions.sh (BL-026)
# Tests use fixture JSON and mocked curl to avoid network dependency.

bats_require_minimum_version 1.5.0

SCRIPT_DIR="$(cd "$(dirname "$BATS_TEST_FILENAME")/.." && pwd)"
SCRIPT="$SCRIPT_DIR/audit-mcp-versions.sh"
FIXTURES_DIR="$BATS_TEST_TMPDIR/fixtures"

setup() {
  mkdir -p "$FIXTURES_DIR"
}

teardown() {
  rm -rf "$FIXTURES_DIR"
}

# Helper: create a fixture mcp-servers.json
create_fixture() {
  local file="$FIXTURES_DIR/mcp-servers.json"
  cat > "$file" <<'FIXTURE'
{
  "mcpServers": {
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github@2025.4.8"]
    },
    "vercel": {
      "type": "http",
      "url": "https://mcp.vercel.com"
    },
    "context7": {
      "command": "npx",
      "args": ["-y", "@upstash/context7-mcp@latest"]
    },
    "supabase": {
      "command": "npx",
      "args": ["-y", "@supabase/mcp-server-supabase@0.7.0", "--project-ref=MY_PROJECT"]
    },
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem@2026.1.14", "/path/to/projects"]
    }
  },
  "_comments": {
    "pinned_date": "2026-03-21"
  }
}
FIXTURE
  echo "$file"
}

# Helper: create a fixture with a no-version-suffix package
create_fixture_no_version() {
  local file="$FIXTURES_DIR/mcp-servers-noversion.json"
  cat > "$file" <<'FIXTURE'
{
  "mcpServers": {
    "bare": {
      "command": "npx",
      "args": ["-y", "some-bare-package"]
    }
  }
}
FIXTURE
  echo "$file"
}

# Helper: create a fixture where all versions match latest (for mocked curl)
create_fixture_all_current() {
  local file="$FIXTURES_DIR/mcp-servers-current.json"
  cat > "$file" <<'FIXTURE'
{
  "mcpServers": {
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github@2025.4.8"]
    }
  }
}
FIXTURE
  echo "$file"
}

# Helper: create a fixture with one outdated package
create_fixture_outdated() {
  local file="$FIXTURES_DIR/mcp-servers-outdated.json"
  cat > "$file" <<'FIXTURE'
{
  "mcpServers": {
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github@1.0.0"]
    }
  }
}
FIXTURE
  echo "$file"
}

# Helper: create a fixture with an unreachable package
create_fixture_unreachable() {
  local file="$FIXTURES_DIR/mcp-servers-unreachable.json"
  cat > "$file" <<'FIXTURE'
{
  "mcpServers": {
    "nonexistent": {
      "command": "npx",
      "args": ["-y", "@nonexistent/this-does-not-exist-zzz@1.0.0"]
    },
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github@2025.4.8"]
    }
  }
}
FIXTURE
  echo "$file"
}

# Helper: create a mixed-state fixture
create_fixture_mixed() {
  local file="$FIXTURES_DIR/mcp-servers-mixed.json"
  cat > "$file" <<'FIXTURE'
{
  "mcpServers": {
    "current-pkg": {
      "command": "npx",
      "args": ["-y", "current-pkg@1.0.0"]
    },
    "outdated-pkg": {
      "command": "npx",
      "args": ["-y", "outdated-pkg@0.5.0"]
    },
    "unpinned-pkg": {
      "command": "npx",
      "args": ["-y", "unpinned-pkg@latest"]
    },
    "http-server": {
      "type": "http",
      "url": "https://example.com/mcp"
    }
  }
}
FIXTURE
  echo "$file"
}

# Mock curl: returns controlled version responses
# Helper: find URL argument from curl args (position-agnostic)
find_url_in_args() {
  for a in "$@"; do
    if [[ "$a" == *"registry.npmjs.org"* ]]; then
      echo "$a"
      return
    fi
  done
}
export -f find_url_in_args

mock_curl_current() {
  curl() {
    local url
    url=$(find_url_in_args "$@")
    if [[ "$url" == *"server-github"* ]]; then
      echo '{"version":"2025.4.8"}'
    elif [[ "$url" == *"server-supabase"* ]]; then
      echo '{"version":"0.7.0"}'
    elif [[ "$url" == *"server-filesystem"* ]]; then
      echo '{"version":"2026.1.14"}'
    elif [[ "$url" == *"current-pkg"* ]]; then
      echo '{"version":"1.0.0"}'
    elif [[ "$url" == *"outdated-pkg"* ]]; then
      echo '{"version":"2.0.0"}'
    else
      echo '{"version":"1.0.0"}'
    fi
    return 0
  }
  export -f curl
}

mock_curl_outdated() {
  curl() {
    echo '{"version":"99.0.0"}'
    return 0
  }
  export -f curl
}

mock_curl_unreachable() {
  curl() {
    local url
    url=$(find_url_in_args "$@")
    if [[ "$url" == *"this-does-not-exist"* ]]; then
      return 22
    fi
    echo '{"version":"2025.4.8"}'
    return 0
  }
  export -f curl
}

# ============================================================
# PC-008: Exits with error when required tool is missing
# ============================================================

@test "exits with error when required tool is missing" {
  # Run script with PATH restricted to exclude jq
  run -2 env PATH="/usr/bin:/bin" bash "$SCRIPT" "$FIXTURES_DIR/mcp-servers.json"
  [[ "$output" == *"required"* ]] || [[ "$output" == *"not found"* ]] || [[ "$output" == *"jq"* ]]
}

# ============================================================
# PC-004: Skips HTTP-type servers
# ============================================================

@test "skips HTTP-type servers" {
  local fixture
  fixture=$(create_fixture)
  mock_curl_current
  run bash "$SCRIPT" "$fixture"
  # vercel is HTTP type — should not appear in output
  [[ "$output" != *"vercel"* ]]
}

# ============================================================
# PC-001: Extracts pinned version from args
# ============================================================

@test "extracts pinned version from args" {
  local fixture
  fixture=$(create_fixture)
  mock_curl_current
  run bash "$SCRIPT" "$fixture"
  # Should show github with version 2025.4.8
  [[ "$output" == *"server-github"* ]]
  [[ "$output" == *"2025.4.8"* ]]
}

# ============================================================
# PC-009: Outputs table with correct columns
# ============================================================

@test "outputs table with correct columns" {
  local fixture
  fixture=$(create_fixture)
  mock_curl_current
  run bash "$SCRIPT" "$fixture"
  # Table header should contain these column names
  [[ "$output" == *"PACKAGE"* ]]
  [[ "$output" == *"PINNED"* ]]
  [[ "$output" == *"LATEST"* ]]
  [[ "$output" == *"STATUS"* ]]
}

# ============================================================
# PC-003: Flags no-version-suffix as unpinned
# ============================================================

@test "flags no-version-suffix as unpinned" {
  local fixture
  fixture=$(create_fixture_no_version)
  mock_curl_current
  run bash "$SCRIPT" "$fixture"
  [[ "$output" == *"unpinned"* ]]
}

# ============================================================
# PC-002: Flags @latest as unpinned
# ============================================================

@test "flags @latest as unpinned" {
  local fixture
  fixture=$(create_fixture)
  mock_curl_current
  run bash "$SCRIPT" "$fixture"
  [[ "$output" == *"context7"* ]]
  [[ "$output" == *"unpinned"* ]]
}

# ============================================================
# PC-010: Exits 1 when unpinned packages exist
# ============================================================

@test "exits 1 when unpinned packages exist" {
  local fixture
  fixture=$(create_fixture)
  mock_curl_current
  run bash "$SCRIPT" "$fixture"
  # context7 uses @latest → unpinned → exit 1
  [ "$status" -eq 1 ]
}

# ============================================================
# PC-005: Exits 0 when all versions are current
# ============================================================

@test "exits 0 when all versions are current" {
  local fixture
  fixture=$(create_fixture_all_current)
  mock_curl_current
  run bash "$SCRIPT" "$fixture"
  [ "$status" -eq 0 ]
  [[ "$output" == *"current"* ]]
}

# ============================================================
# PC-006: Exits 1 when any package is outdated
# ============================================================

@test "exits 1 when any package is outdated" {
  local fixture
  fixture=$(create_fixture_outdated)
  mock_curl_outdated
  run bash "$SCRIPT" "$fixture"
  [ "$status" -eq 1 ]
  [[ "$output" == *"outdated"* ]]
}

# ============================================================
# PC-007: Warns and skips unreachable registry
# ============================================================

@test "warns and skips unreachable registry" {
  local fixture
  fixture=$(create_fixture_unreachable)
  mock_curl_unreachable
  run bash "$SCRIPT" "$fixture"
  # Should warn about the unreachable package
  [[ "$output" == *"WARN"* ]] || [[ "$output" == *"warn"* ]] || [[ "$output" == *"skip"* ]] || [[ "$output" == *"error"* ]]
  # Should still show the reachable package
  [[ "$output" == *"server-github"* ]]
}

# ============================================================
# PC-015: Ignores trailing args after package
# ============================================================

@test "ignores trailing args after package" {
  local fixture
  fixture=$(create_fixture)
  mock_curl_current
  run bash "$SCRIPT" "$fixture"
  # supabase has --project-ref trailing arg — should not appear in package name or version
  [[ "$output" != *"project-ref"* ]]
  # supabase should still be listed with correct version
  [[ "$output" == *"server-supabase"* ]]
  [[ "$output" == *"0.7.0"* ]]
}

# ============================================================
# PC-016: Mixed state output
# ============================================================

@test "mixed state output" {
  local fixture
  fixture=$(create_fixture_mixed)
  mock_curl_current
  run bash "$SCRIPT" "$fixture"
  # Should have current, outdated, and unpinned entries
  [[ "$output" == *"current"* ]]
  [[ "$output" == *"outdated"* ]]
  [[ "$output" == *"unpinned"* ]]
  # HTTP server should not appear
  [[ "$output" != *"http-server"* ]]
  # Should exit 1 (has outdated and unpinned)
  [ "$status" -eq 1 ]
}

# ============================================================
# PC-014: Runs against real mcp-servers.json
# ============================================================

@test "runs against real mcp-servers.json" {
  if [ "${SKIP_NETWORK:-0}" = "1" ]; then
    skip "SKIP_NETWORK=1 — skipping network test"
  fi
  local real_config="$SCRIPT_DIR/../mcp-configs/mcp-servers.json"
  if [ ! -f "$real_config" ]; then
    skip "Real mcp-servers.json not found"
  fi
  run bash "$SCRIPT" "$real_config"
  # Should contain known npx packages
  [[ "$output" == *"server-github"* ]]
  [[ "$output" == *"firecrawl"* ]]
  # Should NOT contain HTTP servers
  [[ "$output" != *"vercel"* ]]
}

# ============================================================
# Security: Rejects invalid package names
# ============================================================

@test "rejects invalid package names with injection characters" {
  local file="$FIXTURES_DIR/mcp-injection.json"
  cat > "$file" <<'FIXTURE'
{
  "mcpServers": {
    "evil": {
      "command": "npx",
      "args": ["-y", "evil-pkg;rm -rf /@1.0.0"]
    }
  }
}
FIXTURE
  mock_curl_current
  run bash "$SCRIPT" "$file"
  [[ "$output" == *"WARN"* ]] || [[ "$output" == *"invalid"* ]] || [[ "$output" == *"skip"* ]]
}

# ============================================================
# PC-011: Runbook exists with required sections
# ============================================================

@test "runbook exists with required sections" {
  local runbook="$SCRIPT_DIR/../docs/runbooks/audit-mcp-versions.md"
  [ -f "$runbook" ]
  # Check required sections
  grep -q "Prerequisites\|prerequisites" "$runbook"
  grep -q -i "how to run\|running the script\|usage" "$runbook"
  grep -q -i "interpret\|output" "$runbook"
  grep -q -i "update.*pin\|updating.*version" "$runbook"
  grep -q -i "audit_reminder" "$runbook"
}
