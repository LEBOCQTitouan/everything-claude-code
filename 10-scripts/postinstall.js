#!/usr/bin/env node
// Runs automatically after `npm install -g @lebocqtitouan/ecc`.
// Injects the shell completion line into the user's rc file if not already present.

const fs = require('fs');
const os = require('os');
const path = require('path');

const MARKER = '# ecc shell completion';

const SHELL_CONFIGS = {
    zsh:  ['.zshrc', '.zprofile'],
    bash: ['.bashrc', '.bash_profile', '.profile'],
    fish: [path.join('.config', 'fish', 'config.fish')],
};

const COMPLETION_LINES = {
    zsh:  `\n${MARKER}\neval "$(ecc completion zsh)"\n`,
    bash: `\n${MARKER}\neval "$(ecc completion bash)"\n`,
    fish: `\n${MARKER}\necc completion fish | source\n`,
};

function detectShell() {
    const shell = process.env.SHELL || '';
    if (shell.includes('zsh'))  return 'zsh';
    if (shell.includes('fish')) return 'fish';
    if (shell.includes('bash')) return 'bash';
    return null;
}

function findRcFile(shell) {
    const candidates = SHELL_CONFIGS[shell] || [];
    for (const rel of candidates) {
        const full = path.join(os.homedir(), rel);
        if (fs.existsSync(full)) return full;
    }
    // Return the first candidate as default (will be created)
    return candidates.length ? path.join(os.homedir(), candidates[0]) : null;
}

function alreadyInstalled(rcFile) {
    try {
        return fs.readFileSync(rcFile, 'utf8').includes(MARKER);
    } catch {
        return false;
    }
}

function install(shell, rcFile) {
    fs.mkdirSync(path.dirname(rcFile), { recursive: true });
    fs.appendFileSync(rcFile, COMPLETION_LINES[shell]);
}

// --- main ---
const shell = detectShell();

if (!shell) {
    console.log('ecc: Could not detect shell — run `eval "$(ecc completion)"` manually to enable tab completion.');
    process.exit(0);
}

const rcFile = findRcFile(shell);
if (!rcFile) {
    console.log(`ecc: Could not find ${shell} config file — run \`eval "$(ecc completion)"\` manually.`);
    process.exit(0);
}

if (alreadyInstalled(rcFile)) {
    // Already set up, nothing to do
    process.exit(0);
}

try {
    install(shell, rcFile);
    console.log(`ecc: Shell completion enabled in ${rcFile}`);
    console.log(`     Run \`source ${rcFile}\` or open a new terminal to activate.`);
} catch (err) {
    console.log(`ecc: Could not write to ${rcFile} — run \`eval "$(ecc completion)"\` manually. (${err.message})`);
}
