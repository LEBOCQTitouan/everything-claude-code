# 0008. Designs Directory Convention

Date: 2026-03-22

## Status
Accepted

## Context
The interface-designer agent (BL-014) produces standalone design exploration artifacts — interface design comparisons with multiple alternatives, comparison matrices, and synthesis rationale. These artifacts are not part of the spec-driven pipeline (`docs/specs/`), which stores spec+design+tasks for workflow-tracked features. A separate location is needed for standalone design explorations that may be triggered conversationally or from within the `/design` command.

## Decision
Establish `docs/designs/` as the output directory for standalone design exploration artifacts. Files follow the naming convention `{module}-interface-{date}.md` where module is kebab-case and date is YYYY-MM-DD. If a file already exists at the target path, a numeric suffix `-N` is appended before the extension. When the interface-designer is invoked from within the `/design` pipeline, output follows the spec directory convention (`docs/specs/{slug}/`) instead.

## Consequences
- **Positive**: Clear separation between pipeline artifacts (docs/specs/) and standalone explorations (docs/designs/). Design explorations are preserved for team review and future reference.
- **Positive**: The naming convention is self-documenting — module name and date are immediately visible.
- **Negative**: New directory to maintain. Must be excluded from auto-cleanup scripts that target docs/.
- **Negative**: Contributors must learn which directory to use (docs/specs/ for pipeline, docs/designs/ for standalone).
