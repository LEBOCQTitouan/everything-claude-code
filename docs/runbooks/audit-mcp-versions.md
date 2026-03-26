# Quarterly MCP Version Audit

Quarterly process to check pinned MCP server versions against current npm releases.

## Prerequisites

- **curl**: HTTP client (pre-installed on macOS/Linux)
- **jq**: JSON processor (`brew install jq` on macOS, `apt install jq` on Ubuntu)
- **Network access**: Script queries `registry.npmjs.org` over HTTPS

## Running the Script

From the repository root:

```bash
./scripts/audit-mcp-versions.sh
```

To use a custom config path:

```bash
./scripts/audit-mcp-versions.sh path/to/mcp-servers.json
```

The default path is `mcp-configs/mcp-servers.json`.

## Interpreting Output

The script outputs a table with four columns:

| Column | Meaning |
|--------|---------|
| PACKAGE | npm package name |
| PINNED | Version currently in mcp-servers.json |
| LATEST | Latest version on npm registry |
| STATUS | `current`, `outdated`, `unpinned`, or `skipped` |

Exit codes:

- **0**: All packages are current
- **1**: One or more packages are outdated, unpinned, or unreachable
- **2**: Missing prerequisites (curl or jq not installed)

HTTP-type MCP servers (Vercel, Cloudflare, ClickHouse) are skipped automatically — they have no npm version to check.

## Updating Pinned Versions

When the script reports `outdated` packages:

1. Review the changelog of the outdated package for breaking changes
2. Update the version in `mcp-configs/mcp-servers.json` (in the `args` array)
3. Test the MCP server with the new version
4. Commit: `chore: bump <package> to <version>`

For `unpinned` packages (using `@latest`):

1. Find the current version: `npm view <package> version`
2. Replace `@latest` with `@<version>` in the `args` array
3. Test and commit

## Updating the audit_reminder Date

After completing the audit, update the `_comments.audit_reminder` field in `mcp-configs/mcp-servers.json`:

```json
"audit_reminder": "Review and update pinned versions quarterly (next: YYYY-MM-DD)"
```

Set the date to approximately 3 months from today. Also update `pinned_date` to today's date if any versions were changed.

## Schedule

The audit runs quarterly. The next audit date is tracked in `mcp-configs/mcp-servers.json` under `_comments.audit_reminder`.
