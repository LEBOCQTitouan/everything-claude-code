/**
 * Tests for src/lib/hook-flags.ts
 *
 * Tests normalizeId, getHookProfile, getDisabledHookIds, parseProfiles, isHookEnabled.
 *
 * Run with: node tests/lib/hook-flags.test.js
 */

const assert = require('assert');

// Import the compiled module
const hookFlags = require('../../dist/lib/hook-flags');
const { test, describe } = require('../harness');

// Save original env values
const originalProfile = process.env.ECC_HOOK_PROFILE;
const originalDisabled = process.env.ECC_DISABLED_HOOKS;

function resetEnv() {
  if (originalProfile === undefined) delete process.env.ECC_HOOK_PROFILE;
  else process.env.ECC_HOOK_PROFILE = originalProfile;
  if (originalDisabled === undefined) delete process.env.ECC_DISABLED_HOOKS;
  else process.env.ECC_DISABLED_HOOKS = originalDisabled;
}

async function runTests() {
  describe('Testing hook-flags.ts');


  // --- normalizeId ---
  describe('normalizeId');


  await test('normalizes string to lowercase trimmed', () => {
      assert.strictEqual(hookFlags.normalizeId('  Hello '), 'hello');
    });

  await test('handles null/undefined', () => {
      assert.strictEqual(hookFlags.normalizeId(null), '');
      assert.strictEqual(hookFlags.normalizeId(undefined), '');
    });

  await test('handles empty string', () => {
      assert.strictEqual(hookFlags.normalizeId(''), '');
    });

  await test('handles number input', () => {
      assert.strictEqual(hookFlags.normalizeId(42), '42');
    });

  await test('handles falsy input (false coerces to empty via ||)', () => {
      assert.strictEqual(hookFlags.normalizeId(false), '');
    });

  // --- getHookProfile ---
  describe('getHookProfile');


  await test('returns standard by default', () => {
      delete process.env.ECC_HOOK_PROFILE;
      assert.strictEqual(hookFlags.getHookProfile(), 'standard');
      resetEnv();
    });

  await test('returns minimal when set', () => {
      process.env.ECC_HOOK_PROFILE = 'minimal';
      assert.strictEqual(hookFlags.getHookProfile(), 'minimal');
      resetEnv();
    });

  await test('returns strict when set', () => {
      process.env.ECC_HOOK_PROFILE = 'strict';
      assert.strictEqual(hookFlags.getHookProfile(), 'strict');
      resetEnv();
    });

  await test('returns standard for invalid profile', () => {
      process.env.ECC_HOOK_PROFILE = 'invalid';
      assert.strictEqual(hookFlags.getHookProfile(), 'standard');
      resetEnv();
    });

  await test('trims and lowercases profile', () => {
      process.env.ECC_HOOK_PROFILE = '  STRICT  ';
      assert.strictEqual(hookFlags.getHookProfile(), 'strict');
      resetEnv();
    });

  await test('returns standard for empty string', () => {
      process.env.ECC_HOOK_PROFILE = '';
      assert.strictEqual(hookFlags.getHookProfile(), 'standard');
      resetEnv();
    });

  // --- getDisabledHookIds ---
  describe('getDisabledHookIds');


  await test('returns empty set when not set', () => {
      delete process.env.ECC_DISABLED_HOOKS;
      const result = hookFlags.getDisabledHookIds();
      assert.strictEqual(result.size, 0);
      resetEnv();
    });

  await test('returns empty set for empty string', () => {
      process.env.ECC_DISABLED_HOOKS = '';
      const result = hookFlags.getDisabledHookIds();
      assert.strictEqual(result.size, 0);
      resetEnv();
    });

  await test('returns empty set for whitespace only', () => {
      process.env.ECC_DISABLED_HOOKS = '   ';
      const result = hookFlags.getDisabledHookIds();
      assert.strictEqual(result.size, 0);
      resetEnv();
    });

  await test('parses single hook id', () => {
      process.env.ECC_DISABLED_HOOKS = 'pre:bash:tmux-reminder';
      const result = hookFlags.getDisabledHookIds();
      assert.strictEqual(result.size, 1);
      assert.ok(result.has('pre:bash:tmux-reminder'));
      resetEnv();
    });

  await test('parses multiple comma-separated ids', () => {
      process.env.ECC_DISABLED_HOOKS = 'hook-a,hook-b,hook-c';
      const result = hookFlags.getDisabledHookIds();
      assert.strictEqual(result.size, 3);
      assert.ok(result.has('hook-a'));
      assert.ok(result.has('hook-b'));
      assert.ok(result.has('hook-c'));
      resetEnv();
    });

  await test('normalizes ids (trim + lowercase)', () => {
      process.env.ECC_DISABLED_HOOKS = ' HOOK-A , Hook-B ';
      const result = hookFlags.getDisabledHookIds();
      assert.ok(result.has('hook-a'));
      assert.ok(result.has('hook-b'));
      resetEnv();
    });

  await test('filters empty entries from trailing commas', () => {
      process.env.ECC_DISABLED_HOOKS = 'hook-a,,hook-b,';
      const result = hookFlags.getDisabledHookIds();
      assert.strictEqual(result.size, 2);
      resetEnv();
    });

  // --- parseProfiles ---
  describe('parseProfiles');


  await test('returns fallback when null', () => {
      const result = hookFlags.parseProfiles(null);
      assert.deepStrictEqual(result, ['standard', 'strict']);
    });

  await test('returns fallback when undefined', () => {
      const result = hookFlags.parseProfiles(undefined);
      assert.deepStrictEqual(result, ['standard', 'strict']);
    });

  await test('returns custom fallback', () => {
      const result = hookFlags.parseProfiles(null, ['minimal']);
      assert.deepStrictEqual(result, ['minimal']);
    });

  await test('parses comma-separated string', () => {
      const result = hookFlags.parseProfiles('minimal,standard');
      assert.deepStrictEqual(result, ['minimal', 'standard']);
    });

  await test('parses single string', () => {
      const result = hookFlags.parseProfiles('strict');
      assert.deepStrictEqual(result, ['strict']);
    });

  await test('filters invalid profiles from string', () => {
      const result = hookFlags.parseProfiles('minimal,invalid,strict');
      assert.deepStrictEqual(result, ['minimal', 'strict']);
    });

  await test('returns fallback when all profiles invalid (string)', () => {
      const result = hookFlags.parseProfiles('foo,bar');
      assert.deepStrictEqual(result, ['standard', 'strict']);
    });

  await test('parses array of profiles', () => {
      const result = hookFlags.parseProfiles(['minimal', 'standard']);
      assert.deepStrictEqual(result, ['minimal', 'standard']);
    });

  await test('filters invalid profiles from array', () => {
      const result = hookFlags.parseProfiles(['minimal', 'nope', 'strict']);
      assert.deepStrictEqual(result, ['minimal', 'strict']);
    });

  await test('returns fallback when all profiles invalid (array)', () => {
      const result = hookFlags.parseProfiles(['foo', 'bar']);
      assert.deepStrictEqual(result, ['standard', 'strict']);
    });

  await test('handles empty string', () => {
      const result = hookFlags.parseProfiles('');
      assert.deepStrictEqual(result, ['standard', 'strict']);
    });

  await test('handles empty array', () => {
      const result = hookFlags.parseProfiles([]);
      assert.deepStrictEqual(result, ['standard', 'strict']);
    });

  await test('trims and lowercases string entries', () => {
      const result = hookFlags.parseProfiles(' MINIMAL , STRICT ');
      assert.deepStrictEqual(result, ['minimal', 'strict']);
    });

  await test('trims and lowercases array entries', () => {
      const result = hookFlags.parseProfiles([' MINIMAL ', ' STRICT ']);
      assert.deepStrictEqual(result, ['minimal', 'strict']);
    });

  // --- isHookEnabled ---
  describe('isHookEnabled');


  await test('returns true when hook matches current profile', () => {
      delete process.env.ECC_DISABLED_HOOKS;
      process.env.ECC_HOOK_PROFILE = 'standard';
      assert.strictEqual(hookFlags.isHookEnabled('test-hook', { profiles: 'standard,strict' }), true);
      resetEnv();
    });

  await test('returns false when hook does not match profile', () => {
      delete process.env.ECC_DISABLED_HOOKS;
      process.env.ECC_HOOK_PROFILE = 'minimal';
      assert.strictEqual(hookFlags.isHookEnabled('test-hook', { profiles: 'standard,strict' }), false);
      resetEnv();
    });

  await test('returns false when hook is disabled', () => {
      process.env.ECC_HOOK_PROFILE = 'standard';
      process.env.ECC_DISABLED_HOOKS = 'test-hook';
      assert.strictEqual(hookFlags.isHookEnabled('test-hook', { profiles: 'standard,strict' }), false);
      resetEnv();
    });

  await test('disabled check is case-insensitive', () => {
      process.env.ECC_HOOK_PROFILE = 'standard';
      process.env.ECC_DISABLED_HOOKS = 'TEST-HOOK';
      assert.strictEqual(hookFlags.isHookEnabled('test-hook'), false);
      resetEnv();
    });

  await test('returns true for empty hookId', () => {
      delete process.env.ECC_DISABLED_HOOKS;
      process.env.ECC_HOOK_PROFILE = 'standard';
      assert.strictEqual(hookFlags.isHookEnabled(''), true);
      resetEnv();
    });

  await test('uses default profiles (standard,strict) when not specified', () => {
      delete process.env.ECC_DISABLED_HOOKS;
      process.env.ECC_HOOK_PROFILE = 'standard';
      assert.strictEqual(hookFlags.isHookEnabled('test-hook'), true);
      resetEnv();
    });

  await test('minimal profile not in default allowed profiles', () => {
      delete process.env.ECC_DISABLED_HOOKS;
      process.env.ECC_HOOK_PROFILE = 'minimal';
      assert.strictEqual(hookFlags.isHookEnabled('test-hook'), false);
      resetEnv();
    });

  await test('allows minimal when explicitly in profiles', () => {
      delete process.env.ECC_DISABLED_HOOKS;
      process.env.ECC_HOOK_PROFILE = 'minimal';
      assert.strictEqual(hookFlags.isHookEnabled('test-hook', { profiles: 'minimal,standard,strict' }), true);
      resetEnv();
    });

  // --- VALID_PROFILES ---
  describe('VALID_PROFILES');


  await test('contains exactly minimal, standard, strict', () => {
      assert.strictEqual(hookFlags.VALID_PROFILES.size, 3);
      assert.ok(hookFlags.VALID_PROFILES.has('minimal'));
      assert.ok(hookFlags.VALID_PROFILES.has('standard'));
      assert.ok(hookFlags.VALID_PROFILES.has('strict'));
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
