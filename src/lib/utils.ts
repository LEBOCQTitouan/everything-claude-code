/**
 * Cross-platform utility functions for Claude Code hooks and scripts
 * Works on Windows, macOS, and Linux
 */

import fs from 'fs';
import path from 'path';
import os from 'os';
import { execSync, spawnSync, type ExecSyncOptions } from 'child_process';

// Platform detection
export const isWindows = process.platform === 'win32';
export const isMacOS = process.platform === 'darwin';
export const isLinux = process.platform === 'linux';

/**
 * Get the user's home directory (cross-platform)
 */
export function getHomeDir(): string {
  return os.homedir();
}

/**
 * Get the Claude config directory
 */
export function getClaudeDir(): string {
  return path.join(getHomeDir(), '.claude');
}

/**
 * Get the sessions directory
 */
export function getSessionsDir(): string {
  return path.join(getClaudeDir(), 'sessions');
}

/**
 * Get the learned skills directory
 */
export function getLearnedSkillsDir(): string {
  return path.join(getClaudeDir(), 'skills', 'learned');
}

/**
 * Get the temp directory (cross-platform)
 */
export function getTempDir(): string {
  return os.tmpdir();
}

/**
 * Ensure a directory exists (create if not)
 * @throws If directory cannot be created (e.g., permission denied)
 */
export function ensureDir(dirPath: string): string {
  try {
    if (!fs.existsSync(dirPath)) {
      fs.mkdirSync(dirPath, { recursive: true });
    }
  } catch (err: unknown) {
    // EEXIST is fine (race condition with another process creating it)
    if ((err as NodeJS.ErrnoException).code !== 'EEXIST') {
      throw new Error(`Failed to create directory '${dirPath}': ${(err as Error).message}`);
    }
  }
  return dirPath;
}

/**
 * Get current date in YYYY-MM-DD format
 */
export function getDateString(): string {
  const now = new Date();
  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, '0');
  const day = String(now.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

/**
 * Get current time in HH:MM format
 */
export function getTimeString(): string {
  const now = new Date();
  const hours = String(now.getHours()).padStart(2, '0');
  const minutes = String(now.getMinutes()).padStart(2, '0');
  return `${hours}:${minutes}`;
}

/**
 * Get git repository name
 */
export function getGitRepoName(): string | null {
  const result = runCommand('git rev-parse --show-toplevel');
  if (!result.success) return null;
  return path.basename(result.output);
}

/**
 * Get project name from git repo or current directory
 */
export function getProjectName(): string | null {
  const repoName = getGitRepoName();
  if (repoName) return repoName;
  return path.basename(process.cwd()) || null;
}

/**
 * Get short session ID from CLAUDE_SESSION_ID environment variable
 * Returns last 8 characters, falls back to project name then 'default'
 */
export function getSessionIdShort(fallback = 'default'): string {
  const sessionId = process.env.CLAUDE_SESSION_ID;
  if (sessionId && sessionId.length > 0) {
    return sessionId.slice(-8);
  }
  return getProjectName() || fallback;
}

/**
 * Get current datetime in YYYY-MM-DD HH:MM:SS format
 */
export function getDateTimeString(): string {
  const now = new Date();
  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, '0');
  const day = String(now.getDate()).padStart(2, '0');
  const hours = String(now.getHours()).padStart(2, '0');
  const minutes = String(now.getMinutes()).padStart(2, '0');
  const seconds = String(now.getSeconds()).padStart(2, '0');
  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
}

export interface FileMatch {
  path: string;
  mtime: number;
}

export interface FindFilesOptions {
  maxAge?: number | null;
  recursive?: boolean;
}

/**
 * Find files matching a pattern in a directory (cross-platform alternative to find)
 */
export function findFiles(dir: string, pattern: string, options: FindFilesOptions = {}): FileMatch[] {
  if (!dir || typeof dir !== 'string') return [];
  if (!pattern || typeof pattern !== 'string') return [];

  const { maxAge = null, recursive = false } = options;
  const results: FileMatch[] = [];

  if (!fs.existsSync(dir)) {
    return results;
  }

  // Escape all regex special characters, then convert glob wildcards.
  const regexPattern = pattern
    .replace(/[.+^${}()|[\]\\]/g, '\\$&')
    .replace(/\*/g, '.*')
    .replace(/\?/g, '.');
  const regex = new RegExp(`^${regexPattern}$`);

  function searchDir(currentDir: string): void {
    try {
      const entries = fs.readdirSync(currentDir, { withFileTypes: true });

      for (const entry of entries) {
        const fullPath = path.join(currentDir, entry.name);

        if (entry.isFile() && regex.test(entry.name)) {
          let stats: fs.Stats;
          try {
            stats = fs.statSync(fullPath);
          } catch {
            continue; // File deleted between readdir and stat
          }

          if (maxAge !== null) {
            const ageInDays = (Date.now() - stats.mtimeMs) / (1000 * 60 * 60 * 24);
            if (ageInDays <= maxAge) {
              results.push({ path: fullPath, mtime: stats.mtimeMs });
            }
          } else {
            results.push({ path: fullPath, mtime: stats.mtimeMs });
          }
        } else if (entry.isDirectory() && recursive) {
          searchDir(fullPath);
        }
      }
    } catch {
      // Ignore permission errors
    }
  }

  searchDir(dir);

  // Sort by modification time (newest first)
  results.sort((a, b) => b.mtime - a.mtime);

  return results;
}

export interface ReadStdinJsonOptions {
  timeoutMs?: number;
  maxSize?: number;
}

/**
 * Read JSON from stdin (for hook input)
 */
export async function readStdinJson(options: ReadStdinJsonOptions = {}): Promise<Record<string, unknown>> {
  const { timeoutMs = 5000, maxSize = 1024 * 1024 } = options;

  return new Promise((resolve) => {
    let data = '';
    let settled = false;

    const timer = setTimeout(() => {
      if (!settled) {
        settled = true;
        process.stdin.removeAllListeners('data');
        process.stdin.removeAllListeners('end');
        process.stdin.removeAllListeners('error');
        if (process.stdin.unref) process.stdin.unref();
        try {
          resolve(data.trim() ? JSON.parse(data) : {});
        } catch {
          resolve({});
        }
      }
    }, timeoutMs);

    process.stdin.setEncoding('utf8');
    process.stdin.on('data', (chunk: string) => {
      if (data.length < maxSize) {
        data += chunk;
      }
    });

    process.stdin.on('end', () => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      try {
        resolve(data.trim() ? JSON.parse(data) : {});
      } catch {
        resolve({});
      }
    });

    process.stdin.on('error', () => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      resolve({});
    });
  });
}

/**
 * Log to stderr (visible to user in Claude Code)
 */
export function log(message: string): void {
  console.error(message);
}

/**
 * Output to stdout (returned to Claude)
 */
export function output(data: string | Record<string, unknown>): void {
  if (typeof data === 'object') {
    console.log(JSON.stringify(data));
  } else {
    console.log(data);
  }
}

/**
 * Read a text file safely
 */
export function readFile(filePath: string): string | null {
  try {
    return fs.readFileSync(filePath, 'utf8');
  } catch {
    return null;
  }
}

/**
 * Write a text file
 */
export function writeFile(filePath: string, content: string): void {
  ensureDir(path.dirname(filePath));
  fs.writeFileSync(filePath, content, 'utf8');
}

/**
 * Append to a text file
 */
export function appendFile(filePath: string, content: string): void {
  ensureDir(path.dirname(filePath));
  fs.appendFileSync(filePath, content, 'utf8');
}

/**
 * Check if a command exists in PATH
 * Uses spawnSync to prevent command injection
 */
export function commandExists(cmd: string): boolean {
  if (!/^[a-zA-Z0-9_.-]+$/.test(cmd)) {
    return false;
  }

  try {
    if (isWindows) {
      const result = spawnSync('where', [cmd], { stdio: 'pipe' });
      return result.status === 0;
    } else {
      const result = spawnSync('which', [cmd], { stdio: 'pipe' });
      return result.status === 0;
    }
  } catch {
    return false;
  }
}

export interface CommandResult {
  success: boolean;
  output: string;
}

/**
 * Run a command and return output
 *
 * SECURITY NOTE: This function uses execSync which invokes a shell.
 * Only use with trusted, hardcoded commands — never pass user-controlled input.
 * For user input, use spawnSync with argument arrays instead.
 */
export function runCommand(cmd: string, options: ExecSyncOptions = {}): CommandResult {
  try {
    const result = execSync(cmd, {
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'pipe'],
      ...options
    });
    return { success: true, output: (result as string).trim() };
  } catch (err: unknown) {
    const e = err as { stderr?: string; message: string };
    return { success: false, output: e.stderr || e.message };
  }
}

/**
 * Check if current directory is a git repository
 */
export function isGitRepo(): boolean {
  return runCommand('git rev-parse --git-dir').success;
}

/**
 * Get git modified files, optionally filtered by regex patterns
 */
export function getGitModifiedFiles(patterns: string[] = []): string[] {
  if (!isGitRepo()) return [];

  const result = runCommand('git diff --name-only HEAD');
  if (!result.success) return [];

  let files = result.output.split('\n').filter(Boolean);

  if (patterns.length > 0) {
    const compiled: RegExp[] = [];
    for (const pattern of patterns) {
      if (typeof pattern !== 'string' || pattern.length === 0) continue;
      try {
        compiled.push(new RegExp(pattern));
      } catch {
        // Skip invalid regex patterns
      }
    }
    if (compiled.length > 0) {
      files = files.filter(file => compiled.some(regex => regex.test(file)));
    }
  }

  return files;
}

export interface ReplaceInFileOptions {
  all?: boolean;
}

/**
 * Replace text in a file (cross-platform sed alternative)
 */
export function replaceInFile(filePath: string, search: string | RegExp, replace: string, options: ReplaceInFileOptions = {}): boolean {
  const content = readFile(filePath);
  if (content === null) return false;

  try {
    let newContent: string;
    if (options.all && typeof search === 'string') {
      newContent = content.replaceAll(search, replace);
    } else {
      newContent = content.replace(search, replace);
    }
    writeFile(filePath, newContent);
    return true;
  } catch (err: unknown) {
    log(`[Utils] replaceInFile failed for ${filePath}: ${(err as Error).message}`);
    return false;
  }
}

/**
 * Count occurrences of a pattern in a file
 */
export function countInFile(filePath: string, pattern: string | RegExp): number {
  const content = readFile(filePath);
  if (content === null) return 0;

  let regex: RegExp;
  try {
    if (pattern instanceof RegExp) {
      regex = new RegExp(pattern.source, pattern.flags.includes('g') ? pattern.flags : pattern.flags + 'g');
    } else if (typeof pattern === 'string') {
      regex = new RegExp(pattern, 'g');
    } else {
      return 0;
    }
  } catch {
    return 0;
  }
  const matches = content.match(regex);
  return matches ? matches.length : 0;
}

export interface GrepMatch {
  lineNumber: number;
  content: string;
}

/**
 * Search for pattern in file and return matching lines with line numbers
 */
export function grepFile(filePath: string, pattern: string | RegExp): GrepMatch[] {
  const content = readFile(filePath);
  if (content === null) return [];

  let regex: RegExp;
  try {
    if (pattern instanceof RegExp) {
      const flags = pattern.flags.replace('g', '');
      regex = new RegExp(pattern.source, flags);
    } else {
      regex = new RegExp(pattern);
    }
  } catch {
    return [];
  }
  const lines = content.split('\n');
  const results: GrepMatch[] = [];

  lines.forEach((line, index) => {
    if (regex.test(line)) {
      results.push({ lineNumber: index + 1, content: line });
    }
  });

  return results;
}
