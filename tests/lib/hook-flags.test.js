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

// Save original env values
const originalProfile = process.env.ECC_HOOK_PROFILE;
const originalDisabled = process.env.ECC_DISABLED_HOOKS;

function resetEnv() {
  if (originalProfile === undefined) delete process.env.ECC_HOOK_PROFILE;
  else process.env.ECC_HOOK_PROFILE = originalProfile;
  if (originalDisabled === undefined) delete process.env.ECC_DISABLED_HOOKS;
  else process.env.ECC_DISABLED_HOOKS = originalDisabled;
}

function runTests() {
  console.log('\n=== Testing hook-flags.ts ===\n');

  let passed = 0;
  let failed = 0;

  // --- normalizeId ---
  console.log('normalizeId:');

  if (
    test('normalizes string to lowercase trimmed', () => {
      assert.strictEqual(hookFlags.normalizeId('  Hello '), 'hello');
    })
  )
    passed++;
  else failed++;

  if (
    test('handles null/undefined', () => {
      assert.strictEqual(hookFlags.normalizeId(null), '');
      assert.strictEqual(hookFlags.normalizeId(undefined), '');
    })
  )
    passed++;
  else failed++;

  if (
    test('handles empty string', () => {
      assert.strictEqual(hookFlags.normalizeId(''), '');
    })
  )
    passed++;
  else failed++;

  if (
    test('handles number input', () => {
      assert.strictEqual(hookFlags.normalizeId(42), '42');
    })
  )
    passed++;
  else failed++;

  if (
    test('handles falsy input (false coerces to empty via ||)', () => {
      assert.strictEqual(hookFlags.normalizeId(false), '');
    })
  )
    passed++;
  else failed++;

  // --- getHookProfile ---
  console.log('\ngetHookProfile:');

  if (
    test('returns standard by default', () => {
      delete process.env.ECC_HOOK_PROFILE;
      assert.strictEqual(hookFlags.getHookProfile(), 'standard');
      resetEnv();
    })
  )
    passed++;
  else failed++;

  if (
    test('returns minimal when set', () => {
      process.env.ECC_HOOK_PROFILE = 'minimal';
      assert.strictEqual(hookFlags.getHookProfile(), 'minimal');
      resetEnv();
    })
  )
    passed++;
  else failed++;

  if (
    test('returns strict when set', () => {
      process.env.ECC_HOOK_PROFILE = 'strict';
      assert.strictEqual(hookFlags.getHookProfile(), 'strict');
      resetEnv();
    })
  )
    passed++;
  else failed++;

  if (
    test('returns standard for invalid profile', () => {
      process.env.ECC_HOOK_PROFILE = 'invalid';
      assert.strictEqual(hookFlags.getHookProfile(), 'standard');
      resetEnv();
    })
  )
    passed++;
  else failed++;

  if (
    test('trims and lowercases profile', () => {
      process.env.ECC_HOOK_PROFILE = '  STRICT  ';
      assert.strictEqual(hookFlags.getHookProfile(), 'strict');
      resetEnv();
    })
  )
    passed++;
  else failed++;

  if (
    test('returns standard for empty string', () => {
      process.env.ECC_HOOK_PROFILE = '';
      assert.strictEqual(hookFlags.getHookProfile(), 'standard');
      resetEnv();
    })
  )
    passed++;
  else failed++;

  // --- getDisabledHookIds ---
  console.log('\ngetDisabledHookIds:');

  if (
    test('returns empty set when not set', () => {
      delete process.env.ECC_DISABLED_HOOKS;
      const result = hookFlags.getDisabledHookIds();
      assert.strictEqual(result.size, 0);
      resetEnv();
    })
  )
    passed++;
  else failed++;

  if (
    test('returns empty set for empty string', () => {
      process.env.ECC_DISABLED_HOOKS = '';
      const result = hookFlags.getDisabledHookIds();
      assert.strictEqual(result.size, 0);
      resetEnv();
    })
  )
    passed++;
  else failed++;

  if (
    test('returns empty set for whitespace only', () => {
      process.env.ECC_DISABLED_HOOKS = '   ';
      const result = hookFlags.getDisabledHookIds();
      assert.strictEqual(result.size, 0);
      resetEnv();
    })
  )
    passed++;
  else failed++;

  if (
    test('parses single hook id', () => {
      process.env.ECC_DISABLED_HOOKS = 'pre:bash:tmux-reminder';
      const result = hookFlags.getDisabledHookIds();
      assert.strictEqual(result.size, 1);
      assert.ok(result.has('pre:bash:tmux-reminder'));
      resetEnv();
    })
  )
    passed++;
  else failed++;

  if (
    test('parses multiple comma-separated ids', () => {
      process.env.ECC_DISABLED_HOOKS = 'hook-a,hook-b,hook-c';
      const result = hookFlags.getDisabledHookIds();
      assert.strictEqual(result.size, 3);
      assert.ok(result.has('hook-a'));
      assert.ok(result.has('hook-b'));
      assert.ok(result.has('hook-c'));
      resetEnv();
    })
  )
    passed++;
  else failed++;

  if (
    test('normalizes ids (trim + lowercase)', () => {
      process.env.ECC_DISABLED_HOOKS = ' HOOK-A , Hook-B ';
      const result = hookFlags.getDisabledHookIds();
      assert.ok(result.has('hook-a'));
      assert.ok(result.has('hook-b'));
      resetEnv();
    })
  )
    passed++;
  else failed++;

  if (
    test('filters empty entries from trailing commas', () => {
      process.env.ECC_DISABLED_HOOKS = 'hook-a,,hook-b,';
      const result = hookFlags.getDisabledHookIds();
      assert.strictEqual(result.size, 2);
      resetEnv();
    })
  )
    passed++;
  else failed++;

  // --- parseProfiles ---
  console.log('\nparseProfiles:');

  if (
    test('returns fallback when null', () => {
      const result = hookFlags.parseProfiles(null);
      assert.deepStrictEqual(result, ['standard', 'strict']);
    })
  )
    passed++;
  else failed++;

  if (
    test('returns fallback when undefined', () => {
      const result = hookFlags.parseProfiles(undefined);
      assert.deepStrictEqual(result, ['standard', 'strict']);
    })
  )
    passed++;
  else failed++;

  if (
    test('returns custom fallback', () => {
      const result = hookFlags.parseProfiles(null, ['minimal']);
      assert.deepStrictEqual(result, ['minimal']);
    })
  )
    passed++;
  else failed++;

  if (
    test('parses comma-separated string', () => {
      const result = hookFlags.parseProfiles('minimal,standard');
      assert.deepStrictEqual(result, ['minimal', 'standard']);
    })
  )
    passed++;
  else failed++;

  if (
    test('parses single string', () => {
      const result = hookFlags.parseProfiles('strict');
      assert.deepStrictEqual(result, ['strict']);
    })
  )
    passed++;
  else failed++;

  if (
    test('filters invalid profiles from string', () => {
      const result = hookFlags.parseProfiles('minimal,invalid,strict');
      assert.deepStrictEqual(result, ['minimal', 'strict']);
    })
  )
    passed++;
  else failed++;

  if (
    test('returns fallback when all profiles invalid (string)', () => {
      const result = hookFlags.parseProfiles('foo,bar');
      assert.deepStrictEqual(result, ['standard', 'strict']);
    })
  )
    passed++;
  else failed++;

  if (
    test('parses array of profiles', () => {
      const result = hookFlags.parseProfiles(['minimal', 'standard']);
      assert.deepStrictEqual(result, ['minimal', 'standard']);
    })
  )
    passed++;
  else failed++;

  if (
    test('filters invalid profiles from array', () => {
      const result = hookFlags.parseProfiles(['minimal', 'nope', 'strict']);
      assert.deepStrictEqual(result, ['minimal', 'strict']);
    })
  )
    passed++;
  else failed++;

  if (
    test('returns fallback when all profiles invalid (array)', () => {
      const result = hookFlags.parseProfiles(['foo', 'bar']);
      assert.deepStrictEqual(result, ['standard', 'strict']);
    })
  )
    passed++;
  else failed++;

  if (
    test('handles empty string', () => {
      const result = hookFlags.parseProfiles('');
      assert.deepStrictEqual(result, ['standard', 'strict']);
    })
  )
    passed++;
  else failed++;

  if (
    test('handles empty array', () => {
      const result = hookFlags.parseProfiles([]);
      assert.deepStrictEqual(result, ['standard', 'strict']);
    })
  )
    passed++;
  else failed++;

  if (
    test('trims and lowercases string entries', () => {
      const result = hookFlags.parseProfiles(' MINIMAL , STRICT ');
      assert.deepStrictEqual(result, ['minimal', 'strict']);
    })
  )
    passed++;
  else failed++;

  if (
    test('trims and lowercases array entries', () => {
      const result = hookFlags.parseProfiles([' MINIMAL ', ' STRICT ']);
      assert.deepStrictEqual(result, ['minimal', 'strict']);
    })
  )
    passed++;
  else failed++;

  // --- isHookEnabled ---
  console.log('\nisHookEnabled:');

  if (
    test('returns true when hook matches current profile', () => {
      delete process.env.ECC_DISABLED_HOOKS;
      process.env.ECC_HOOK_PROFILE = 'standard';
      assert.strictEqual(hookFlags.isHookEnabled('test-hook', { profiles: 'standard,strict' }), true);
      resetEnv();
    })
  )
    passed++;
  else failed++;

  if (
    test('returns false when hook does not match profile', () => {
      delete process.env.ECC_DISABLED_HOOKS;
      process.env.ECC_HOOK_PROFILE = 'minimal';
      assert.strictEqual(hookFlags.isHookEnabled('test-hook', { profiles: 'standard,strict' }), false);
      resetEnv();
    })
  )
    passed++;
  else failed++;

  if (
    test('returns false when hook is disabled', () => {
      process.env.ECC_HOOK_PROFILE = 'standard';
      process.env.ECC_DISABLED_HOOKS = 'test-hook';
      assert.strictEqual(hookFlags.isHookEnabled('test-hook', { profiles: 'standard,strict' }), false);
      resetEnv();
    })
  )
    passed++;
  else failed++;

  if (
    test('disabled check is case-insensitive', () => {
      process.env.ECC_HOOK_PROFILE = 'standard';
      process.env.ECC_DISABLED_HOOKS = 'TEST-HOOK';
      assert.strictEqual(hookFlags.isHookEnabled('test-hook'), false);
      resetEnv();
    })
  )
    passed++;
  else failed++;

  if (
    test('returns true for empty hookId', () => {
      delete process.env.ECC_DISABLED_HOOKS;
      process.env.ECC_HOOK_PROFILE = 'standard';
      assert.strictEqual(hookFlags.isHookEnabled(''), true);
      resetEnv();
    })
  )
    passed++;
  else failed++;

  if (
    test('uses default profiles (standard,strict) when not specified', () => {
      delete process.env.ECC_DISABLED_HOOKS;
      process.env.ECC_HOOK_PROFILE = 'standard';
      assert.strictEqual(hookFlags.isHookEnabled('test-hook'), true);
      resetEnv();
    })
  )
    passed++;
  else failed++;

  if (
    test('minimal profile not in default allowed profiles', () => {
      delete process.env.ECC_DISABLED_HOOKS;
      process.env.ECC_HOOK_PROFILE = 'minimal';
      assert.strictEqual(hookFlags.isHookEnabled('test-hook'), false);
      resetEnv();
    })
  )
    passed++;
  else failed++;

  if (
    test('allows minimal when explicitly in profiles', () => {
      delete process.env.ECC_DISABLED_HOOKS;
      process.env.ECC_HOOK_PROFILE = 'minimal';
      assert.strictEqual(hookFlags.isHookEnabled('test-hook', { profiles: 'minimal,standard,strict' }), true);
      resetEnv();
    })
  )
    passed++;
  else failed++;

  // --- VALID_PROFILES ---
  console.log('\nVALID_PROFILES:');

  if (
    test('contains exactly minimal, standard, strict', () => {
      assert.strictEqual(hookFlags.VALID_PROFILES.size, 3);
      assert.ok(hookFlags.VALID_PROFILES.has('minimal'));
      assert.ok(hookFlags.VALID_PROFILES.has('standard'));
      assert.ok(hookFlags.VALID_PROFILES.has('strict'));
    })
  )
    passed++;
  else failed++;

  console.log(`\n=== Results: ${passed} passed, ${failed} failed ===\n`);
  process.exit(failed > 0 ? 1 : 0);
}

runTests();
