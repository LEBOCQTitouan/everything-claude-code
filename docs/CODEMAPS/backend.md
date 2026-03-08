<!-- Generated: 2026-03-08 | Files scanned: 48 | Token estimate: ~900 -->

# Backend — Core Library & Hooks

## Library Modules (`src/lib/`)

```
ansi.ts (39 LOC)
  └─ ANSI color utilities: bold, dim, red, green, yellow, cyan, etc.
  └─ stripAnsi() — remove escape codes
  └─ Respects NO_COLOR env var

utils.ts (473 LOC)
  └─ Platform: isWindows, isMacOS, isLinux
  └─ Dirs: getHomeDir, getClaudeDir, getSessionsDir, ensureDir
  └─ File I/O: readFile, writeFile, appendFile, findFiles, grepFile
  └─ Git: isGitRepo, getGitRepoName, getGitModifiedFiles
  └─ Shell: runCommand, commandExists, readStdinJson

package-manager.ts (364 LOC)
  └─ detectFromLockFile → detectFromPackageJson → getPackageManager
  └─ getRunCommand, getExecCommand, setPreferredPackageManager

session-manager.ts (388 LOC)
  └─ parseSessionFilename → getSessionPath → listSessions
  └─ parseSessionMetadata, deleteSession

session-aliases.ts (469 LOC)
  └─ loadAliases → createAlias / deleteAlias / listAliases

project-detect.ts (337 LOC)
  └─ detectLanguages (12 rules) → detectFrameworks (24 rules)
  └─ detectProjectType → { languages, frameworks, primary }

hook-flags.ts (68 LOC)
  └─ getHookProfile → getDisabledHookIds → isHookEnabled
  └─ Profiles: minimal, standard, strict
```

## Install Pipeline (`src/lib/`)

```
detect.ts (234 LOC)
  └─ detectAgents, detectCommands, detectSkills, detectRules, detectHooks
  └─ detect() → full scan → generateReport()

manifest.ts (155 LOC)
  └─ readManifest → createManifest / updateManifest → writeManifest
  └─ isEccManaged, isEccManagedRule, diffFileList

merge.ts (647 LOC)
  └─ mergeDirectory (agents/commands per-file)
  └─ mergeSkills (per-skill-directory)
  └─ mergeRules (grouped by language)
  └─ Interactive diff review: promptFileReview → reviewFile → applyReviewChoice
  └─ Per-category applyAll scoping (A/K/M resets between agents/commands/skills)
  └─ Pre-scan with contentsDiffer → skip unchanged files
  └─ printCategoryHeader: "--- Agents (3 changed out of 19) ---"
  └─ ReviewChoice: 'accept' | 'keep' | 'smart-merge'

smart-merge.ts (363 LOC)
  └─ computeLineDiff (LCS algorithm, O(n*m) guard at 1M)
  └─ formatSideBySideDiff (colored, paired blocks)
  └─ generateDiff (public API, unchanged signature)
  └─ smartMerge → Claude CLI invocation
  └─ contentsDiffer (Buffer.compare byte-level)
  └─ readFileForMerge (null for missing files)

gitignore.ts (153 LOC)
  └─ ensureGitignoreEntries (append-only)
  └─ findTrackedEccFiles → gitUntrack
```

## Hook Implementations (`src/hooks/`, 23 files)

```
Lifecycle:
  session-start.ts → load context, detect project, list aliases
  session-end.ts → save session summary
  session-end-marker.ts → mark end in metrics

Pre-Tool:
  pre-bash-dev-server-block.ts → block dev servers outside tmux
  pre-bash-tmux-reminder.ts → tmux reminder for long-running cmds
  pre-bash-git-push-reminder.ts → confirm before push
  pre-write-doc-warn.ts → warn about non-standard docs
  suggest-compact.ts → suggest manual compaction
  pre-compact.ts → save state before compaction

Post-Tool:
  post-bash-pr-created.ts → PR creation events
  post-bash-build-complete.ts → build completion
  post-edit-format.ts → auto-format after edits
  post-edit-typecheck.ts → type-check TS edits
  post-edit-console-warn.ts → console.log detection
  doc-coverage-reminder.ts → undocumented export nudge
  quality-gate.ts → quality assurance checks

Evaluation:
  evaluate-session.ts → session quality & learning
  cost-tracker.ts → API cost tracking per model
  stop-uncommitted-reminder.ts → remind to commit
  check-console-log.ts → console.log scan at session end

Execution:
  run-with-flags.ts → profile-gated hook execution
  check-hook-enabled.ts → hook enable/disable check
```

## CI Validators (`src/ci/`, 6 files)

```
validate-agents.ts → frontmatter schema (model, tools required)
validate-commands.ts → command structure
validate-hooks.ts → hooks.json schema
validate-skills.ts → skill directory structure
validate-rules.ts → rules structure
validate-no-personal-paths.ts → prevent /Users/xxx leaks
```

## Doc System Agents (5 agents, parallel pipeline)

```
doc-orchestrator (sonnet) → coordinates pipeline
  ├─ Phase 1: doc-analyzer (sonnet) → structure, API surface, domain concepts
  ├─ Phase 2 (parallel per module):
  │    ├─ doc-generator (haiku) → doc comments, summaries, glossary, changelog
  │    ├─ doc-validator (sonnet) → accuracy, quality scoring, contradictions
  │    └─ doc-reporter (haiku) → coverage %, diffs, regressions
  └─ Phase 3: index assembly + cross-references
```
