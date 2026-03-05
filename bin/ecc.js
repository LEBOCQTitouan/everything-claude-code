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
    const completion = omelette('ecc <command> [arg] [value]');

    completion.on('command', ({ reply }) => {
        reply(COMMANDS);
    });

    completion.on('arg', ({ before, reply }) => {
        switch (before) {
            case 'install':    return reply(LANGUAGES);
            case 'init':       return reply(['--template', ...LANGUAGES]);
            case 'help':       return reply(COMMANDS);
            case 'completion': return reply(['bash', 'zsh', 'fish', 'pwsh']);
            default:           return reply([]);
        }
    });

    completion.on('value', ({ before, reply }) => {
        if (before === '--template') return reply(TEMPLATES);
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

// ---------------------------------------------------------------------------
// Hidden: `ecc completion-server [<prior>...] -- <current>`
// Called by shell completion scripts. Outputs one match per line, no ANSI.
// ---------------------------------------------------------------------------
if (cmd === 'completion-server') {
    try {
        const rawArgs = args.slice(1);
        const sepIdx  = rawArgs.indexOf('--');
        const prior   = sepIdx >= 0 ? rawArgs.slice(0, sepIdx) : [];
        const current = sepIdx >= 0 ? (rawArgs[sepIdx + 1] || '') : '';

        let candidates = [];

        if (prior.length === 0) {
            // Top-level: complete commands
            candidates = COMMANDS;
        } else {
            const sub = prior[0];
            switch (sub) {
                case 'install': {
                    const selected = new Set(prior.slice(1));
                    candidates = LANGUAGES.filter(l => !selected.has(l));
                    break;
                }
                case 'init':
                    if (prior[prior.length - 1] === '--template') {
                        candidates = TEMPLATES;
                    } else {
                        candidates = ['--template', ...LANGUAGES];
                    }
                    break;
                case 'help':
                    candidates = COMMANDS;
                    break;
                case 'completion':
                    candidates = ['bash', 'zsh', 'fish', 'pwsh'];
                    break;
                default:
                    candidates = [];
            }
        }

        const matches = current
            ? candidates.filter(c => c.startsWith(current))
            : candidates;

        if (matches.length > 0) {
            process.stdout.write(matches.join('\n') + '\n');
        }
    } catch {
        // Never crash — just emit nothing
    }
    process.exit(0);
}

// ---------------------------------------------------------------------------
// `ecc completion [<shell>]`
// No shell arg: auto-detect shell via omelette and write to rc file.
// Shell arg:    output shell-specific completion script to stdout.
// ---------------------------------------------------------------------------
if (cmd === 'completion') {
    const shell = args[1];

    // No shell specified — use omelette to auto-detect and write to rc
    if (!shell) {
        if (!completionHandle) {
            console.error('ecc: omelette package not found.');
            console.error('     Run `npm install -g @lebocqtitouan/ecc` to reinstall.');
            process.exit(1);
        }
        try {
            completionHandle.setupShellInitFile();
        } catch (err) {
            console.error('ecc: completion setup failed: ' + err.message);
            process.exit(1);
        }
        console.log('');
        console.log('\u2705 Autocompletion installed! Restart your shell or reload it:');
        console.log('');
        console.log('   source ~/.zshrc                       (zsh)');
        console.log('   source ~/.bashrc                      (bash)');
        console.log('   source ~/.config/fish/config.fish     (fish)');
        console.log('   . $PROFILE                            (PowerShell)');
        console.log('');
        process.exit(0);
    }

    // Shell arg provided: output the completion script to stdout
    switch (shell) {
        case 'bash':
            process.stdout.write([
                '# ecc bash completion — generated by `ecc completion bash`',
                '# Append to ~/.bashrc:',
                '#   eval "$(ecc completion bash)"',
                '_ecc_completion() {',
                '    local cur="${COMP_WORDS[COMP_CWORD]}"',
                '    local i=1 prior=()',
                '    while [ "$i" -lt "$COMP_CWORD" ]; do',
                '        prior+=("${COMP_WORDS[$i]}")',
                '        i=$((i+1))',
                '    done',
                "    local IFS=$'\\n'",
                '    COMPREPLY=( $(ecc completion-server "${prior[@]}" -- "$cur" 2>/dev/null) )',
                '}',
                'complete -F _ecc_completion ecc',
                '',
            ].join('\n'));
            break;

        case 'zsh':
            process.stdout.write([
                '#compdef ecc',
                '# ecc zsh completion — generated by `ecc completion zsh`',
                '# Append to ~/.zshrc:',
                '#   eval "$(ecc completion zsh)"',
                '_ecc() {',
                '    local cur="${words[-1]}"',
                '    local -a prior',
                '    prior=( ${words[2,-2]} )',
                '    local -a completions',
                '    completions=( ${(f)"$(ecc completion-server ${prior} -- "$cur" 2>/dev/null)"} )',
                '    compadd -a completions',
                '}',
                'compdef _ecc ecc',
                '',
            ].join('\n'));
            break;

        case 'fish':
            process.stdout.write([
                '# ecc fish completion — generated by `ecc completion fish`',
                '# Write to ~/.config/fish/completions/ecc.fish:',
                '#   ecc completion fish > ~/.config/fish/completions/ecc.fish',
                'function __ecc_complete',
                '    set -l tokens (commandline -opc)',
                '    set -l cur (commandline -ct)',
                '    if test (count $tokens) -gt 1',
                '        set -e tokens[1]',
                '        ecc completion-server $tokens -- $cur 2>/dev/null',
                '    else',
                '        ecc completion-server -- $cur 2>/dev/null',
                '    end',
                'end',
                "complete -c ecc -f -a '(__ecc_complete)'",
                '',
            ].join('\n'));
            break;

        case 'pwsh':
        case 'powershell':
            process.stdout.write([
                '# ecc PowerShell completion — generated by `ecc completion pwsh`',
                '# Append to your $PROFILE:',
                '#   ecc completion pwsh | Out-String | Invoke-Expression',
                'Register-ArgumentCompleter -Native -CommandName ecc -ScriptBlock {',
                '    param($wordToComplete, $commandAst, $cursorPosition)',
                '    $prior = $commandAst.CommandElements |',
                '             Select-Object -Skip 1 |',
                '             Where-Object { $_.Extent.EndOffset -lt $cursorPosition } |',
                '             ForEach-Object { $_.ToString() }',
                '    ecc completion-server @prior -- $wordToComplete 2>$null |',
                '    ForEach-Object {',
                '        [System.Management.Automation.CompletionResult]::new(',
                '            $_, $_, "ParameterValue", $_)',
                '    }',
                '}',
                '',
            ].join('\n'));
            break;

        default:
            console.error("ecc: unknown shell '" + shell + "'. Supported: bash, zsh, fish, pwsh");
            process.exit(1);
    }
    process.exit(0);
}

// All other commands — delegate to install.sh
const installSh = path.resolve(__dirname, '..', 'install.sh');
const result = spawnSync('bash', [installSh, ...args], {
    stdio: 'inherit',
    env: process.env,
});

process.exit(result.status ?? 0);
