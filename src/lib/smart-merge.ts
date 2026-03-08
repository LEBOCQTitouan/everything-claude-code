/**
 * Claude-powered intelligent file merging.
 * Invokes Claude CLI to merge two versions of a file,
 * preserving user customizations while incorporating ECC updates.
 */

import { spawnSync } from 'child_process';
import fs from 'fs';

export interface SmartMergeResult {
  success: boolean;
  merged: string | null;
  error: string | null;
}

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
 * Returns the merged content or an error.
 */
export function smartMerge(
  existingContent: string,
  incomingContent: string,
  filename: string,
): SmartMergeResult {
  if (!isClaudeAvailable()) {
    return {
      success: false,
      merged: null,
      error: 'Claude CLI not available. Install it with: npm install -g @anthropic-ai/claude-code',
    };
  }

  const prompt = buildMergePrompt(existingContent, incomingContent, filename);

  try {
    const result = spawnSync('claude', ['-p', prompt, '--no-input'], {
      stdio: ['pipe', 'pipe', 'pipe'],
      encoding: 'utf8',
      timeout: 60_000, // 60 second timeout
    });

    if (result.status !== 0) {
      return {
        success: false,
        merged: null,
        error: `Claude exited with status ${result.status}: ${result.stderr || ''}`.trim(),
      };
    }

    const output = result.stdout.trim();
    if (!output) {
      return {
        success: false,
        merged: null,
        error: 'Claude returned empty output',
      };
    }

    return {
      success: true,
      merged: output,
      error: null,
    };
  } catch (err) {
    return {
      success: false,
      merged: null,
      error: `Failed to invoke Claude: ${(err as Error).message}`,
    };
  }
}

/**
 * Generate a unified diff between two strings for display.
 */
export function generateDiff(existingContent: string, incomingContent: string, filename: string): string {
  const existingLines = existingContent.split('\n');
  const incomingLines = incomingContent.split('\n');

  const lines: string[] = [`--- existing/${filename}`, `+++ incoming/${filename}`];

  // Simple line-by-line diff (not a true unified diff, but useful for display)
  const maxLen = Math.max(existingLines.length, incomingLines.length);
  for (let i = 0; i < maxLen; i++) {
    const existingLine = i < existingLines.length ? existingLines[i] : undefined;
    const incomingLine = i < incomingLines.length ? incomingLines[i] : undefined;

    if (existingLine === incomingLine) {
      lines.push(` ${existingLine}`);
    } else {
      if (existingLine !== undefined) lines.push(`-${existingLine}`);
      if (incomingLine !== undefined) lines.push(`+${incomingLine}`);
    }
  }

  return lines.join('\n');
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
