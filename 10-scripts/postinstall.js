#!/usr/bin/env node
'use strict';

// Runs automatically after `npm install -g @lebocqtitouan/ecc`.
// Checks environment health and prints getting-started hints.
// Never exits with a non-zero code unless Node.js version is fatally too old.

const { execFileSync } = require('child_process');

// ---------------------------------------------------------------------------
// ANSI color helpers — no external dependencies
// ---------------------------------------------------------------------------
const green  = (s) => `\x1b[32m${s}\x1b[0m`;
const yellow = (s) => `\x1b[33m${s}\x1b[0m`;
const red    = (s) => `\x1b[31m${s}\x1b[0m`;
const bold   = (s) => `\x1b[1m${s}\x1b[0m`;
const dim    = (s) => `\x1b[2m${s}\x1b[0m`;

const MIN_NODE_MAJOR = 18;

// ---------------------------------------------------------------------------
// Checks
// ---------------------------------------------------------------------------
function checkNodeVersion() {
    const major = parseInt(process.versions.node.split('.')[0], 10);
    if (major < MIN_NODE_MAJOR) {
        console.error(red(`\n  ✖ Node.js ${MIN_NODE_MAJOR}+ is required (found ${process.versions.node})`));
        console.error(yellow(`    Update at: https://nodejs.org\n`));
        process.exit(1); // hard failure — ecc cannot run on this Node version
    }
}

function checkBash() {
    try {
        execFileSync('bash', ['--version'], { stdio: 'ignore' });
    } catch {
        console.warn(yellow('  ⚠ bash not found in PATH'));
        console.warn(yellow('    ecc requires bash to run. Install bash or ensure it is in your PATH.'));
    }
}

function checkOmelette() {
    try {
        require('omelette');
    } catch {
        console.warn(yellow('  ⚠ omelette dependency missing — shell completion will not work'));
        console.warn(yellow('    Run: npm install -g @lebocqtitouan/ecc  to reinstall cleanly'));
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------
try {
    checkNodeVersion();   // exits with code 1 if Node is too old
    checkBash();
    checkOmelette();

    console.log('');
    console.log(bold(green('  ✔ ecc installed successfully')));
    console.log(dim('    Claude Code configuration manager'));
    console.log('');
    console.log(`  ${bold('Getting started:')}`);
    console.log(`    ${bold('ecc completion')}              enable shell tab-completion`);
    console.log(`    ${bold('ecc install typescript')}      global Claude setup`);
    console.log(`    ${bold('ecc init')}                    per-project setup`);
    console.log(`    ${bold('ecc help')}                    full command reference`);
    console.log('');
} catch (err) {
    // Catch-all: a broken postinstall must never block npm install
    console.warn(yellow(`  ecc: postinstall warning: ${err.message}`));
}
