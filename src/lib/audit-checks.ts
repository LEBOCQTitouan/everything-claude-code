/**
 * Individual audit check functions for ECC configuration.
 * Each check returns a structured result with a grade and fix suggestions.
 */

import fs from 'fs';
import path from 'path';
import { ECC_DENY_RULES } from './deny-rules';
import { ECC_GITIGNORE_ENTRIES } from './gitignore';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type Severity = 'critical' | 'high' | 'medium' | 'low';

export interface AuditFinding {
  id: string;
  severity: Severity;
  title: string;
  detail: string;
  fix: string;
}

export interface AuditCheckResult {
  name: string;
  passed: boolean;
  findings: AuditFinding[];
}

export interface AuditReport {
  checks: AuditCheckResult[];
  score: number;
  grade: string;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function readJsonSafe(filePath: string): Record<string, unknown> | null {
  try {
    return JSON.parse(fs.readFileSync(filePath, 'utf8'));
  } catch {
    return null;
  }
}

function parseFrontmatter(content: string): Record<string, unknown> {
  const match = content.match(/^---\n([\s\S]*?)\n---/);
  if (!match) return {};
  const result: Record<string, unknown> = {};
  for (const line of match[1].split('\n')) {
    const colonIdx = line.indexOf(':');
    if (colonIdx > 0) {
      const key = line.slice(0, colonIdx).trim();
      const value = line.slice(colonIdx + 1).trim();
      result[key] = value;
    }
  }
  return result;
}

// ---------------------------------------------------------------------------
// Check: Deny rules present
// ---------------------------------------------------------------------------

export function checkDenyRules(settingsJsonPath: string): AuditCheckResult {
  const findings: AuditFinding[] = [];
  const settings = readJsonSafe(settingsJsonPath);

  if (!settings) {
    findings.push({
      id: 'DENY-001',
      severity: 'critical',
      title: 'No settings.json found',
      detail: `Expected settings at ${settingsJsonPath}`,
      fix: 'Run `ecc install` to create settings with deny rules.'
    });
    return { name: 'Deny rules', passed: false, findings };
  }

  const permissions = settings.permissions as Record<string, unknown> | undefined;
  const deny = (permissions?.deny as string[]) || [];
  const denySet = new Set(deny);

  const missing = ECC_DENY_RULES.filter(rule => !denySet.has(rule));
  if (missing.length > 0) {
    findings.push({
      id: 'DENY-002',
      severity: 'critical',
      title: `${missing.length} deny rule(s) missing`,
      detail: `Missing: ${missing.slice(0, 3).join(', ')}${missing.length > 3 ? ` (and ${missing.length - 3} more)` : ''}`,
      fix: 'Run `ecc install` to add deny rules, or add them manually to ~/.claude/settings.json.'
    });
  }

  return { name: 'Deny rules', passed: findings.length === 0, findings };
}

// ---------------------------------------------------------------------------
// Check: .gitignore excludes local configs
// ---------------------------------------------------------------------------

export function checkGitignore(projectDir: string): AuditCheckResult {
  const findings: AuditFinding[] = [];
  const gitignorePath = path.join(projectDir, '.gitignore');

  if (!fs.existsSync(gitignorePath)) {
    findings.push({
      id: 'GIT-001',
      severity: 'medium',
      title: 'No .gitignore file found',
      detail: 'Project has no .gitignore — local configs may be committed accidentally.',
      fix: 'Run `ecc init` to create .gitignore with ECC entries.'
    });
    return { name: 'Gitignore', passed: false, findings };
  }

  const content = fs.readFileSync(gitignorePath, 'utf8');
  const patterns = new Set(
    content
      .split('\n')
      .map(l => l.trim())
      .filter(l => l.length > 0 && !l.startsWith('#'))
  );

  const missing = ECC_GITIGNORE_ENTRIES.filter(e => !patterns.has(e.pattern));
  if (missing.length > 0) {
    findings.push({
      id: 'GIT-002',
      severity: 'high',
      title: `${missing.length} gitignore entry/ies missing`,
      detail: `Missing: ${missing.map(e => e.pattern).join(', ')}`,
      fix: 'Run `ecc init` to add missing entries.'
    });
  }

  return { name: 'Gitignore', passed: findings.length === 0, findings };
}

// ---------------------------------------------------------------------------
// Check: Hooks have no duplicates
// ---------------------------------------------------------------------------

export function checkHookDuplicates(settingsJsonPath: string): AuditCheckResult {
  const findings: AuditFinding[] = [];
  const settings = readJsonSafe(settingsJsonPath);

  if (!settings || !settings.hooks) {
    return { name: 'Hook duplicates', passed: true, findings };
  }

  const hooks = settings.hooks as Record<string, Array<Record<string, unknown>>>;
  let totalDuplicates = 0;

  for (const [_event, entries] of Object.entries(hooks)) {
    if (!Array.isArray(entries)) continue;
    const seen = new Set<string>();
    for (const entry of entries) {
      const key = JSON.stringify(entry.hooks);
      if (seen.has(key)) {
        totalDuplicates++;
      }
      seen.add(key);
    }
  }

  if (totalDuplicates > 0) {
    findings.push({
      id: 'HOOK-001',
      severity: 'high',
      title: `${totalDuplicates} duplicate hook(s) found`,
      detail: 'Duplicate hooks fire multiple times per event, wasting resources.',
      fix: 'Run `ecc install` to replace hooks section with the clean source.'
    });
  }

  return { name: 'Hook duplicates', passed: findings.length === 0, findings };
}

// ---------------------------------------------------------------------------
// Check: Global CLAUDE.md exists
// ---------------------------------------------------------------------------

export function checkGlobalClaudeMd(claudeDir: string): AuditCheckResult {
  const findings: AuditFinding[] = [];
  const claudeMdPath = path.join(claudeDir, 'CLAUDE.md');

  if (!fs.existsSync(claudeMdPath)) {
    findings.push({
      id: 'CMD-001',
      severity: 'medium',
      title: 'No global ~/.claude/CLAUDE.md',
      detail: 'Critical cross-project instructions only load when rules match file paths.',
      fix: 'Create ~/.claude/CLAUDE.md with a 50-80 line summary of key rules.'
    });
  }

  return { name: 'Global CLAUDE.md', passed: findings.length === 0, findings };
}

// ---------------------------------------------------------------------------
// Check: Agents use skills preloading
// ---------------------------------------------------------------------------

export function checkAgentSkills(agentsDir: string): AuditCheckResult {
  const findings: AuditFinding[] = [];

  if (!fs.existsSync(agentsDir)) {
    return { name: 'Agent skills', passed: true, findings };
  }

  const agents = fs.readdirSync(agentsDir).filter(f => f.endsWith('.md'));
  let withSkills = 0;
  let withoutSkills = 0;

  for (const agent of agents) {
    const content = fs.readFileSync(path.join(agentsDir, agent), 'utf8');
    const fm = parseFrontmatter(content);
    if (fm.skills) {
      withSkills++;
    } else {
      withoutSkills++;
    }
  }

  if (withoutSkills > 0 && agents.length > 5) {
    const ratio = Math.round((withSkills / agents.length) * 100);
    if (ratio < 50) {
      findings.push({
        id: 'AGT-001',
        severity: 'low',
        title: `Only ${withSkills}/${agents.length} agents use skills: preloading`,
        detail: 'Agents without skills: must discover skills at runtime — slower and less reliable.',
        fix: 'Add skills: frontmatter to agents that reference specific skills.'
      });
    }
  }

  return { name: 'Agent skills', passed: findings.length === 0, findings };
}

// ---------------------------------------------------------------------------
// Check: Commands have description frontmatter
// ---------------------------------------------------------------------------

export function checkCommandDescriptions(commandsDir: string): AuditCheckResult {
  const findings: AuditFinding[] = [];

  if (!fs.existsSync(commandsDir)) {
    return { name: 'Command descriptions', passed: true, findings };
  }

  const commands = fs.readdirSync(commandsDir).filter(f => f.endsWith('.md') && !f.startsWith('_'));
  const missing: string[] = [];

  for (const cmd of commands) {
    const content = fs.readFileSync(path.join(commandsDir, cmd), 'utf8');
    const fm = parseFrontmatter(content);
    if (!fm.description) {
      missing.push(cmd);
    }
  }

  if (missing.length > 0) {
    findings.push({
      id: 'CMD-002',
      severity: 'low',
      title: `${missing.length} command(s) missing description frontmatter`,
      detail: `Missing: ${missing.join(', ')}`,
      fix: 'Add description: field to YAML frontmatter in each command file.'
    });
  }

  return { name: 'Command descriptions', passed: findings.length === 0, findings };
}

// ---------------------------------------------------------------------------
// Check: CLAUDE.md size and stale references
// ---------------------------------------------------------------------------

export function checkProjectClaudeMd(projectDir: string): AuditCheckResult {
  const findings: AuditFinding[] = [];
  const claudeMdPath = path.join(projectDir, 'CLAUDE.md');

  if (!fs.existsSync(claudeMdPath)) {
    return { name: 'Project CLAUDE.md', passed: true, findings };
  }

  const content = fs.readFileSync(claudeMdPath, 'utf8');
  const lines = content.split('\n').length;

  if (lines > 200) {
    findings.push({
      id: 'PCM-001',
      severity: 'medium',
      title: `CLAUDE.md is ${lines} lines (recommended < 200)`,
      detail: 'Large CLAUDE.md files consume context budget on every conversation.',
      fix: 'Move detailed instructions to rules/ or skills/ and keep CLAUDE.md lean.'
    });
  }

  return { name: 'Project CLAUDE.md', passed: findings.length === 0, findings };
}

// ---------------------------------------------------------------------------
// Run all checks
// ---------------------------------------------------------------------------

export function runAllChecks(options: { claudeDir: string; projectDir: string; eccRoot: string }): AuditReport {
  const { claudeDir, projectDir, eccRoot } = options;
  const settingsPath = path.join(claudeDir, 'settings.json');
  const agentsDir = path.join(eccRoot, 'agents');
  const commandsDir = path.join(eccRoot, 'commands');

  const checks = [
    checkDenyRules(settingsPath),
    checkGitignore(projectDir),
    checkHookDuplicates(settingsPath),
    checkGlobalClaudeMd(claudeDir),
    checkAgentSkills(agentsDir),
    checkCommandDescriptions(commandsDir),
    checkProjectClaudeMd(projectDir)
  ];

  const criticalFindings = checks.flatMap(c => c.findings).filter(f => f.severity === 'critical').length;
  const highFindings = checks.flatMap(c => c.findings).filter(f => f.severity === 'high').length;

  // Score: start at 100, deduct for findings
  let score = 100;
  score -= criticalFindings * 20;
  score -= highFindings * 10;
  score -= checks.flatMap(c => c.findings).filter(f => f.severity === 'medium').length * 5;
  score -= checks.flatMap(c => c.findings).filter(f => f.severity === 'low').length * 2;
  score = Math.max(0, Math.min(100, score));

  const grade = score >= 90 ? 'A' : score >= 80 ? 'B' : score >= 70 ? 'C' : score >= 60 ? 'D' : 'F';

  return { checks, score, grade };
}
