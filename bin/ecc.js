#!/usr/bin/env node
'use strict';

const path = require('path');
const { spawnSync } = require('child_process');

// ---------------------------------------------------------------------------
// Completion data — must match install.sh commands/options exactly
// ---------------------------------------------------------------------------
const COMMANDS  = ['install', 'init', 'help', 'completion'];
const LANGUAGES = ['golang', 'python', 'rust', 'swift', 'typescript'];
const TEMPLATES = ['django-api', 'go-microservice', 'rust-api', 'saas-nextjs'];

// ---------------------------------------------------------------------------
// Part 1: omelette shell autocompletion
// Must be wired up BEFORE any argument parsing.
// ---------------------------------------------------------------------------
let completionHandle;

try {
    const omelette = require('omelette');

    // Template: ecc <command> [arg] [value]
    // - command → top-level commands
    // - arg     → per-command args / flags  (before = command name)
    // - value   → per-arg values           (before = flag or language)
    const completion = omelette('ecc <command> [arg] [value]');

    completion.on('command', ({ reply }) => {
        reply(COMMANDS);
    });

    completion.on('arg', ({ before, reply }) => {
        switch (before) {
            case 'install':    return reply(LANGUAGES);
            case 'init':       return reply(['--template', ...LANGUAGES]);
            case 'help':       return reply(COMMANDS);
            case 'completion': return reply(['bash', 'zsh', 'fish', 'powershell']);
            default:           return reply([]);
        }
    });

    completion.on('value', ({ before, reply }) => {
        // --template <TAB> → template names
        if (before === '--template') return reply(TEMPLATES);
        // ecc install typescript <TAB> → remaining languages
        if (LANGUAGES.includes(before)) return reply(LANGUAGES.filter(l => l !== before));
        return reply([]);
    });

    try {
        completion.init(); // must run before argv parsing
    } catch {
        // never crash the CLI over a completion init failure
    }

    completionHandle = completion;
} catch {
    // omelette unavailable — CLI works normally, completions disabled
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------
const args = process.argv.slice(2);
const cmd  = args[0];

// `ecc completion` — set up shell completion via omelette
if (cmd === 'completion') {
    if (!completionHandle) {
        console.error('ecc: omelette package not found.');
        console.error('     Run `npm install -g @lebocqtitouan/ecc` to reinstall.');
        process.exit(1);
    }

    completionHandle.setupShellInitFile();

    console.log('');
    console.log('✅ Autocompletion installed! Restart your shell or reload it:');
    console.log('');
    console.log('   source ~/.zshrc                       (zsh)');
    console.log('   source ~/.bashrc                      (bash)');
    console.log('   source ~/.config/fish/config.fish     (fish)');
    console.log('   . $PROFILE                            (PowerShell)');
    console.log('');
    process.exit(0);
}

// All other commands — delegate to install.sh
const installSh = path.resolve(__dirname, '..', 'install.sh');
const result = spawnSync('bash', [installSh, ...args], {
    stdio: 'inherit',
    env: process.env,
});

process.exit(result.status ?? 0);
