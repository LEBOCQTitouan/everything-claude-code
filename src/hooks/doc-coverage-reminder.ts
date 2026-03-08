#!/usr/bin/env node
/**
 * Doc Coverage Reminder Hook (PostToolUse: Edit|Write, async)
 *
 * After source file edits, checks if the file has undocumented exports.
 * Emits a notification reminder if undocumented public items are found.
 */

import fs from 'fs';
import path from 'path';

const MAX_STDIN = 1024 * 1024;
let raw = '';

const SOURCE_EXTENSIONS = new Set(['.ts', '.tsx', '.js', '.jsx', '.py', '.go', '.rs', '.java']);
const SKIP_PATTERNS = ['/node_modules/', '/dist/', '/build/', '/.', '/vendor/', '/__pycache__/'];

interface ExportScanResult {
  readonly totalExports: number;
  readonly undocumentedExports: number;
}

function isSourceFile(filePath: string): boolean {
  const ext = path.extname(filePath).toLowerCase();
  if (!SOURCE_EXTENSIONS.has(ext)) return false;
  return !SKIP_PATTERNS.some(p => filePath.includes(p));
}

function scanExports(filePath: string): ExportScanResult {
  const content = fs.readFileSync(filePath, 'utf8');
  const lines = content.split('\n');
  const ext = path.extname(filePath).toLowerCase();

  let totalExports = 0;
  let undocumentedExports = 0;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const isExport = detectExport(line, ext);
    if (!isExport) continue;

    totalExports++;
    const hasDoc = hasDocComment(lines, i, ext);
    if (!hasDoc) {
      undocumentedExports++;
    }
  }

  return { totalExports, undocumentedExports };
}

function detectExport(line: string, ext: string): boolean {
  const trimmed = line.trim();

  if (['.ts', '.tsx', '.js', '.jsx'].includes(ext)) {
    return /^export\s+(function|class|const|let|var|type|interface|enum|default|async\s+function)\b/.test(trimmed);
  }

  if (ext === '.py') {
    // Top-level function/class definitions (not private)
    return /^(def|class)\s+[A-Za-z]/.test(trimmed) && !trimmed.startsWith('def _') && !trimmed.startsWith('class _');
  }

  if (ext === '.go') {
    // Capitalized function/type declarations
    return /^(func|type|var|const)\s+[A-Z]/.test(trimmed);
  }

  if (ext === '.rs') {
    return /^pub\s+(fn|struct|enum|trait|type|const|static|mod)\b/.test(trimmed);
  }

  if (ext === '.java') {
    return /^public\s+(class|interface|enum|static|abstract|final|void|int|String|boolean|long|double|float)\b/.test(trimmed);
  }

  return false;
}

function hasDocComment(lines: readonly string[], exportLine: number, ext: string): boolean {
  if (exportLine === 0) return false;

  if (['.ts', '.tsx', '.js', '.jsx', '.rs', '.java'].includes(ext)) {
    // Look for /** ... */ or /// above the export
    for (let j = exportLine - 1; j >= Math.max(0, exportLine - 5); j--) {
      const prev = lines[j].trim();
      if (prev === '' || prev.startsWith('@') || prev.startsWith('#')) continue;
      if (prev.startsWith('/**') || prev.startsWith('*/') || prev.startsWith('*') || prev.startsWith('///')) return true;
      break;
    }
    return false;
  }

  if (ext === '.py') {
    // Look for docstring in the line after the def/class
    if (exportLine + 1 < lines.length) {
      const nextLine = lines[exportLine + 1].trim();
      return nextLine.startsWith('"""') || nextLine.startsWith("'''");
    }
    return false;
  }

  if (ext === '.go') {
    // Look for // comment directly above
    if (exportLine > 0) {
      const prev = lines[exportLine - 1].trim();
      return prev.startsWith('//');
    }
    return false;
  }

  return false;
}

function log(msg: string): void {
  process.stderr.write(`${msg}\n`);
}

process.stdin.setEncoding('utf8');
process.stdin.on('data', (chunk: string) => {
  if (raw.length < MAX_STDIN) {
    const remaining = MAX_STDIN - raw.length;
    raw += chunk.substring(0, remaining);
  }
});

process.stdin.on('end', () => {
  try {
    const input = JSON.parse(raw);
    const filePath = String(input.tool_input?.file_path || '');

    if (!filePath || !fs.existsSync(filePath) || !isSourceFile(filePath)) {
      process.stdout.write(raw);
      return;
    }

    const result = scanExports(filePath);

    if (result.totalExports > 0 && result.undocumentedExports > 0) {
      const basename = path.basename(filePath);
      log(
        `[DocCoverage] ${basename}: ${result.undocumentedExports}/${result.totalExports} ` +
        `exported items lack doc comments. Run /doc-generate --comments-only to add them.`
      );
    }
  } catch {
    // Ignore parse errors.
  }

  process.stdout.write(raw);
});
