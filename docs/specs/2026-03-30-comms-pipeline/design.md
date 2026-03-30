# Design Preview: Autonomous Comms Pipeline (BL-109)

## Design Summary

5 file changes (4 CREATE, 1 MODIFY), all Markdown. No Rust code. No compilation. 15 pass conditions verifying file existence, frontmatter correctness, content structure, and ECC convention compliance.

**Files:**
1. `agents/comms-generator.md` — orchestrator agent
2. `skills/comms-strategy/SKILL.md` — per-channel strategy definitions
3. `skills/comms-adapter/SKILL.md` — destination adapter patterns
4. `skills/comms-redactor/SKILL.md` — secret redaction rules
5. `.gitignore` — add `comms/` entry

**No Rust code, no tests, no build step.** Verification is structural: frontmatter parsing, content grep, file existence checks.

## Architecture Preview

### No ARCHITECTURE.md changes needed
This is a content-layer-only addition. The Rust hexagonal architecture is untouched.

### No Mermaid diagrams needed
Agent orchestration flow is documented within the agent file itself.

### No bounded context changes
Comms is an agent-layer concern, not a domain bounded context.

## Pass Conditions Overview

| ID | Type | Description | Command |
|----|------|-------------|---------|
| PC-001 | lint | comms-generator.md exists with valid frontmatter | `head -10 agents/comms-generator.md \| grep -c 'name: comms-generator'` |
| PC-002 | lint | Agent has required tools | `grep -c 'Read.*Write.*Bash.*Grep.*Glob.*Agent' agents/comms-generator.md` |
| PC-003 | lint | Agent has model field | `grep -c 'model: sonnet' agents/comms-generator.md` |
| PC-004 | lint | Agent references all 3 skills | `grep -c 'comms-strategy.*comms-adapter.*comms-redactor' agents/comms-generator.md` |
| PC-005 | lint | Agent includes TodoWrite with graceful degradation | `grep -c 'TodoWrite' agents/comms-generator.md` |
| PC-006 | lint | comms-strategy skill exists with valid frontmatter | `head -10 skills/comms-strategy/SKILL.md \| grep -c 'name: comms-strategy'` |
| PC-007 | lint | Strategy defines all 4 channels | `grep -c 'social\|blog\|devblog\|docs' skills/comms-strategy/SKILL.md` |
| PC-008 | lint | Strategy skill under 500 words | `wc -w < skills/comms-strategy/SKILL.md` |
| PC-009 | lint | comms-adapter skill exists | `head -10 skills/comms-adapter/SKILL.md \| grep -c 'name: comms-adapter'` |
| PC-010 | lint | Adapter documents file-output pattern | `grep -c 'drafts/' skills/comms-adapter/SKILL.md` |
| PC-011 | lint | comms-redactor skill exists | `head -10 skills/comms-redactor/SKILL.md \| grep -c 'name: comms-redactor'` |
| PC-012 | lint | Redactor covers API key patterns | `grep -c 'sk-\|ghp_\|AKIA' skills/comms-redactor/SKILL.md` |
| PC-013 | lint | Redactor is fail-safe | `grep -c 'BLOCK\|fail-safe\|halt' skills/comms-redactor/SKILL.md` |
| PC-014 | lint | .gitignore has comms/ entry | `grep -c '^comms/' .gitignore` |
| PC-015 | build | ecc validate agents passes | `ecc validate agents` |
