---
name: cartography-processing
description: Cartography delta processing phase for the doc-orchestrator pipeline. Reads pending deltas, dispatches cartographer agent, archives processed deltas.
origin: ECC
---

# Cartography Processing Phase

Phase 1.5 of the doc-orchestrator pipeline. Processes pending cartography deltas into journey, flow, and element documentation.

## When to Use

Referenced by `agents/doc-orchestrator.md` during Phase 1.5. Can also be run standalone via `/doc-suite --phase=cartography`.

## Prerequisites

- `.claude/cartography/` directory may contain `pending-delta-*.json` files written by the `stop:cartography` hook
- `docs/cartography/` directory with `journeys/`, `flows/`, and `elements/` subdirectories

## Execution Steps

### 1. Check for Pending Deltas

Scan `.claude/cartography/` for files matching `pending-delta-*.json`.

- If the directory does not exist or contains no matching files: skip with log "No pending cartography deltas" and return
- Count and report: "Processing N pending cartography deltas"

### 2. Acquire Lock

Use shell `flock(1)` to acquire an exclusive lock on `.claude/cartography/cartography-merge.lock`:

```bash
exec 200>>.claude/cartography/cartography-merge.lock
flock -n 200 || { echo "Another session is processing deltas"; exit 0; }
```

If the lock cannot be acquired (another session is processing), skip with a log message.

### 3. Parse and Validate Deltas

For each `pending-delta-*.json` file:

1. Read the file content
2. Parse as JSON — if malformed or invalid JSON, log as error ("Skipping malformed delta: <filename>"), skip the file (do NOT archive it), and continue with remaining deltas
3. Check if already processed (exists in `.claude/cartography/processed/`) — if so, skip
4. Sort remaining deltas by timestamp ascending

### 4. Dispatch Cartographer Agent

For each valid, unprocessed delta:

1. Build context with: the parsed delta, existing journey/flow content for the classification slug, flow file slugs, and external I/O patterns
2. Dispatch the `cartographer` agent with the context
3. The agent returns a JSON envelope:
   ```json
   {"status": "success"|"error", "type": "journey"|"flow"|"element", "file_path": "<relative-path>", "content": "<markdown>", "error": "<message>|null"}
   ```
4. Parse the JSON envelope response:
   - Validate `type` against allowlist: `["journey", "flow", "element"]`
   - Validate `file_path` is under `docs/cartography/` (path traversal protection — derive actual write path from `type` + slug using `derive_slug` algorithm)
   - If `status` is `"error"` or JSON is invalid: log the error, do NOT archive the failed delta, continue with remaining deltas
   - If `status` is `"success"`: write `content` to the appropriate file under `docs/cartography/<type>s/`

### 5. Single Git Commit

After ALL deltas have been processed:

```bash
git add docs/cartography/
git commit -m "docs: process cartography deltas"
```

One commit for all cartography changes — the doc-orchestrator owns the transaction.

### 6. Archive Processed Deltas

Move successfully processed delta files to `.claude/cartography/processed/`:

```bash
mkdir -p .claude/cartography/processed/
mv .claude/cartography/pending-delta-<id>.json .claude/cartography/processed/
```

Deltas that failed processing are NOT archived — they remain in the pending directory for retry on the next run.

### 7. Release Lock

The `flock` is automatically released when the file descriptor is closed (script exit).

## Error Handling

- **Malformed delta JSON**: Logged as error, skipped, not archived. Processing continues with remaining deltas.
- **Agent dispatch failure**: Delta not archived, error reported. Remaining deltas continue.
- **Invalid agent JSON envelope**: Treated as agent failure — delta not archived.
- **Git commit failure**: Log warning. Deltas are still archived if the agent produced valid output.
- **Lock contention**: Skip gracefully with a message.

## Slug Derivation

Use the `derive_slug` algorithm for file path construction: lowercase, replace non-alphanumeric characters with hyphens, collapse multiple hyphens, truncate at 60 characters, strip leading/trailing hyphens.
