#!/usr/bin/env node
/**
 * PreToolUse Hook: Warn about non-standard documentation files
 *
 * Cross-platform (Windows, macOS, Linux)
 *
 * Exit code 0 — warn only, does not block.
 */

import path from 'path';

const MAX_STDIN = 1024 * 1024;
let data = '';
process.stdin.setEncoding('utf8');

process.stdin.on('data', (chunk: string) => {
  if (data.length < MAX_STDIN) {
    const remaining = MAX_STDIN - data.length;
    data += chunk.length > remaining ? chunk.slice(0, remaining) : chunk;
  }
});

process.stdin.on('end', () => {
  try {
    const input = JSON.parse(data);
    const filePath: string = input.tool_input?.file_path || '';

    if (!/\.(md|txt)$/.test(filePath)) {
      process.stdout.write(data);
      return;
    }

    const basename = path.basename(filePath);
    if (/^(README|CLAUDE|AGENTS|CONTRIBUTING|CHANGELOG|LICENSE|SKILL)\.md$/i.test(basename)) {
      process.stdout.write(data);
      return;
    }

    const normalized = filePath.replace(/\\/g, '/');
    if (/\.claude\/plans\//.test(normalized) || /(^|\/)(docs|skills)\//.test(normalized)) {
      process.stdout.write(data);
      return;
    }

    console.error('[Hook] WARNING: Non-standard documentation file detected');
    console.error('[Hook] File: ' + filePath);
    console.error('[Hook] Consider consolidating into README.md or docs/ directory');
  } catch {
    // Parse error — pass through
  }

  process.stdout.write(data);
});
