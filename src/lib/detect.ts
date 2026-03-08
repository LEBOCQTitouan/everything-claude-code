/**
 * Detection engine: scan existing Claude Code setup at global and project level.
 * Pure read-only scanning — no side effects.
 */

import fs from 'fs';
import path from 'path';

export interface DetectedAgent {
  filename: string;
  name: string | null; // extracted from frontmatter
}

export interface DetectedSkill {
  dirname: string;
  hasSkillMd: boolean;
}

export interface DetectedHook {
  event: string;
  description: string;
  matcher: string;
}

export interface DetectionResult {
  agents: DetectedAgent[];
  commands: string[];       // filenames
  skills: DetectedSkill[];
  rules: Record<string, string[]>; // subdirectory -> filenames
  hooks: DetectedHook[];
  claudeMdHeadings: string[];      // ## headings found in CLAUDE.md
  hasSettingsJson: boolean;
  hasClaudeMd: boolean;
}

/**
 * Extract `name` from YAML frontmatter in a markdown file.
 * Returns null if no frontmatter or no name field.
 */
function extractFrontmatterName(filePath: string): string | null {
  try {
    const content = fs.readFileSync(filePath, 'utf8');
    if (!content.startsWith('---')) return null;

    const endIdx = content.indexOf('---', 3);
    if (endIdx === -1) return null;

    const frontmatter = content.slice(3, endIdx);
    const nameMatch = frontmatter.match(/^name:\s*(.+)$/m);
    return nameMatch ? nameMatch[1].trim().replace(/^["']|["']$/g, '') : null;
  } catch {
    return null;
  }
}

/**
 * List markdown files in a directory.
 */
function listMdFiles(dir: string): string[] {
  try {
    if (!fs.existsSync(dir)) return [];
    return fs.readdirSync(dir)
      .filter(f => f.endsWith('.md'))
      .sort();
  } catch {
    return [];
  }
}

/**
 * Detect agents in a directory.
 */
export function detectAgents(dir: string): DetectedAgent[] {
  const agentsDir = path.join(dir, 'agents');
  return listMdFiles(agentsDir).map(filename => ({
    filename,
    name: extractFrontmatterName(path.join(agentsDir, filename)),
  }));
}

/**
 * Detect commands in a directory.
 */
export function detectCommands(dir: string): string[] {
  return listMdFiles(path.join(dir, 'commands'));
}

/**
 * Detect skills in a directory.
 */
export function detectSkills(dir: string): DetectedSkill[] {
  const skillsDir = path.join(dir, 'skills');
  try {
    if (!fs.existsSync(skillsDir)) return [];
    return fs.readdirSync(skillsDir, { withFileTypes: true })
      .filter(e => e.isDirectory())
      .map(e => ({
        dirname: e.name,
        hasSkillMd: fs.existsSync(path.join(skillsDir, e.name, 'SKILL.md')),
      }))
      .sort((a, b) => a.dirname.localeCompare(b.dirname));
  } catch {
    return [];
  }
}

/**
 * Detect rules in a directory, grouped by subdirectory.
 */
export function detectRules(dir: string): Record<string, string[]> {
  const rulesDir = path.join(dir, 'rules');
  const result: Record<string, string[]> = {};
  try {
    if (!fs.existsSync(rulesDir)) return result;
    for (const entry of fs.readdirSync(rulesDir, { withFileTypes: true })) {
      if (entry.isDirectory()) {
        const files = listMdFiles(path.join(rulesDir, entry.name));
        if (files.length > 0) {
          result[entry.name] = files;
        }
      }
    }
  } catch {
    // ignore
  }
  return result;
}

/**
 * Detect hooks from a settings.json file.
 */
export function detectHooks(dir: string): DetectedHook[] {
  const settingsPath = path.join(dir, 'settings.json');
  try {
    if (!fs.existsSync(settingsPath)) return [];
    const settings = JSON.parse(fs.readFileSync(settingsPath, 'utf8'));
    const hooks: DetectedHook[] = [];
    if (settings.hooks && typeof settings.hooks === 'object') {
      for (const [event, entries] of Object.entries(settings.hooks)) {
        if (!Array.isArray(entries)) continue;
        for (const entry of entries) {
          hooks.push({
            event,
            description: (entry as Record<string, unknown>).description as string || '',
            matcher: (entry as Record<string, unknown>).matcher as string || '*',
          });
        }
      }
    }
    return hooks;
  } catch {
    return [];
  }
}

/**
 * Detect CLAUDE.md headings in a project directory.
 */
export function detectClaudeMd(projectDir: string): string[] {
  const claudeMdPath = path.join(projectDir, 'CLAUDE.md');
  try {
    if (!fs.existsSync(claudeMdPath)) return [];
    const content = fs.readFileSync(claudeMdPath, 'utf8');
    return content.split('\n')
      .filter(line => line.startsWith('## '))
      .map(line => line.trim());
  } catch {
    return [];
  }
}

/**
 * Run full detection on a directory (global ~/.claude/ or project .claude/).
 */
export function detect(dir: string, projectDir?: string): DetectionResult {
  return {
    agents: detectAgents(dir),
    commands: detectCommands(dir),
    skills: detectSkills(dir),
    rules: detectRules(dir),
    hooks: detectHooks(dir),
    claudeMdHeadings: projectDir ? detectClaudeMd(projectDir) : [],
    hasSettingsJson: fs.existsSync(path.join(dir, 'settings.json')),
    hasClaudeMd: projectDir ? fs.existsSync(path.join(projectDir, 'CLAUDE.md')) : false,
  };
}

/**
 * Generate a human-readable report from detection results.
 */
export function generateReport(result: DetectionResult): string {
  const lines: string[] = ['Existing Claude Code configuration:'];

  if (result.agents.length > 0) {
    lines.push(`  Agents:   ${result.agents.length} found`);
    for (const a of result.agents) {
      lines.push(`    - ${a.filename}${a.name ? ` (${a.name})` : ''}`);
    }
  }

  if (result.commands.length > 0) {
    lines.push(`  Commands: ${result.commands.length} found`);
  }

  if (result.skills.length > 0) {
    lines.push(`  Skills:   ${result.skills.length} found`);
  }

  const ruleGroups = Object.keys(result.rules);
  if (ruleGroups.length > 0) {
    const totalRules = Object.values(result.rules).reduce((sum, arr) => sum + arr.length, 0);
    lines.push(`  Rules:    ${totalRules} across ${ruleGroups.length} group(s) [${ruleGroups.join(', ')}]`);
  }

  if (result.hooks.length > 0) {
    lines.push(`  Hooks:    ${result.hooks.length} found`);
  }

  if (result.hasClaudeMd) {
    lines.push(`  CLAUDE.md: exists (${result.claudeMdHeadings.length} sections)`);
  }

  if (result.hasSettingsJson) {
    lines.push(`  settings.json: exists`);
  }

  if (result.agents.length === 0 && result.commands.length === 0 &&
      result.skills.length === 0 && ruleGroups.length === 0 &&
      result.hooks.length === 0) {
    lines.push('  (no existing configuration found)');
  }

  return lines.join('\n');
}
