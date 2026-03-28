# 0018. Worktree Segment Replaces Branch Segment

Date: 2026-03-27

## Status

Accepted

## Context

The ECC statusline already shows a git branch segment (`⎇ main`) when the user is inside a git repository. A new worktree segment (`🌳 wt-name (branch)`) was designed to give users visibility into which linked worktree they are operating in — a common scenario during `/implement` TDD waves where agents run in isolated worktrees.

Three display strategies were considered:

1. **Show both**: Render the branch segment and the worktree segment simultaneously. The branch info appears twice — once in `⎇ branch` and once inside the worktree segment parentheses.
2. **Worktree without branch**: Render the worktree segment as `🌳 wt-name` (name only, no branch). Branch info is still available in the standalone branch segment.
3. **Replace branch with worktree**: When inside a worktree, suppress the standalone branch segment and render only `🌳 wt-name (branch)`. When not in a worktree, render the branch segment as usual.

## Decision

When the user is inside a git worktree, the worktree segment (`🌳 wt-name (branch)`) replaces the standalone branch segment (`⎇ branch`). When not in a worktree, the branch segment renders normally and no worktree segment appears.

Option 1 was rejected because it duplicates branch information, wastes horizontal space, and adds visual noise with no benefit.

Option 2 was rejected because it strips branch context from the worktree segment, forcing the user to read two segments to understand both the worktree and the branch — the opposite of the goal.

Option 3 was chosen because the worktree segment already embeds the branch name in parentheses, making the standalone branch segment fully redundant when inside a worktree. Replacement eliminates duplication, preserves all information in a single cohesive segment, and saves horizontal space on narrow terminals.

## Consequences

**Positive:**

- No branch information is duplicated — the worktree segment is the single source of truth for git context when inside a worktree
- Horizontal space is conserved, which matters for narrow terminals and the priority-based truncation system
- The worktree context is always visible when relevant — users immediately know they are in a worktree and which branch it tracks
- The narrow variant (`🌳 name`, branch dropped) degrades gracefully without leaving an orphaned branch segment

**Negative:**

- Users lose the familiar `⎇` icon when inside a worktree; they see `🌳` instead
- Users who have muscle-memory for scanning the `⎇` position must adapt when switching between main working tree and worktrees
