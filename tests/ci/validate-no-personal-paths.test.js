/**
 * Tests for src/ci/validate-no-personal-paths.ts
 *
 * Tests the personal path detection CI validator.
 *
 * Run with: node tests/ci/validate-no-personal-paths.test.js
 */

const assert = require('assert');
const path = require('path');
const fs = require('fs');
const { spawnSync } = require('child_process');

const validatorScript = path.join(__dirname, '..', '..', 'dist', 'ci', 'validate-no-personal-paths.js');
const srcPath = path.join(__dirname, '..', '..', 'src', 'ci', 'validate-no-personal-paths.ts');

function test(name, fn) {
  try {
    fn();
    console.log(` \u2713 ${name}`);
    return true;
  } catch (err) {
    console.log(` \u2717 ${name}`);
    console.log(`   Error: ${err.message}`);
    return false;
  }
}

function runValidator() {
  const result = spawnSync('node', [validatorScript], {
    encoding: 'utf8',
    cwd: path.join(__dirname, '..', '..'),
    timeout: 15000
  });
  return {
    code: result.status,
    stdout: result.stdout || '',
    stderr: result.stderr || ''
  };
}

function runTests() {
  console.log('\n=== Testing validate-no-personal-paths.ts ===\n');

  let passed = 0;
  let failed = 0;

  console.log('Script structure:');

  if (test('compiled script exists', () => {
    assert.ok(fs.existsSync(validatorScript), `Script not found at ${validatorScript}`);
  })) passed++; else failed++;

  if (test('source file exists', () => {
    assert.ok(fs.existsSync(srcPath), `Source not found at ${srcPath}`);
  })) passed++; else failed++;

  // Read source for structural validation
  const src = fs.readFileSync(srcPath, 'utf8');

  if (test('defines BLOCK_PATTERNS array', () => {
    assert.ok(src.includes('BLOCK_PATTERNS'), 'Should define BLOCK_PATTERNS');
  })) passed++; else failed++;

  if (test('blocks affoon unix path pattern', () => {
    assert.ok(src.includes('affoon'), 'Should contain affoon username');
    assert.ok(src.includes('Users'), 'Should reference Users directory');
  })) passed++; else failed++;

  if (test('blocks affoon windows path pattern (case-insensitive)', () => {
    assert.ok(src.includes('C:\\\\Users\\\\affoon'), 'Should block Windows path');
  })) passed++; else failed++;

  if (test('scans expected target directories', () => {
    assert.ok(src.includes("'README.md'"), 'Should scan README.md');
    assert.ok(src.includes("'skills'"), 'Should scan skills');
    assert.ok(src.includes("'commands'"), 'Should scan commands');
    assert.ok(src.includes("'agents'"), 'Should scan agents');
    assert.ok(src.includes("'docs'"), 'Should scan docs');
  })) passed++; else failed++;

  if (test('scans relevant file extensions', () => {
    // Extensions are in a regex: /\.(md|json|js|ts|sh|toml|yml|yaml)$/i
    assert.ok(src.includes('md|json|js|ts|sh'), 'Should include standard extensions');
  })) passed++; else failed++;

  if (test('skips node_modules and .git', () => {
    assert.ok(src.includes("'node_modules'"), 'Should skip node_modules');
    assert.ok(src.includes("'.git'"), 'Should skip .git');
  })) passed++; else failed++;

  if (test('exits with code 1 on failure', () => {
    assert.ok(src.includes('process.exit(1)'), 'Should exit 1 on detection');
  })) passed++; else failed++;

  if (test('outputs error message on detection', () => {
    assert.ok(src.includes('ERROR: personal path detected'), 'Should report detected paths');
  })) passed++; else failed++;

  if (test('outputs success message when clean', () => {
    assert.ok(src.includes('no personal absolute paths'), 'Should confirm when clean');
  })) passed++; else failed++;

  console.log('\nRuntime behavior:');

  if (test('runs without crashing', () => {
    const result = runValidator();
    assert.ok(result.code === 0 || result.code === 1, `Unexpected exit code: ${result.code}`);
  })) passed++; else failed++;

  if (test('detects known personal path in skills/continuous-learning/SKILL.md', () => {
    // The repo currently has a personal path in this file
    const result = runValidator();
    assert.strictEqual(result.code, 1, 'Should exit 1 when personal paths are found');
    assert.ok(result.stderr.includes('personal path detected'), 'Should report the detection');
    assert.ok(result.stderr.includes('continuous-learning'), 'Should identify the offending file');
  })) passed++; else failed++;

  if (test('collectFiles recursively scans directories', () => {
    assert.ok(src.includes('collectFiles'), 'Should define collectFiles function');
    assert.ok(src.includes('isFile'), 'Should check if entry is file');
    assert.ok(src.includes('readdirSync'), 'Should read directory entries');
  })) passed++; else failed++;

  console.log(`\n=== Results: ${passed} passed, ${failed} failed ===\n`);
  process.exit(failed > 0 ? 1 : 0);
}

runTests();
