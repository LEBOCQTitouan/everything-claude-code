/**
 * Claude-powered intelligent file merging and diff display.
 * Includes LCS-based diff algorithm and colored side-by-side formatter.
 */

import { spawnSync } from 'child_process';
import fs from 'fs';
import { red, green, dim, bold, stripAnsi } from './ansi';

/** Result of a Claude-powered smart merge — success flag, merged content, and error. */
export interface SmartMergeResult {
  success: boolean;
  merged: string | null;
  error: string | null;
}

/** A single diff line — type indicates same, removed, or added. */
export interface DiffLine {
  type: 'same' | 'removed' | 'added';
  text: string;
}

// ---------------------------------------------------------------------------
// LCS-based diff algorithm
// ---------------------------------------------------------------------------

/**
 * Compute a diff between two arrays of lines using LCS (Longest Common Subsequence).
 * Returns an array of DiffLine entries indicating same/removed/added.
 */
export function computeLineDiff(existingLines: string[], incomingLines: string[]): DiffLine[] {
  const n = existingLines.length;
  const m = incomingLines.length;

  // Guard against very large files — fall back to simple diff
  if (n * m > 1_000_000) {
    return simpleDiff(existingLines, incomingLines);
  }

  // Build LCS table
  const dp: number[][] = Array.from({ length: n + 1 }, () => new Array(m + 1).fill(0));
  for (let i = 1; i <= n; i++) {
    for (let j = 1; j <= m; j++) {
      if (existingLines[i - 1] === incomingLines[j - 1]) {
        dp[i][j] = dp[i - 1][j - 1] + 1;
      } else {
        dp[i][j] = Math.max(dp[i - 1][j], dp[i][j - 1]);
      }
    }
  }

  // Backtrack to produce diff
  const result: DiffLine[] = [];
  let i = n;
  let j = m;
  while (i > 0 || j > 0) {
    if (i > 0 && j > 0 && existingLines[i - 1] === incomingLines[j - 1]) {
      result.push({ type: 'same', text: existingLines[i - 1] });
      i--;
      j--;
    } else if (j > 0 && (i === 0 || dp[i][j - 1] >= dp[i - 1][j])) {
      result.push({ type: 'added', text: incomingLines[j - 1] });
      j--;
    } else {
      result.push({ type: 'removed', text: existingLines[i - 1] });
      i--;
    }
  }

  result.reverse();
  return result;
}

/**
 * Simple fallback diff for very large files.
 */
function simpleDiff(existingLines: string[], incomingLines: string[]): DiffLine[] {
  const result: DiffLine[] = [];
  const maxLen = Math.max(existingLines.length, incomingLines.length);
  for (let i = 0; i < maxLen; i++) {
    const existing = i < existingLines.length ? existingLines[i] : undefined;
    const incoming = i < incomingLines.length ? incomingLines[i] : undefined;
    if (existing === incoming) {
      result.push({ type: 'same', text: existing! });
    } else {
      if (existing !== undefined) result.push({ type: 'removed', text: existing });
      if (incoming !== undefined) result.push({ type: 'added', text: incoming });
    }
  }
  return result;
}

// ---------------------------------------------------------------------------
// Side-by-side colored formatter
// ---------------------------------------------------------------------------

const MIN_SIDE_BY_SIDE_WIDTH = 60;
const SEPARATOR = ' \u2502 '; // " │ "

/**
 * Truncate a string to fit within a column width, adding "..." if truncated.
 */
function truncate(text: string, maxWidth: number): string {
  if (text.length <= maxWidth) return text;
  if (maxWidth <= 3) return text.slice(0, maxWidth);
  return text.slice(0, maxWidth - 3) + dim('...');
}

/**
 * Pad a string to a fixed width.
 */
function padEnd(text: string, width: number): string {
  const visible = stripAnsi(text).length;
  const padding = Math.max(0, width - visible);
  return text + ' '.repeat(padding);
}

/**
 * Format line number gutter (4 chars, right-aligned, dim).
 */
function gutter(lineNum: number | null): string {
  if (lineNum === null) return dim('    ');
  const s = String(lineNum);
  return dim(s.length < 4 ? ' '.repeat(4 - s.length) + s : s);
}

/**
 * Format diff as colored side-by-side output.
 */
export function formatSideBySideDiff(diffLines: DiffLine[], filename: string): string {
  const termWidth = process.stderr.columns || 80;

  // Fall back to unified colored diff if terminal is too narrow
  if (termWidth < MIN_SIDE_BY_SIDE_WIDTH) {
    return formatUnifiedDiff(diffLines, filename);
  }

  // Column math: gutter(4) + space(1) + content + separator(3) + gutter(4) + space(1) + content
  const separatorWidth = stripAnsi(SEPARATOR).length;
  const gutterWidth = 5; // 4 digits + 1 space
  const contentWidth = Math.floor((termWidth - separatorWidth - gutterWidth * 2) / 2);

  const lines: string[] = [];

  // Header
  const leftHeader = padEnd(bold(`--- existing/${filename}`), contentWidth + gutterWidth);
  const rightHeader = bold(`+++ incoming/${filename}`);
  lines.push(leftHeader + SEPARATOR + rightHeader);
  lines.push(dim('\u2500'.repeat(termWidth)));

  // Track line numbers for each side
  let leftLineNum = 0;
  let rightLineNum = 0;

  // Group adjacent removed+added lines for paired display
  let i = 0;
  while (i < diffLines.length) {
    const line = diffLines[i];

    if (line.type === 'same') {
      leftLineNum++;
      rightLineNum++;
      const text = truncate(line.text, contentWidth);
      const left = gutter(leftLineNum) + ' ' + padEnd(dim(text), contentWidth);
      const right = gutter(rightLineNum) + ' ' + dim(text);
      lines.push(left + SEPARATOR + right);
      i++;
    } else if (line.type === 'removed') {
      // Collect consecutive removed lines
      const removedBlock: string[] = [];
      while (i < diffLines.length && diffLines[i].type === 'removed') {
        removedBlock.push(diffLines[i].text);
        i++;
      }
      // Collect consecutive added lines
      const addedBlock: string[] = [];
      while (i < diffLines.length && diffLines[i].type === 'added') {
        addedBlock.push(diffLines[i].text);
        i++;
      }

      // Pair them side by side
      const maxPairs = Math.max(removedBlock.length, addedBlock.length);
      for (let p = 0; p < maxPairs; p++) {
        const hasLeft = p < removedBlock.length;
        const hasRight = p < addedBlock.length;

        let left: string;
        if (hasLeft) {
          leftLineNum++;
          left = gutter(leftLineNum) + ' ' + padEnd(red(truncate(removedBlock[p], contentWidth)), contentWidth);
        } else {
          left = gutter(null) + ' ' + padEnd('', contentWidth);
        }

        let right: string;
        if (hasRight) {
          rightLineNum++;
          right = gutter(rightLineNum) + ' ' + green(truncate(addedBlock[p], contentWidth));
        } else {
          right = gutter(null) + ' ';
        }

        lines.push(left + SEPARATOR + right);
      }
    } else {
      // Standalone added line (no preceding removed)
      rightLineNum++;
      const left = gutter(null) + ' ' + padEnd('', contentWidth);
      const right = gutter(rightLineNum) + ' ' + green(truncate(line.text, contentWidth));
      lines.push(left + SEPARATOR + right);
      i++;
    }
  }

  lines.push(dim('\u2500'.repeat(termWidth)));
  return lines.join('\n');
}

/**
 * Format diff as colored unified output (fallback for narrow terminals).
 */
function formatUnifiedDiff(diffLines: DiffLine[], filename: string): string {
  const lines: string[] = [bold(`--- existing/${filename}`), bold(`+++ incoming/${filename}`), dim('\u2500'.repeat(40))];

  for (const dl of diffLines) {
    switch (dl.type) {
      case 'same':
        lines.push(dim(` ${dl.text}`));
        break;
      case 'removed':
        lines.push(red(`-${dl.text}`));
        break;
      case 'added':
        lines.push(green(`+${dl.text}`));
        break;
    }
  }

  return lines.join('\n');
}

// ---------------------------------------------------------------------------
// Public API (signature unchanged)
// ---------------------------------------------------------------------------

/**
 * Generate a colored diff between two strings for display.
 * Uses LCS algorithm and side-by-side format when terminal is wide enough.
 */
export function generateDiff(existingContent: string, incomingContent: string, filename: string): string {
  const existingLines = existingContent.split('\n');
  const incomingLines = incomingContent.split('\n');
  const diffLines = computeLineDiff(existingLines, incomingLines);
  return formatSideBySideDiff(diffLines, filename);
}

// ---------------------------------------------------------------------------
// Smart merge (Claude CLI)
// ---------------------------------------------------------------------------

/**
 * Build the merge prompt for Claude.
 */
function buildMergePrompt(existingContent: string, incomingContent: string, filename: string): string {
  return `You are merging two versions of a Claude Code configuration file: "${filename}".

EXISTING VERSION (user's current file — may have customizations):
\`\`\`
${existingContent}
\`\`\`

INCOMING VERSION (latest ECC version — has updates):
\`\`\`
${incomingContent}
\`\`\`

Rules:
1. Keep ALL user customizations (added sections, modified instructions, custom content)
2. Add NEW sections/content from incoming that don't exist in existing
3. Update ECC-standard sections with incoming improvements where the user hasn't customized them
4. If a conflict is ambiguous, keep both versions with a <!-- CONFLICT: review needed --> marker
5. Preserve the overall file structure and formatting

Output ONLY the merged file content, with no explanations or code fences.`;
}

/**
 * Check if Claude CLI is available.
 */
export function isClaudeAvailable(): boolean {
  const result = spawnSync('which', ['claude'], { stdio: 'pipe' });
  return result.status === 0;
}

/**
 * Merge two file versions using Claude CLI.
 */
export function smartMerge(existingContent: string, incomingContent: string, filename: string): SmartMergeResult {
  if (!isClaudeAvailable()) {
    return {
      success: false,
      merged: null,
      error: 'Claude CLI not available. Install it with: npm install -g @anthropic-ai/claude-code'
    };
  }

  const prompt = buildMergePrompt(existingContent, incomingContent, filename);

  try {
    const result = spawnSync('claude', ['-p', prompt, '--no-input'], {
      stdio: ['pipe', 'pipe', 'pipe'],
      encoding: 'utf8',
      timeout: 60_000
    });

    if (result.status !== 0) {
      return {
        success: false,
        merged: null,
        error: `Claude exited with status ${result.status}: ${result.stderr || ''}`.trim()
      };
    }

    const output = result.stdout.trim();
    if (!output) {
      return { success: false, merged: null, error: 'Claude returned empty output' };
    }

    return { success: true, merged: output, error: null };
  } catch (err) {
    return {
      success: false,
      merged: null,
      error: `Failed to invoke Claude: ${(err as Error).message}`
    };
  }
}

/**
 * Check if two files have different content.
 * Returns true if dest doesn't exist (new file = "differs").
 * Uses Buffer.compare for byte-level accuracy.
 */
export function contentsDiffer(srcPath: string, destPath: string): boolean {
  try {
    const destBuf = fs.readFileSync(destPath);
    const srcBuf = fs.readFileSync(srcPath);
    return Buffer.compare(srcBuf, destBuf) !== 0;
  } catch {
    // dest doesn't exist or read error → treat as different
    return true;
  }
}

/**
 * Read file content for merging. Returns null if file doesn't exist.
 */
export function readFileForMerge(filePath: string): string | null {
  try {
    return fs.readFileSync(filePath, 'utf8');
  } catch {
    return null;
  }
}
