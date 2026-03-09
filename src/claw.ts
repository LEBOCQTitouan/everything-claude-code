#!/usr/bin/env node
/**
 * NanoClaw v2 — Barebones Agent REPL for Everything Claude Code
 *
 * Zero external dependencies. Session-aware REPL around `claude -p`.
 * Uses spawnSync with argument arrays (not shell strings) for safety.
 */

'use strict';

import fs from 'fs';
import path from 'path';
import os from 'os';
import { spawnSync } from 'child_process';
import readline from 'readline';

const SESSION_NAME_RE = /^[a-zA-Z0-9][-a-zA-Z0-9]*$/;
const DEFAULT_MODEL = process.env.CLAW_MODEL || 'sonnet';
const DEFAULT_COMPACT_KEEP_TURNS = 20;

/**
 * Validate a session name against the allowed format (alphanumeric and hyphens, starting with alphanumeric).
 * @param name - The session name to validate
 * @returns Whether the name matches the allowed pattern
 */
export function isValidSessionName(name: string): boolean {
  return typeof name === 'string' && name.length > 0 && SESSION_NAME_RE.test(name);
}

/**
 * Get the path to the NanoClaw sessions directory (~/.claude/claw/).
 * @returns Absolute path to the claw directory
 */
export function getClawDir(): string {
  return path.join(os.homedir(), '.claude', 'claw');
}

/**
 * Get the file path for a named NanoClaw session.
 * @param name - Session name (alphanumeric with hyphens)
 * @returns Absolute path to the session markdown file
 */
export function getSessionPath(name: string): string {
  return path.join(getClawDir(), `${name}.md`);
}

/**
 * List all saved NanoClaw session names by scanning for .md files.
 * @param dir - Optional directory to scan (defaults to ~/.claude/claw/)
 * @returns Array of session names (without .md extension)
 */
export function listSessions(dir?: string): string[] {
  const clawDir = dir || getClawDir();
  if (!fs.existsSync(clawDir)) return [];
  return fs
    .readdirSync(clawDir)
    .filter(f => f.endsWith('.md'))
    .map(f => f.replace(/\.md$/, ''));
}

/**
 * Load conversation history from a session file.
 * @param filePath - Absolute path to the session markdown file
 * @returns File contents as a string, or empty string if file does not exist
 */
export function loadHistory(filePath: string): string {
  try {
    return fs.readFileSync(filePath, 'utf8');
  } catch {
    return '';
  }
}

/**
 * Append a conversation turn to a session file in NanoClaw markdown format.
 * @param filePath - Absolute path to the session file
 * @param role - Speaker role (e.g., "User" or "Assistant")
 * @param content - Message content
 * @param timestamp - Optional ISO timestamp (defaults to current time)
 */
export function appendTurn(filePath: string, role: string, content: string, timestamp?: string): void {
  const ts = timestamp || new Date().toISOString();
  const entry = `### [${ts}] ${role}\n${content}\n---\n`;
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.appendFileSync(filePath, entry, 'utf8');
}

function normalizeSkillList(raw: string | string[]): string[] {
  if (!raw) return [];
  if (Array.isArray(raw)) return raw.map(s => String(s).trim()).filter(Boolean);
  return String(raw)
    .split(',')
    .map(s => s.trim())
    .filter(Boolean);
}

/**
 * Load ECC skill files as context strings for the REPL prompt.
 * Reads SKILL.md from each requested skill directory under cwd/skills/.
 * @param skillList - Comma-separated string or array of skill names (defaults to CLAW_SKILLS env var)
 * @returns Concatenated skill content, or empty string if no skills found
 */
export function loadECCContext(skillList?: string | string[]): string {
  const requested = normalizeSkillList(skillList !== undefined ? skillList : process.env.CLAW_SKILLS || '');
  if (requested.length === 0) return '';

  const chunks: string[] = [];
  for (const name of requested) {
    const skillPath = path.join(process.cwd(), 'skills', name, 'SKILL.md');
    try {
      chunks.push(fs.readFileSync(skillPath, 'utf8'));
    } catch {
      // Skip missing skills silently
    }
  }

  return chunks.join('\n\n');
}

/**
 * Build a full prompt string combining system context, conversation history, and user message.
 * @param systemPrompt - System-level context (e.g., loaded skills)
 * @param history - Previous conversation turns
 * @param userMessage - Current user input
 * @returns Formatted prompt with labeled sections
 */
export function buildPrompt(systemPrompt: string, history: string, userMessage: string): string {
  const parts: string[] = [];
  if (systemPrompt) parts.push(`=== SYSTEM CONTEXT ===\n${systemPrompt}\n`);
  if (history) parts.push(`=== CONVERSATION HISTORY ===\n${history}\n`);
  parts.push(`=== USER MESSAGE ===\n${userMessage}`);
  return parts.join('\n');
}

/**
 * Call claude CLI with argument array (safe — no shell interpolation).
 */
export function askClaude(systemPrompt: string, history: string, userMessage: string, model?: string): string {
  const fullPrompt = buildPrompt(systemPrompt, history, userMessage);
  const args: string[] = [];
  if (model) {
    args.push('--model', model);
  }
  args.push('-p', fullPrompt);

  const result = spawnSync('claude', args, {
    encoding: 'utf8',
    stdio: ['pipe', 'pipe', 'pipe'],
    env: { ...process.env, CLAUDECODE: '' },
    timeout: 300000
  });

  if (result.error) {
    return `[Error: ${result.error.message}]`;
  }

  if (result.status !== 0 && result.stderr) {
    return `[Error: claude exited with code ${result.status}: ${result.stderr.trim()}]`;
  }

  return (result.stdout || '').trim();
}

interface Turn {
  timestamp: string;
  role: string;
  content: string;
}

/**
 * Parse a session history string into structured Turn objects.
 * Extracts timestamp, role, and content from NanoClaw markdown format.
 * @param history - Raw markdown history content
 * @returns Array of parsed turns
 */
export function parseTurns(history: string): Turn[] {
  const turns: Turn[] = [];
  const regex = /### \[([^\]]+)\] ([^\n]+)\n([\s\S]*?)\n---\n/g;
  let match: RegExpExecArray | null;
  while ((match = regex.exec(history)) !== null) {
    turns.push({ timestamp: match[1], role: match[2], content: match[3] });
  }
  return turns;
}

/**
 * Estimate token count from text length using a 4-characters-per-token heuristic.
 * @param text - Input text to estimate
 * @returns Estimated token count (ceiling)
 */
export function estimateTokenCount(text: string): number {
  return Math.ceil((text || '').length / 4);
}

/**
 * Compute usage metrics for a NanoClaw session (turn counts, character count, token estimate).
 * @param filePath - Absolute path to the session file
 * @returns Object with total, user, and assistant turn counts, character count, and token estimate
 */
export function getSessionMetrics(filePath: string): {
  turns: number;
  userTurns: number;
  assistantTurns: number;
  charCount: number;
  tokenEstimate: number;
} {
  const history = loadHistory(filePath);
  const turns = parseTurns(history);
  return {
    turns: turns.length,
    userTurns: turns.filter(t => t.role === 'User').length,
    assistantTurns: turns.filter(t => t.role === 'Assistant').length,
    charCount: history.length,
    tokenEstimate: estimateTokenCount(history)
  };
}

/**
 * Search across all saved sessions for a keyword, returning matching session names and context snippets.
 * @param query - Search term (case-insensitive)
 * @param dir - Optional directory to search (defaults to ~/.claude/claw/)
 * @returns Array of matches with session name and surrounding text snippet
 */
export function searchSessions(query: string, dir?: string): Array<{ session: string; snippet: string }> {
  const q = String(query || '')
    .toLowerCase()
    .trim();
  if (!q) return [];

  const sessionDir = dir || getClawDir();
  const sessions = listSessions(sessionDir);
  const results: Array<{ session: string; snippet: string }> = [];
  for (const name of sessions) {
    const p = path.join(sessionDir, `${name}.md`);
    const content = loadHistory(p);
    if (!content) continue;

    const idx = content.toLowerCase().indexOf(q);
    if (idx >= 0) {
      const start = Math.max(0, idx - 40);
      const end = Math.min(content.length, idx + q.length + 40);
      const snippet = content.slice(start, end).replace(/\n/g, ' ');
      results.push({ session: name, snippet });
    }
  }
  return results;
}

/**
 * Compact a session file by retaining only the most recent turns and discarding older ones.
 * Writes a compaction header with metadata. No-op if the session has fewer turns than the threshold.
 * @param filePath - Absolute path to the session file
 * @param keepTurns - Number of recent turns to retain (default: 20)
 * @returns Whether compaction was performed
 */
export function compactSession(filePath: string, keepTurns = DEFAULT_COMPACT_KEEP_TURNS): boolean {
  const history = loadHistory(filePath);
  if (!history) return false;

  const turns = parseTurns(history);
  if (turns.length <= keepTurns) return false;

  const retained = turns.slice(-keepTurns);
  const compactedHeader = `# NanoClaw Compaction\nCompacted at: ${new Date().toISOString()}\nRetained turns: ${keepTurns}/${turns.length}\n\n---\n`;
  const compactedTurns = retained.map(t => `### [${t.timestamp}] ${t.role}\n${t.content}\n---\n`).join('');
  fs.writeFileSync(filePath, compactedHeader + compactedTurns, 'utf8');
  return true;
}

/**
 * Export a session to a file in the specified format (md, json, or txt).
 * @param filePath - Absolute path to the source session file
 * @param format - Export format: "md" | "markdown" | "json" | "txt" | "text"
 * @param outputPath - Optional output file path (auto-generated if omitted)
 * @returns Result object with success flag, output path, or error message
 */
export function exportSession(filePath: string, format: string, outputPath?: string): { ok: boolean; message?: string; path?: string } {
  const history = loadHistory(filePath);
  const sessionName = path.basename(filePath, '.md');
  const fmt = String(format || 'md').toLowerCase();

  if (!history) {
    return { ok: false, message: 'No session history to export.' };
  }

  const dir = path.dirname(filePath);
  let out = outputPath;
  if (!out) {
    out = path.join(dir, `${sessionName}.export.${fmt === 'markdown' ? 'md' : fmt}`);
  }

  if (fmt === 'md' || fmt === 'markdown') {
    fs.writeFileSync(out, history, 'utf8');
    return { ok: true, path: out };
  }

  if (fmt === 'json') {
    const turns = parseTurns(history);
    fs.writeFileSync(out, JSON.stringify({ session: sessionName, turns }, null, 2), 'utf8');
    return { ok: true, path: out };
  }

  if (fmt === 'txt' || fmt === 'text') {
    const turns = parseTurns(history);
    const txt = turns.map(t => `[${t.timestamp}] ${t.role}:\n${t.content}\n`).join('\n');
    fs.writeFileSync(out, txt, 'utf8');
    return { ok: true, path: out };
  }

  return { ok: false, message: `Unsupported export format: ${format}` };
}

/**
 * Branch (duplicate) the current session into a new named session file.
 * @param currentSessionPath - Path to the source session file
 * @param newSessionName - Name for the branched session
 * @param targetDir - Directory for the new session file (defaults to ~/.claude/claw/)
 * @returns Result object with success flag, new session path and name, or error message
 */
export function branchSession(currentSessionPath: string, newSessionName: string, targetDir = getClawDir()): { ok: boolean; message?: string; path?: string; session?: string } {
  if (!isValidSessionName(newSessionName)) {
    return { ok: false, message: `Invalid branch session name: ${newSessionName}` };
  }

  const target = path.join(targetDir, `${newSessionName}.md`);
  fs.mkdirSync(path.dirname(target), { recursive: true });

  const content = loadHistory(currentSessionPath);
  fs.writeFileSync(target, content, 'utf8');
  return { ok: true, path: target, session: newSessionName };
}

function skillExists(skillName: string): boolean {
  const p = path.join(process.cwd(), 'skills', skillName, 'SKILL.md');
  return fs.existsSync(p);
}

/**
 * Clear all history from the current session file.
 * @param sessionPath - Absolute path to the session file to clear
 */
export function handleClear(sessionPath: string): void {
  fs.mkdirSync(path.dirname(sessionPath), { recursive: true });
  fs.writeFileSync(sessionPath, '', 'utf8');
  console.log('Session cleared.');
}

/**
 * Print the full conversation history of a session to stdout.
 * @param sessionPath - Absolute path to the session file
 */
export function handleHistory(sessionPath: string): void {
  const history = loadHistory(sessionPath);
  if (!history) {
    console.log('(no history)');
    return;
  }
  console.log(history);
}

/**
 * Print all saved session names to stdout.
 * @param dir - Optional directory to scan (defaults to ~/.claude/claw/)
 */
export function handleSessions(dir?: string): void {
  const sessions = listSessions(dir);
  if (sessions.length === 0) {
    console.log('(no sessions)');
    return;
  }
  console.log('Sessions:');
  for (const s of sessions) {
    console.log(`  - ${s}`);
  }
}

/**
 * Print the NanoClaw REPL help text listing all available commands.
 */
export function handleHelp(): void {
  console.log('NanoClaw REPL Commands:');
  console.log('  /help                          Show this help');
  console.log('  /clear                         Clear current session history');
  console.log('  /history                       Print full conversation history');
  console.log('  /sessions                      List saved sessions');
  console.log('  /model [name]                  Show/set model');
  console.log('  /load <skill-name>             Load a skill into active context');
  console.log('  /branch <session-name>         Branch current session into a new session');
  console.log('  /search <query>                Search query across sessions');
  console.log('  /compact                       Keep recent turns, compact older context');
  console.log('  /export <md|json|txt> [path]   Export current session');
  console.log('  /metrics                       Show session metrics');
  console.log('  exit                           Quit the REPL');
}

export function main(): void {
  const initialSessionName = process.env.CLAW_SESSION || 'default';
  if (!isValidSessionName(initialSessionName)) {
    console.error(`Error: Invalid session name "${initialSessionName}". Use alphanumeric characters and hyphens only.`);
    process.exit(1);
  }

  fs.mkdirSync(getClawDir(), { recursive: true });

  const state = {
    sessionName: initialSessionName,
    sessionPath: getSessionPath(initialSessionName),
    model: DEFAULT_MODEL,
    skills: normalizeSkillList(process.env.CLAW_SKILLS || '')
  };

  let eccContext = loadECCContext(state.skills);
  const loadedCount = state.skills.filter(skillExists).length;

  console.log(`NanoClaw v2 \u2014 Session: ${state.sessionName}`);
  console.log(`Model: ${state.model}`);
  if (loadedCount > 0) {
    console.log(`Loaded ${loadedCount} skill(s) as context.`);
  }
  console.log('Type /help for commands, exit to quit.\n');

  const rl = readline.createInterface({ input: process.stdin, output: process.stdout });

  const prompt = (): void => {
    rl.question('claw> ', input => {
      const line = input.trim();
      if (!line) return prompt();
      if (line === 'exit') {
        console.log('Goodbye.');
        rl.close();
        return;
      }
      if (line === '/help') {
        handleHelp();
        return prompt();
      }
      if (line === '/clear') {
        handleClear(state.sessionPath);
        return prompt();
      }
      if (line === '/history') {
        handleHistory(state.sessionPath);
        return prompt();
      }
      if (line === '/sessions') {
        handleSessions();
        return prompt();
      }

      if (line.startsWith('/model')) {
        const model = line.replace('/model', '').trim();
        if (!model) {
          console.log(`Current model: ${state.model}`);
        } else {
          state.model = model;
          console.log(`Model set to: ${state.model}`);
        }
        return prompt();
      }

      if (line.startsWith('/load ')) {
        const skill = line.replace('/load', '').trim();
        if (!skill) {
          console.log('Usage: /load <skill-name>');
          return prompt();
        }
        if (!skillExists(skill)) {
          console.log(`Skill not found: ${skill}`);
          return prompt();
        }
        if (!state.skills.includes(skill)) {
          state.skills.push(skill);
        }
        eccContext = loadECCContext(state.skills);
        console.log(`Loaded skill: ${skill}`);
        return prompt();
      }

      if (line.startsWith('/branch ')) {
        const target = line.replace('/branch', '').trim();
        const result = branchSession(state.sessionPath, target);
        if (!result.ok) {
          console.log(result.message);
          return prompt();
        }
        state.sessionName = result.session!;
        state.sessionPath = result.path!;
        console.log(`Branched to session: ${state.sessionName}`);
        return prompt();
      }

      if (line.startsWith('/search ')) {
        const query = line.replace('/search', '').trim();
        const matches = searchSessions(query);
        if (matches.length === 0) {
          console.log('(no matches)');
          return prompt();
        }
        console.log(`Found ${matches.length} match(es):`);
        for (const m of matches) {
          console.log(`- ${m.session}: ${m.snippet}`);
        }
        return prompt();
      }

      if (line === '/compact') {
        const changed = compactSession(state.sessionPath);
        console.log(changed ? 'Session compacted.' : 'No compaction needed.');
        return prompt();
      }

      if (line.startsWith('/export ')) {
        const parts = line.split(/\s+/).filter(Boolean);
        const format = parts[1];
        const outputPathArg = parts[2];
        if (!format) {
          console.log('Usage: /export <md|json|txt> [path]');
          return prompt();
        }
        const result = exportSession(state.sessionPath, format, outputPathArg);
        if (!result.ok) {
          console.log(result.message);
        } else {
          console.log(`Exported: ${result.path}`);
        }
        return prompt();
      }

      if (line === '/metrics') {
        const m = getSessionMetrics(state.sessionPath);
        console.log(`Session: ${state.sessionName}`);
        console.log(`Model: ${state.model}`);
        console.log(`Turns: ${m.turns} (user ${m.userTurns}, assistant ${m.assistantTurns})`);
        console.log(`Chars: ${m.charCount}`);
        console.log(`Estimated tokens: ${m.tokenEstimate}`);
        return prompt();
      }

      // Regular message
      const history = loadHistory(state.sessionPath);
      appendTurn(state.sessionPath, 'User', line);
      const response = askClaude(eccContext, history, line, state.model);
      console.log(`\n${response}\n`);
      appendTurn(state.sessionPath, 'Assistant', response);
      prompt();
    });
  };

  prompt();
}

if (require.main === module) {
  main();
}
