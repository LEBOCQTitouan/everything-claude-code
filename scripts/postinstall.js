#!/usr/bin/env node
'use strict';

// Runs automatically after `npm install -g @lebocqtitouan/ecc`.
// Checks environment health and prints getting-started hints.
// Never exits with a non-zero code unless Node.js version is fatally too old.
// Uses ONLY Node.js built-ins — no external require() calls.

const fs   = require('fs');
const path = require('path');
const { execFileSync } = require('child_process');

// ---------------------------------------------------------------------------
// ANSI color helpers — no external dependencies
// ---------------------------------------------------------------------------
const green  = (s) => `\x1b[32m${s}\x1b[0m`;
const yellow = (s) => `\x1b[33m${s}\x1b[0m`;
const red    = (s) => `\x1b[31m${s}\x1b[0m`;
const bold   = (s) => `\x1b[1m${s}\x1b[0m`;
const dim    = (s) => `\x1b[2m${s}\x1b[0m`;

// ---------------------------------------------------------------------------
// Checks
// ---------------------------------------------------------------------------
function checkNodeVersion() {
    // Read required version from package.json engines field
    let minMajor = 18; // fallback
    try {
        const pkg = JSON.parse(fs.readFileSync(path.join(__dirname, '..', 'package.json'), 'utf8'));
        const enginesNode = (pkg.engines && pkg.engines.node) || '>=18';
        const match = enginesNode.match(/(\d+)/);
        if (match) minMajor = parseInt(match[1], 10);
    } catch { /* use fallback */ }

    const major = parseInt(process.versions.node.split('.')[0], 10);
    if (major < minMajor) {
        console.error(red(`\n  \u2716 Node.js ${minMajor}+ is required (found ${process.versions.node})`));
        console.error(yellow(`    Update at: https://nodejs.org\n`));
        process.exit(1); // hard failure — ecc cannot run on this Node version
    }
}

function checkBash() {
    try {
        execFileSync('bash', ['--version'], { stdio: 'ignore' });
    } catch {
        console.warn(yellow('  \u26a0 bash not found in PATH'));
        console.warn(yellow('    ecc requires bash to run. Install bash or ensure it is in your PATH.'));
    }
}

function checkDependencies() {
    // Read dependencies from package.json — no external modules needed
    let deps = {};
    try {
        const pkg = JSON.parse(fs.readFileSync(path.join(__dirname, '..', 'package.json'), 'utf8'));
        deps = pkg.dependencies || {};
    } catch { /* skip dependency check if package.json unreadable */ }

    for (const dep of Object.keys(deps)) {
        try {
            require.resolve(dep); // built-in — resolves path without executing
        } catch {
            console.warn(yellow(`  \u26a0 dependency '${dep}' missing — some features may not work`));
            console.warn(yellow(`    Run: npm install -g @lebocqtitouan/ecc  to reinstall cleanly`));
        }
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------
try {
    checkNodeVersion();   // exits with code 1 if Node is too old
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
} catch (err) {
    // Catch-all: a broken postinstall must never block npm install
    console.warn(yellow(`  ecc: postinstall warning: ${err.message}`));
}
