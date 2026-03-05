#!/usr/bin/env node
// Runs automatically before `npm uninstall -g @lebocqtitouan/ecc`.
// Removes static completion files written by postinstall.

const fs = require('fs');
const os = require('os');
const path = require('path');

const HOME = os.homedir();
const MARKER = '# ecc completion';

const COMPLETION_FILES = [
    path.join(HOME, '.config', 'fish', 'completions', 'ecc.fish'),
    path.join(HOME, '.local', 'share', 'bash-completion', 'completions', 'ecc'),
    path.join(HOME, '.zsh', 'completions', '_ecc'),
];

// Remove completion files
for (const f of COMPLETION_FILES) {
    try {
        fs.unlinkSync(f);
        console.log(`ecc: removed ${f}`);
    } catch {}
}

// Remove the fpath block from .zshrc
const rcFile = path.join(HOME, '.zshrc');
try {
    const content = fs.readFileSync(rcFile, 'utf8');
    if (content.includes(MARKER)) {
        // Remove the marker line + the two lines after it (fpath + autoload)
        const cleaned = content.replace(
            /\n# ecc completion\nfpath=\(~\/.zsh\/completions \$fpath\)\nautoload -Uz compinit && compinit\n/,
            ''
        );
        fs.writeFileSync(rcFile, cleaned);
        console.log(`ecc: removed fpath entry from ${rcFile}`);
    }
} catch {}
