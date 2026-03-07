#!/usr/bin/env node
'use strict';

// Runs automatically after `npm install -g @lebocqtitouan/ecc`.
// Checks environment health and prints getting-started hints.
// Never exits with a non-zero code unless Node.js version is fatally too old.
// Uses ONLY Node.js built-ins — no external require() calls.

import fs from 'fs';
import path from 'path';
import { execFileSync } from 'child_process';

// ANSI color helpers
const green  = (s: string) => `\x1b[32m${s}\x1b[0m`;
const yellow = (s: string) => `\x1b[33m${s}\x1b[0m`;
const red    = (s: string) => `\x1b[31m${s}\x1b[0m`;
const bold   = (s: string) => `\x1b[1m${s}\x1b[0m`;
const dim    = (s: string) => `\x1b[2m${s}\x1b[0m`;

function checkNodeVersion(): void {
  let minMajor = 18;
  try {
    const pkg = JSON.parse(fs.readFileSync(path.join(__dirname, '..', 'package.json'), 'utf8')) as {
      engines?: { node?: string };
    };
    const enginesNode = pkg.engines?.node || '>=18';
    const match = enginesNode.match(/(\d+)/);
    if (match) minMajor = parseInt(match[1], 10);
  } catch { /* use fallback */ }

  const major = parseInt(process.versions.node.split('.')[0], 10);
  if (major < minMajor) {
    console.error(red(`\n  \u2716 Node.js ${minMajor}+ is required (found ${process.versions.node})`));
    console.error(yellow(`    Update at: https://nodejs.org\n`));
    process.exit(1);
  }
}

function checkBash(): void {
  try {
    execFileSync('bash', ['--version'], { stdio: 'ignore' });
  } catch {
    console.warn(yellow('  \u26a0 bash not found in PATH'));
    console.warn(yellow('    ecc requires bash to run. Install bash or ensure it is in your PATH.'));
  }
}

function checkDependencies(): void {
  let deps: Record<string, string> = {};
  try {
    const pkg = JSON.parse(fs.readFileSync(path.join(__dirname, '..', 'package.json'), 'utf8')) as {
      dependencies?: Record<string, string>;
    };
    deps = pkg.dependencies || {};
  } catch { /* skip dependency check if package.json unreadable */ }

  for (const dep of Object.keys(deps)) {
    try {
      require.resolve(dep);
    } catch {
      console.warn(yellow(`  \u26a0 dependency '${dep}' missing — some features may not work`));
      console.warn(yellow(`    Run: npm install -g @lebocqtitouan/ecc  to reinstall cleanly`));
    }
  }
}

try {
  checkNodeVersion();
  checkBash();
  checkDependencies();

  console.log('');
  console.log(bold(green('  \u2714 ecc installed successfully')));
  console.log(dim('    Claude Code configuration manager'));
  console.log('');
  console.log(`  ${bold('Getting started:')}`);
  console.log(`    ${bold('ecc completion')}              enable shell tab-completion`);
  console.log(`    ${bold('ecc install typescript')}      global Claude setup`);
  console.log(`    ${bold('ecc init')}                    per-project setup`);
  console.log(`    ${bold('ecc help')}                    full command reference`);
  console.log('');
} catch (err: unknown) {
  console.warn(yellow(`  ecc: postinstall warning: ${(err as Error).message}`));
}
