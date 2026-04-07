# Campaign Manifest

## Artifacts

| Artifact | Path | Status |
|----------|------|--------|

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|

| 1 | Root cause: CWD mismatch in hook subprocess | Agreed — hook subprocess CWD resolves to main repo, git returns .git not .git/worktrees/<name> | user |
| 2 | Fix strategy | Dotfile anchor at .claude/workflow/.state-dir, written by ecc-workflow init, read before git resolution | user |
| 3 | Write timing | Write on init only | user |
| 4 | Cleanup on reset | Delete .state-dir on reset --force | user |
| 5 | Spec handling | Fresh spec at 2026-04-07 | user |
| 6 | US-002 scope | Include untrack implement-done.md in this spec | user |
## Agent Outputs

| Agent | Phase | Summary |
|-------|-------|---------|

## Commit Trail

| SHA | Message |
|-----|---------|
