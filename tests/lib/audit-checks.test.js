/**
 * Tests for src/lib/audit-checks.ts
 *
 * Run with: npx tsx tests/lib/audit-checks.test.js
 */

const assert = require('assert');
const fs = require('fs');
const path = require('path');
const os = require('os');

const {
  checkDenyRules,
  checkGitignore,
  checkHookDuplicates,
  checkGlobalClaudeMd,
  checkAgentSkills,
  checkCommandDescriptions,
  checkProjectClaudeMd,
  runAllChecks
} = require('../../src/lib/audit-checks');
const { test, describe } = require('../harness');

function makeTempDir() {
  return fs.mkdtempSync(path.join(os.tmpdir(), 'ecc-audit-test-'));
}

function cleanup(dir) {
  fs.rmSync(dir, { recursive: true, force: true });
}

async function runTests() {
  describe('Testing audit-checks.ts');

  // --- checkDenyRules ---
  describe('checkDenyRules');

  await test('fails when no settings.json exists', () => {
    const result = checkDenyRules('/nonexistent/path/settings.json');
    assert.strictEqual(result.passed, false);
    assert.strictEqual(result.findings[0].severity, 'critical');
  });

  const tmpDir1 = makeTempDir();
  try {
    await test('fails when deny rules are missing', () => {
      const settingsPath = path.join(tmpDir1, 'settings.json');
      fs.writeFileSync(settingsPath, JSON.stringify({ hooks: {} }));
      const result = checkDenyRules(settingsPath);
      assert.strictEqual(result.passed, false);
      assert.ok(result.findings.some(f => f.id === 'DENY-002'));
    });
  } finally {
    cleanup(tmpDir1);
  }

  const tmpDir2 = makeTempDir();
  try {
    await test('passes when all deny rules present', () => {
      const settingsPath = path.join(tmpDir2, 'settings.json');
      const { ECC_DENY_RULES } = require('../../src/lib/deny-rules');
      fs.writeFileSync(settingsPath, JSON.stringify({
        permissions: { deny: [...ECC_DENY_RULES] }
      }));
      const result = checkDenyRules(settingsPath);
      assert.strictEqual(result.passed, true);
    });
  } finally {
    cleanup(tmpDir2);
  }

  // --- checkGitignore ---
  describe('checkGitignore');

  await test('fails when no .gitignore exists', () => {
    const result = checkGitignore('/nonexistent/dir');
    assert.strictEqual(result.passed, false);
  });

  const tmpDir3 = makeTempDir();
  try {
    await test('fails when entries are missing from .gitignore', () => {
      fs.writeFileSync(path.join(tmpDir3, '.gitignore'), 'node_modules/\n');
      const result = checkGitignore(tmpDir3);
      assert.strictEqual(result.passed, false);
      assert.ok(result.findings.some(f => f.id === 'GIT-002'));
    });
  } finally {
    cleanup(tmpDir3);
  }

  const tmpDir4 = makeTempDir();
  try {
    await test('passes when all entries present in .gitignore', () => {
      const { ECC_GITIGNORE_ENTRIES } = require('../../src/lib/gitignore');
      const content = ECC_GITIGNORE_ENTRIES.map(e => e.pattern).join('\n') + '\n';
      fs.writeFileSync(path.join(tmpDir4, '.gitignore'), content);
      const result = checkGitignore(tmpDir4);
      assert.strictEqual(result.passed, true);
    });
  } finally {
    cleanup(tmpDir4);
  }

  // --- checkHookDuplicates ---
  describe('checkHookDuplicates');

  await test('passes when no settings exist', () => {
    const result = checkHookDuplicates('/nonexistent/settings.json');
    assert.strictEqual(result.passed, true);
  });

  const tmpDir5 = makeTempDir();
  try {
    await test('detects duplicate hooks', () => {
      const settingsPath = path.join(tmpDir5, 'settings.json');
      const hook = { type: 'command', command: 'echo test' };
      fs.writeFileSync(settingsPath, JSON.stringify({
        hooks: {
          PreToolUse: [
            { matcher: 'Bash', hooks: [hook] },
            { matcher: 'Bash', hooks: [hook] }
          ]
        }
      }));
      const result = checkHookDuplicates(settingsPath);
      assert.strictEqual(result.passed, false);
      assert.ok(result.findings.some(f => f.id === 'HOOK-001'));
    });
  } finally {
    cleanup(tmpDir5);
  }

  const tmpDir6 = makeTempDir();
  try {
    await test('passes with no duplicate hooks', () => {
      const settingsPath = path.join(tmpDir6, 'settings.json');
      fs.writeFileSync(settingsPath, JSON.stringify({
        hooks: {
          PreToolUse: [
            { matcher: 'Bash', hooks: [{ type: 'command', command: 'echo a' }] },
            { matcher: 'Bash', hooks: [{ type: 'command', command: 'echo b' }] }
          ]
        }
      }));
      const result = checkHookDuplicates(settingsPath);
      assert.strictEqual(result.passed, true);
    });
  } finally {
    cleanup(tmpDir6);
  }

  // --- checkGlobalClaudeMd ---
  describe('checkGlobalClaudeMd');

  await test('fails when no CLAUDE.md exists', () => {
    const result = checkGlobalClaudeMd('/nonexistent/dir');
    assert.strictEqual(result.passed, false);
    assert.ok(result.findings.some(f => f.id === 'CMD-001'));
  });

  const tmpDir7 = makeTempDir();
  try {
    await test('passes when CLAUDE.md exists', () => {
      fs.writeFileSync(path.join(tmpDir7, 'CLAUDE.md'), '# Instructions\n');
      const result = checkGlobalClaudeMd(tmpDir7);
      assert.strictEqual(result.passed, true);
    });
  } finally {
    cleanup(tmpDir7);
  }

  // --- checkAgentSkills ---
  describe('checkAgentSkills');

  await test('passes when directory does not exist', () => {
    const result = checkAgentSkills('/nonexistent/agents');
    assert.strictEqual(result.passed, true);
  });

  const tmpDir8 = makeTempDir();
  try {
    await test('detects agents without skills preloading', () => {
      const agentsDir = path.join(tmpDir8, 'agents');
      fs.mkdirSync(agentsDir);
      // Create 6 agents without skills
      for (let i = 0; i < 6; i++) {
        fs.writeFileSync(path.join(agentsDir, `agent-${i}.md`),
          `---\nname: agent-${i}\ndescription: Test agent\ntools: ["Read"]\nmodel: sonnet\n---\n\n# Agent ${i}\n`);
      }
      const result = checkAgentSkills(agentsDir);
      assert.strictEqual(result.passed, false);
    });
  } finally {
    cleanup(tmpDir8);
  }

  // --- checkCommandDescriptions ---
  describe('checkCommandDescriptions');

  const tmpDir9 = makeTempDir();
  try {
    await test('detects commands without description', () => {
      const cmdsDir = path.join(tmpDir9, 'commands');
      fs.mkdirSync(cmdsDir);
      fs.writeFileSync(path.join(cmdsDir, 'plan.md'),
        '---\ndescription: Plan features\n---\n# Plan\n');
      fs.writeFileSync(path.join(cmdsDir, 'fix.md'),
        '# Fix\nFix stuff.\n');
      const result = checkCommandDescriptions(cmdsDir);
      assert.strictEqual(result.passed, false);
      assert.ok(result.findings[0].detail.includes('fix.md'));
    });
  } finally {
    cleanup(tmpDir9);
  }

  // --- checkProjectClaudeMd ---
  describe('checkProjectClaudeMd');

  await test('passes when no CLAUDE.md exists', () => {
    const result = checkProjectClaudeMd('/nonexistent/project');
    assert.strictEqual(result.passed, true);
  });

  const tmpDir10 = makeTempDir();
  try {
    await test('warns when CLAUDE.md exceeds 200 lines', () => {
      const lines = Array(250).fill('Some instruction line.').join('\n');
      fs.writeFileSync(path.join(tmpDir10, 'CLAUDE.md'), lines);
      const result = checkProjectClaudeMd(tmpDir10);
      assert.strictEqual(result.passed, false);
      assert.ok(result.findings.some(f => f.id === 'PCM-001'));
    });
  } finally {
    cleanup(tmpDir10);
  }

  const tmpDir11 = makeTempDir();
  try {
    await test('passes when CLAUDE.md is under 200 lines', () => {
      const lines = Array(50).fill('Instruction.').join('\n');
      fs.writeFileSync(path.join(tmpDir11, 'CLAUDE.md'), lines);
      const result = checkProjectClaudeMd(tmpDir11);
      assert.strictEqual(result.passed, true);
    });
  } finally {
    cleanup(tmpDir11);
  }

  // --- runAllChecks ---
  describe('runAllChecks');

  const tmpDir12 = makeTempDir();
  try {
    await test('returns a valid audit report', () => {
      const claudeDir = path.join(tmpDir12, '.claude');
      fs.mkdirSync(claudeDir, { recursive: true });
      const agentsDir = path.join(tmpDir12, 'agents');
      fs.mkdirSync(agentsDir);
      const cmdsDir = path.join(tmpDir12, 'commands');
      fs.mkdirSync(cmdsDir);
      fs.writeFileSync(path.join(claudeDir, 'settings.json'), '{}');

      const report = runAllChecks({
        claudeDir,
        projectDir: tmpDir12,
        eccRoot: tmpDir12
      });

      assert.ok(report.checks.length > 0);
      assert.ok(typeof report.score === 'number');
      assert.ok(['A', 'B', 'C', 'D', 'F'].includes(report.grade));
      assert.ok(report.score >= 0 && report.score <= 100);
    });
  } finally {
    cleanup(tmpDir12);
  }

  await test('grade reflects severity of findings', () => {
    const report = runAllChecks({
      claudeDir: '/nonexistent',
      projectDir: '/nonexistent',
      eccRoot: '/nonexistent'
    });

    // With many missing things, should have a low score
    assert.ok(report.score < 80, `Score ${report.score} should be below 80 with missing config`);
  });
}

module.exports = { runTests };

if (require.main === module) {
  const { getResults, resetCounters } = require('../harness');
  resetCounters();
  runTests().then(() => {
    const r = getResults();
    console.log('\nPassed: ' + r.passed);
    console.log('Failed: ' + r.failed);
    console.log('Total:  ' + (r.passed + r.failed));
    if (r.failed > 0) process.exit(1);
  });
}
