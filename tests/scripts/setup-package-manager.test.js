/**
 * Tests for scripts/setup-package-manager.js
 *
 * Tests CLI argument parsing and output via subprocess invocation.
 *
 * Run with: node tests/scripts/setup-package-manager.test.js
 */

const assert = require('assert');
const path = require('path');
const fs = require('fs');
const os = require('os');
const { execFileSync } = require('child_process');

const SCRIPT = path.join(__dirname, '..', '..', 'dist', 'setup-package-manager.js');
const { test, describe } = require('../harness');

// Run the script with given args, return { stdout, stderr, code }
function run(args = [], env = {}) {
  try {
    const stdout = execFileSync('node', [SCRIPT, ...args], {
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'pipe'],
      env: { ...process.env, ...env },
      timeout: 10000
    });
    return { stdout, stderr: '', code: 0 };
  } catch (err) {
    return {
      stdout: err.stdout || '',
      stderr: err.stderr || '',
      code: err.status || 1
    };
  }
}

async function runTests() {
  describe('Testing setup-package-manager.js');

  // --help flag
  describe('--help');

  await test('shows help with --help flag', () => {
    const result = run(['--help']);
    assert.strictEqual(result.code, 0);
    assert.ok(result.stdout.includes('Package Manager Setup'));
    assert.ok(result.stdout.includes('--detect'));
    assert.ok(result.stdout.includes('--global'));
    assert.ok(result.stdout.includes('--project'));
  });

  await test('shows help with -h flag', () => {
    const result = run(['-h']);
    assert.strictEqual(result.code, 0);
    assert.ok(result.stdout.includes('Package Manager Setup'));
  });

  await test('shows help with no arguments', () => {
    const result = run([]);
    assert.strictEqual(result.code, 0);
    assert.ok(result.stdout.includes('Package Manager Setup'));
  });

  // --detect flag
  describe('--detect');

  await test('detects current package manager', () => {
    const result = run(['--detect']);
    assert.strictEqual(result.code, 0);
    assert.ok(result.stdout.includes('Package Manager Detection'));
    assert.ok(result.stdout.includes('Current selection'));
  });

  await test('shows detection sources', () => {
    const result = run(['--detect']);
    assert.ok(result.stdout.includes('From package.json'));
    assert.ok(result.stdout.includes('From lock file'));
    assert.ok(result.stdout.includes('Environment var'));
  });

  await test('shows available managers in detection output', () => {
    const result = run(['--detect']);
    assert.ok(result.stdout.includes('npm'));
    assert.ok(result.stdout.includes('pnpm'));
    assert.ok(result.stdout.includes('yarn'));
    assert.ok(result.stdout.includes('bun'));
  });

  // --list flag
  describe('--list');

  await test('lists available package managers', () => {
    const result = run(['--list']);
    assert.strictEqual(result.code, 0);
    assert.ok(result.stdout.includes('Available Package Managers'));
    assert.ok(result.stdout.includes('npm'));
    assert.ok(result.stdout.includes('Lock file'));
    assert.ok(result.stdout.includes('Install'));
  });

  // --global flag
  describe('--global');

  await test('rejects --global without package manager name', () => {
    const result = run(['--global']);
    assert.strictEqual(result.code, 1);
    assert.ok(result.stderr.includes('requires a package manager name'));
  });

  await test('rejects --global with unknown package manager', () => {
    const result = run(['--global', 'unknown-pm']);
    assert.strictEqual(result.code, 1);
    assert.ok(result.stderr.includes('Unknown package manager'));
  });

  // --project flag
  describe('--project');

  await test('rejects --project without package manager name', () => {
    const result = run(['--project']);
    assert.strictEqual(result.code, 1);
    assert.ok(result.stderr.includes('requires a package manager name'));
  });

  await test('rejects --project with unknown package manager', () => {
    const result = run(['--project', 'unknown-pm']);
    assert.strictEqual(result.code, 1);
    assert.ok(result.stderr.includes('Unknown package manager'));
  });

  // Positional argument
  describe('positional argument');

  await test('rejects unknown positional argument', () => {
    const result = run(['not-a-pm']);
    assert.strictEqual(result.code, 1);
    assert.ok(result.stderr.includes('Unknown option or package manager'));
  });

  // Environment variable
  describe('environment variable');

  await test('detects env var override', () => {
    const result = run(['--detect'], { CLAUDE_PACKAGE_MANAGER: 'pnpm' });
    assert.strictEqual(result.code, 0);
    assert.ok(result.stdout.includes('pnpm'));
  });

  // --detect output completeness
  describe('--detect output completeness');

  await test('shows all three command types in detection output', () => {
    const result = run(['--detect']);
    assert.strictEqual(result.code, 0);
    assert.ok(result.stdout.includes('Install:'), 'Should show Install command');
    assert.ok(result.stdout.includes('Run script:'), 'Should show Run script command');
    assert.ok(result.stdout.includes('Execute binary:'), 'Should show Execute binary command');
  });

  await test('shows current marker for active package manager', () => {
    const result = run(['--detect']);
    assert.ok(result.stdout.includes('(current)'), 'Should mark current PM');
  });

  // ── Round 31: flag-as-PM-name rejection ──
  // Note: --help, --detect, --list are checked BEFORE --global/--project in argv
  // parsing, so passing e.g. --global --list triggers the --list handler first.
  // The startsWith('-') fix protects against flags that AREN'T caught earlier,
  // like --global --project or --project --unknown-flag.
  describe('--global flag validation (Round 31)');

  await test('rejects --global --project (flag not caught by earlier checks)', () => {
    const result = run(['--global', '--project']);
    assert.strictEqual(result.code, 1);
    assert.ok(result.stderr.includes('requires a package manager name'));
  });

  await test('rejects --global --unknown-flag (arbitrary flag as PM name)', () => {
    const result = run(['--global', '--foo-bar']);
    assert.strictEqual(result.code, 1);
    assert.ok(result.stderr.includes('requires a package manager name'));
  });

  await test('rejects --global -x (single-dash flag as PM name)', () => {
    const result = run(['--global', '-x']);
    assert.strictEqual(result.code, 1);
    assert.ok(result.stderr.includes('requires a package manager name'));
  });

  await test('--global --list is handled by --list check first (exit 0)', () => {
    // --list is checked before --global in the parsing order
    const result = run(['--global', '--list']);
    assert.strictEqual(result.code, 0);
    assert.ok(result.stdout.includes('Available Package Managers'));
  });

  describe('--project flag validation (Round 31)');

  await test('rejects --project --global (cross-flag confusion)', () => {
    // --global handler runs before --project, catches it first
    const result = run(['--project', '--global']);
    assert.strictEqual(result.code, 1);
    assert.ok(result.stderr.includes('requires a package manager name'));
  });

  await test('rejects --project --unknown-flag', () => {
    const result = run(['--project', '--bar']);
    assert.strictEqual(result.code, 1);
    assert.ok(result.stderr.includes('requires a package manager name'));
  });

  await test('rejects --project -z (single-dash flag)', () => {
    const result = run(['--project', '-z']);
    assert.strictEqual(result.code, 1);
    assert.ok(result.stderr.includes('requires a package manager name'));
  });

  // ── Round 45: output completeness and marker uniqueness ──
  describe('--detect marker uniqueness (Round 45)');

  await test('--detect output shows exactly one (current) marker', () => {
    const result = run(['--detect']);
    assert.strictEqual(result.code, 0);
    const lines = result.stdout.split('\n');
    const currentLines = lines.filter(l => l.includes('(current)'));
    assert.strictEqual(currentLines.length, 1, `Expected exactly 1 "(current)" marker, found ${currentLines.length}`);
    // The (current) marker should be on a line with a PM name
    assert.ok(/\b(npm|pnpm|yarn|bun)\b/.test(currentLines[0]), 'Current marker should be on a PM line');
  });

  describe('--list output completeness (Round 45)');

  await test('--list shows all four supported package managers', () => {
    const result = run(['--list']);
    assert.strictEqual(result.code, 0);
    for (const pm of ['npm', 'pnpm', 'yarn', 'bun']) {
      assert.ok(result.stdout.includes(pm), `Should list ${pm}`);
    }
    // Each PM should show Lock file and Install info
    const lockFileCount = (result.stdout.match(/Lock file:/g) || []).length;
    assert.strictEqual(lockFileCount, 4, `Expected 4 "Lock file:" entries, found ${lockFileCount}`);
    const installCount = (result.stdout.match(/Install:/g) || []).length;
    assert.strictEqual(installCount, 4, `Expected 4 "Install:" entries, found ${installCount}`);
  });

  // ── Round 62: --global success path and bare PM name ──
  describe('--global success path (Round 62)');

  await test('--global npm writes config and succeeds', () => {
    const tmpDir = path.join(os.tmpdir(), `spm-test-global-${Date.now()}`);
    fs.mkdirSync(tmpDir, { recursive: true });
    try {
      const result = run(['--global', 'npm'], { HOME: tmpDir, USERPROFILE: tmpDir });
      assert.strictEqual(result.code, 0, `Expected exit 0, got ${result.code}. stderr: ${result.stderr}`);
      assert.ok(result.stdout.includes('Global preference set to'), 'Should show success message');
      assert.ok(result.stdout.includes('npm'), 'Should mention npm');
      // Verify config file was created
      const configPath = path.join(tmpDir, '.claude', 'package-manager.json');
      assert.ok(fs.existsSync(configPath), 'Config file should be created');
      const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
      assert.strictEqual(config.packageManager, 'npm', 'Config should contain npm');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  describe('bare PM name success (Round 62)');

  await test('bare npm sets global preference and succeeds', () => {
    const tmpDir = path.join(os.tmpdir(), `spm-test-bare-${Date.now()}`);
    fs.mkdirSync(tmpDir, { recursive: true });
    try {
      const result = run(['npm'], { HOME: tmpDir, USERPROFILE: tmpDir });
      assert.strictEqual(result.code, 0, `Expected exit 0, got ${result.code}. stderr: ${result.stderr}`);
      assert.ok(result.stdout.includes('Global preference set to'), 'Should show success message');
      // Verify config file was created
      const configPath = path.join(tmpDir, '.claude', 'package-manager.json');
      assert.ok(fs.existsSync(configPath), 'Config file should be created');
      const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
      assert.strictEqual(config.packageManager, 'npm', 'Config should contain npm');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  describe('--detect source label (Round 62)');

  await test('--detect with env var shows source as environment', () => {
    const result = run(['--detect'], { CLAUDE_PACKAGE_MANAGER: 'pnpm' });
    assert.strictEqual(result.code, 0);
    assert.ok(result.stdout.includes('Source: environment'), 'Should show environment as source');
  });

  // ── Round 68: --project success path and --list (current) marker ──
  describe('--project success path (Round 68)');

  await test('--project npm writes project config and succeeds', () => {
    const tmpDir = path.join(os.tmpdir(), `spm-test-project-${Date.now()}`);
    fs.mkdirSync(tmpDir, { recursive: true });
    try {
      const result = require('child_process').spawnSync('node', [SCRIPT, '--project', 'npm'], {
        encoding: 'utf8',
        stdio: ['pipe', 'pipe', 'pipe'],
        env: { ...process.env },
        timeout: 10000,
        cwd: tmpDir
      });
      assert.strictEqual(result.status, 0, `Expected exit 0, got ${result.status}. stderr: ${result.stderr}`);
      assert.ok(result.stdout.includes('Project preference set to'), 'Should show project success message');
      assert.ok(result.stdout.includes('npm'), 'Should mention npm');
      // Verify config file was created in the project CWD
      const configPath = path.join(tmpDir, '.claude', 'package-manager.json');
      assert.ok(fs.existsSync(configPath), 'Project config file should be created in CWD');
      const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
      assert.strictEqual(config.packageManager, 'npm', 'Config should contain npm');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  describe('--list (current) marker (Round 68)');

  await test('--list output includes (current) marker for active PM', () => {
    const result = run(['--list']);
    assert.strictEqual(result.code, 0);
    assert.ok(result.stdout.includes('(current)'), '--list should mark the active PM with (current)');
    // The (current) marker should appear exactly once
    const currentCount = (result.stdout.match(/\(current\)/g) || []).length;
    assert.strictEqual(currentCount, 1, `Expected exactly 1 "(current)" in --list, found ${currentCount}`);
  });

  // ── Round 74: setGlobal catch — setPreferredPackageManager throws ──
  describe('Round 74: setGlobal catch (save failure)');

  await test('--global npm fails when HOME is not a directory', () => {
    if (process.platform === 'win32') {
      console.log('    (skipped — /dev/null not available on Windows)');
      return;
    }
    // HOME=/dev/null causes ensureDir to throw ENOTDIR when creating ~/.claude/
    const result = run(['--global', 'npm'], { HOME: '/dev/null', USERPROFILE: '/dev/null' });
    assert.strictEqual(result.code, 1, `Expected exit 1, got ${result.code}`);
    assert.ok(result.stderr.includes('Error:'), `stderr should contain Error:, got: ${result.stderr}`);
  });

  // ── Round 74: setProject catch — setProjectPackageManager throws ──
  describe('Round 74: setProject catch (save failure)');

  await test('--project npm fails when CWD is read-only', () => {
    if (process.platform === 'win32' || process.getuid?.() === 0) {
      console.log('    (skipped — chmod ineffective on Windows/root)');
      return;
    }
    const tmpDir = path.join(os.tmpdir(), `spm-test-ro-${Date.now()}`);
    fs.mkdirSync(tmpDir, { recursive: true });
    try {
      // Make CWD read-only so .claude/ dir creation fails with EACCES
      fs.chmodSync(tmpDir, 0o555);
      const result = require('child_process').spawnSync('node', [SCRIPT, '--project', 'npm'], {
        encoding: 'utf8',
        stdio: ['pipe', 'pipe', 'pipe'],
        env: { ...process.env },
        timeout: 10000,
        cwd: tmpDir
      });
      assert.strictEqual(result.status, 1, `Expected exit 1, got ${result.status}. stderr: ${result.stderr}`);
      assert.ok(result.stderr.includes('Error:'), `stderr should contain Error:, got: ${result.stderr}`);
    } finally {
      try {
        fs.chmodSync(tmpDir, 0o755);
      } catch {
        /* best-effort */
      }
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
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
