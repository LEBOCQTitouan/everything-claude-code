# Campaign Manifest

## Artifacts

| Artifact | Path | Status |
|----------|------|--------|

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|

| 1 | Which memory system is in scope for the prune fix? | Both systems (file-based agent memory + SQLite-tiered MemoryStore) | user |
| 2 | For cartography, do we fix the symptom (filter + dedupe, keep session-boundary trigger) or the root cause (switch trigger to commit-boundary)? | Both: filter+dedupe now in this spec, trigger change deferred to a new BL | user |
| 3 | Which adjacent concerns do we bundle into this spec vs defer? | All four: ERR-002 fix + stop:daily-summary filter + one-shot memory prune CLI + legacy dead-code cleanup (process_cartography and related, ~500 LOC) | user |
| 4 | How should we validate the filter/dedupe works? | Full coverage + regression corpus: ~25 new tests + 10 fixture files committed under tests/fixtures/cartography-corpus/ that future filter changes must preserve | user |
| 5 | What's the policy for archived deltas, stale memory files, and rollback path? | Forward-only + explicit one-shot cleanup: archived deltas untouched (historical), run ecc memory prune --orphaned-backlogs once to delete the 6 stale files before merge, rollback = git revert | user |
| 6 | PR split strategy for the 22-file bundled scope? | Split but keep one spec/design pair: PR1 = US-001/002/003/004/005/008 (cartography); PR2 = US-006/007 (memory, depends on PR1 merged). Each /implement runs against the same design doc. | user |
| 7 | Scope for this /implement run? | Both PRs in one run (all 68 PCs). User acknowledged 2-5h wall time and context compaction risk; accepted trade-off vs the staged delivery originally chosen in grill-me. | user |
## Agent Outputs

| Agent | Phase | Summary |
|-------|-------|---------|

## Commit Trail

| SHA | Message |
|-----|---------|
