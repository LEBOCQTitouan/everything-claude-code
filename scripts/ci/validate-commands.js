#!/usr/bin/env node
"use strict";
/**
 * Validate command markdown files are non-empty, readable,
 * and have valid cross-references to other commands, agents, and skills.
 */
const fs = require("fs");
const path = require("path");

const ROOT_DIR = path.join(__dirname, '../..');
const COMMANDS_DIR = path.join(ROOT_DIR, 'commands');
const AGENTS_DIR = path.join(ROOT_DIR, 'agents');
const SKILLS_DIR = path.join(ROOT_DIR, 'skills');

function validateCommands() {
    if (!fs.existsSync(COMMANDS_DIR)) {
        console.log('No commands directory found, skipping validation');
        process.exit(0);
    }
    const files = fs.readdirSync(COMMANDS_DIR).filter(f => f.endsWith('.md'));
    let hasErrors = false;
    let warnCount = 0;
    const validCommands = new Set(files.map(f => f.replace(/\.md$/, '')));
    const validAgents = new Set();
    if (fs.existsSync(AGENTS_DIR)) {
        for (const f of fs.readdirSync(AGENTS_DIR)) {
            if (f.endsWith('.md')) {
                validAgents.add(f.replace(/\.md$/, ''));
            }
        }
    }
    const validSkills = new Set();
    if (fs.existsSync(SKILLS_DIR)) {
        for (const f of fs.readdirSync(SKILLS_DIR)) {
            const skillPath = path.join(SKILLS_DIR, f);
            try {
                if (fs.statSync(skillPath).isDirectory()) {
                    validSkills.add(f);
                }
            }
            catch {
                // skip unreadable entries
            }
        }
    }
    for (const file of files) {
        const filePath = path.join(COMMANDS_DIR, file);
        let content;
        try {
            content = fs.readFileSync(filePath, 'utf-8');
        }
        catch (err) {
            console.error(`ERROR: ${file} - ${err.message}`);
            hasErrors = true;
            continue;
        }
        if (content.trim().length === 0) {
            console.error(`ERROR: ${file} - Empty command file`);
            hasErrors = true;
            continue;
        }
        const contentNoCodeBlocks = content.replace(/```[\s\S]*?```/g, '');
        for (const line of contentNoCodeBlocks.split('\n')) {
            if (/creates:|would create:/i.test(line))
                continue;
            const lineRefs = line.matchAll(/`\/([a-z][-a-z0-9]*)`/g);
            for (const match of lineRefs) {
                const refName = match[1];
                if (!validCommands.has(refName)) {
                    console.error(`ERROR: ${file} - references non-existent command /${refName}`);
                    hasErrors = true;
                }
            }
        }
        const agentPathRefs = contentNoCodeBlocks.matchAll(/agents\/([a-z][-a-z0-9]*)\.md/g);
        for (const match of agentPathRefs) {
            const refName = match[1];
            if (!validAgents.has(refName)) {
                console.error(`ERROR: ${file} - references non-existent agent agents/${refName}.md`);
                hasErrors = true;
            }
        }
        const skillRefs = contentNoCodeBlocks.matchAll(/skills\/([a-z][-a-z0-9]*)\//g);
        for (const match of skillRefs) {
            const refName = match[1];
            if (!validSkills.has(refName)) {
                console.warn(`WARN: ${file} - references skill directory skills/${refName}/ (not found locally)`);
                warnCount++;
            }
        }
        const workflowLines = contentNoCodeBlocks.matchAll(/^([a-z][-a-z0-9]*(?:\s*->\s*[a-z][-a-z0-9]*)+)$/gm);
        for (const match of workflowLines) {
            const agents = match[1].split(/\s*->\s*/);
            for (const agent of agents) {
                if (!validAgents.has(agent)) {
                    console.error(`ERROR: ${file} - workflow references non-existent agent "${agent}"`);
                    hasErrors = true;
                }
            }
        }
    }
    if (hasErrors) {
        process.exit(1);
    }
    let msg = `Validated ${files.length} command files`;
    if (warnCount > 0) {
        msg += ` (${warnCount} warnings)`;
    }
    console.log(msg);
}

validateCommands();
