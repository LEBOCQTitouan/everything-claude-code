/**
 * Tests for scripts/lib/project-detect.js
 *
 * Run with: node tests/lib/project-detect.test.js
 */

const assert = require('assert');
const path = require('path');
const fs = require('fs');
const os = require('os');

const { detectProjectType, LANGUAGE_RULES, FRAMEWORK_RULES, getPackageJsonDeps, getPythonDeps, getGoDeps, getRustDeps, getComposerDeps, getElixirDeps } = require('../../src/lib/project-detect');
const { test, describe } = require('../harness');

// Create a temporary directory for testing
function createTempDir() {
  return fs.mkdtempSync(path.join(os.tmpdir(), 'ecc-test-'));
}

// Clean up temp directory
function cleanupDir(dir) {
  try {
    fs.rmSync(dir, { recursive: true, force: true });
  } catch {
    /* ignore */
  }
}

// Write a file in the temp directory
function writeTestFile(dir, filePath, content = '') {
  const fullPath = path.join(dir, filePath);
  const dirName = path.dirname(fullPath);
  fs.mkdirSync(dirName, { recursive: true });
  fs.writeFileSync(fullPath, content, 'utf8');
}

async function runTests() {
  describe('Testing project-detect.js');

  // Rule definitions tests
  describe('Rule Definitions');

  await test('LANGUAGE_RULES is non-empty array', () => {
    assert.ok(Array.isArray(LANGUAGE_RULES));
    assert.ok(LANGUAGE_RULES.length > 0);
  });

  await test('FRAMEWORK_RULES is non-empty array', () => {
    assert.ok(Array.isArray(FRAMEWORK_RULES));
    assert.ok(FRAMEWORK_RULES.length > 0);
  });

  await test('each language rule has type, markers, and extensions', () => {
    for (const rule of LANGUAGE_RULES) {
      assert.ok(typeof rule.type === 'string', `Missing type`);
      assert.ok(Array.isArray(rule.markers), `Missing markers for ${rule.type}`);
      assert.ok(Array.isArray(rule.extensions), `Missing extensions for ${rule.type}`);
    }
  });

  await test('each framework rule has framework, language, markers, packageKeys', () => {
    for (const rule of FRAMEWORK_RULES) {
      assert.ok(typeof rule.framework === 'string', `Missing framework`);
      assert.ok(typeof rule.language === 'string', `Missing language for ${rule.framework}`);
      assert.ok(Array.isArray(rule.markers), `Missing markers for ${rule.framework}`);
      assert.ok(Array.isArray(rule.packageKeys), `Missing packageKeys for ${rule.framework}`);
    }
  });

  // Empty directory detection
  describe('Empty Directory');

  await test('empty directory returns unknown primary', () => {
    const dir = createTempDir();
    try {
      const result = detectProjectType(dir);
      assert.strictEqual(result.primary, 'unknown');
      assert.deepStrictEqual(result.languages, []);
      assert.deepStrictEqual(result.frameworks, []);
      assert.strictEqual(result.projectDir, dir);
    } finally {
      cleanupDir(dir);
    }
  });

  // Python detection
  describe('Python Detection');

  await test('detects python from requirements.txt', () => {
    const dir = createTempDir();
    try {
      writeTestFile(dir, 'requirements.txt', 'flask==3.0.0\nrequests>=2.31');
      const result = detectProjectType(dir);
      assert.ok(result.languages.includes('python'));
    } finally {
      cleanupDir(dir);
    }
  });

  await test('detects python from pyproject.toml', () => {
    const dir = createTempDir();
    try {
      writeTestFile(dir, 'pyproject.toml', '[project]\nname = "test"');
      const result = detectProjectType(dir);
      assert.ok(result.languages.includes('python'));
    } finally {
      cleanupDir(dir);
    }
  });

  await test('detects flask framework from requirements.txt', () => {
    const dir = createTempDir();
    try {
      writeTestFile(dir, 'requirements.txt', 'flask==3.0.0\nrequests>=2.31');
      const result = detectProjectType(dir);
      assert.ok(result.frameworks.includes('flask'));
    } finally {
      cleanupDir(dir);
    }
  });

  await test('detects django framework from manage.py', () => {
    const dir = createTempDir();
    try {
      writeTestFile(dir, 'manage.py', '#!/usr/bin/env python');
      writeTestFile(dir, 'requirements.txt', 'django>=4.2');
      const result = detectProjectType(dir);
      assert.ok(result.frameworks.includes('django'));
    } finally {
      cleanupDir(dir);
    }
  });

  await test('detects fastapi from pyproject.toml dependencies', () => {
    const dir = createTempDir();
    try {
      writeTestFile(dir, 'pyproject.toml', '[project]\nname = "test"\ndependencies = [\n  "fastapi>=0.100",\n  "uvicorn"\n]');
      const result = detectProjectType(dir);
      assert.ok(result.frameworks.includes('fastapi'));
    } finally {
      cleanupDir(dir);
    }
  });

  // TypeScript/JavaScript detection
  describe('TypeScript/JavaScript Detection');

  await test('detects typescript from tsconfig.json', () => {
    const dir = createTempDir();
    try {
      writeTestFile(dir, 'tsconfig.json', '{}');
      writeTestFile(dir, 'package.json', '{"dependencies":{}}');
      const result = detectProjectType(dir);
      assert.ok(result.languages.includes('typescript'));
      // Should NOT also include javascript when TS is detected
      assert.ok(!result.languages.includes('javascript'));
    } finally {
      cleanupDir(dir);
    }
  });

  await test('detects nextjs from next.config.mjs', () => {
    const dir = createTempDir();
    try {
      writeTestFile(dir, 'tsconfig.json', '{}');
      writeTestFile(dir, 'next.config.mjs', 'export default {}');
      writeTestFile(dir, 'package.json', '{"dependencies":{"next":"14.0.0","react":"18.0.0"}}');
      const result = detectProjectType(dir);
      assert.ok(result.frameworks.includes('nextjs'));
      assert.ok(result.frameworks.includes('react'));
    } finally {
      cleanupDir(dir);
    }
  });

  await test('detects react from package.json', () => {
    const dir = createTempDir();
    try {
      writeTestFile(dir, 'package.json', '{"dependencies":{"react":"18.0.0","react-dom":"18.0.0"}}');
      const result = detectProjectType(dir);
      assert.ok(result.frameworks.includes('react'));
    } finally {
      cleanupDir(dir);
    }
  });

  await test('detects angular from angular.json', () => {
    const dir = createTempDir();
    try {
      writeTestFile(dir, 'angular.json', '{}');
      writeTestFile(dir, 'tsconfig.json', '{}');
      writeTestFile(dir, 'package.json', '{"dependencies":{"@angular/core":"17.0.0"}}');
      const result = detectProjectType(dir);
      assert.ok(result.frameworks.includes('angular'));
    } finally {
      cleanupDir(dir);
    }
  });

  // Go detection
  describe('Go Detection');

  await test('detects golang from go.mod', () => {
    const dir = createTempDir();
    try {
      writeTestFile(dir, 'go.mod', 'module github.com/test/app\n\ngo 1.22\n\nrequire (\n\tgithub.com/gin-gonic/gin v1.9.1\n)');
      const result = detectProjectType(dir);
      assert.ok(result.languages.includes('golang'));
      assert.ok(result.frameworks.includes('gin'));
    } finally {
      cleanupDir(dir);
    }
  });

  // Rust detection
  describe('Rust Detection');

  await test('detects rust from Cargo.toml', () => {
    const dir = createTempDir();
    try {
      writeTestFile(dir, 'Cargo.toml', '[package]\nname = "test"\n\n[dependencies]\naxum = "0.7"');
      const result = detectProjectType(dir);
      assert.ok(result.languages.includes('rust'));
      assert.ok(result.frameworks.includes('axum'));
    } finally {
      cleanupDir(dir);
    }
  });

  // Ruby detection
  describe('Ruby Detection');

  await test('detects ruby and rails', () => {
    const dir = createTempDir();
    try {
      writeTestFile(dir, 'Gemfile', 'source "https://rubygems.org"\ngem "rails"');
      writeTestFile(dir, 'config/routes.rb', 'Rails.application.routes.draw do\nend');
      const result = detectProjectType(dir);
      assert.ok(result.languages.includes('ruby'));
      assert.ok(result.frameworks.includes('rails'));
    } finally {
      cleanupDir(dir);
    }
  });

  // PHP detection
  describe('PHP Detection');

  await test('detects php and laravel', () => {
    const dir = createTempDir();
    try {
      writeTestFile(dir, 'composer.json', '{"require":{"laravel/framework":"^10.0"}}');
      writeTestFile(dir, 'artisan', '#!/usr/bin/env php');
      const result = detectProjectType(dir);
      assert.ok(result.languages.includes('php'));
      assert.ok(result.frameworks.includes('laravel'));
    } finally {
      cleanupDir(dir);
    }
  });

  // Fullstack detection
  describe('Fullstack Detection');

  await test('detects fullstack when frontend + backend frameworks present', () => {
    const dir = createTempDir();
    try {
      writeTestFile(dir, 'package.json', '{"dependencies":{"react":"18.0.0","express":"4.18.0"}}');
      const result = detectProjectType(dir);
      assert.ok(result.frameworks.includes('react'));
      assert.ok(result.frameworks.includes('express'));
      assert.strictEqual(result.primary, 'fullstack');
    } finally {
      cleanupDir(dir);
    }
  });

  // Dependency reader tests
  describe('Dependency Readers');

  await test('getPackageJsonDeps reads deps and devDeps', () => {
    const dir = createTempDir();
    try {
      writeTestFile(dir, 'package.json', '{"dependencies":{"react":"18.0.0"},"devDependencies":{"typescript":"5.0.0"}}');
      const deps = getPackageJsonDeps(dir);
      assert.ok(deps.includes('react'));
      assert.ok(deps.includes('typescript'));
    } finally {
      cleanupDir(dir);
    }
  });

  await test('getPythonDeps reads requirements.txt', () => {
    const dir = createTempDir();
    try {
      writeTestFile(dir, 'requirements.txt', 'flask>=3.0\n# comment\nrequests==2.31\n-r other.txt');
      const deps = getPythonDeps(dir);
      assert.ok(deps.includes('flask'));
      assert.ok(deps.includes('requests'));
      assert.ok(!deps.includes('-r'));
    } finally {
      cleanupDir(dir);
    }
  });

  await test('getGoDeps reads go.mod require block', () => {
    const dir = createTempDir();
    try {
      writeTestFile(dir, 'go.mod', 'module test\n\ngo 1.22\n\nrequire (\n\tgithub.com/gin-gonic/gin v1.9.1\n\tgithub.com/lib/pq v1.10.9\n)');
      const deps = getGoDeps(dir);
      assert.ok(deps.some(d => d.includes('gin-gonic/gin')));
      assert.ok(deps.some(d => d.includes('lib/pq')));
    } finally {
      cleanupDir(dir);
    }
  });

  await test('getRustDeps reads Cargo.toml', () => {
    const dir = createTempDir();
    try {
      writeTestFile(dir, 'Cargo.toml', '[package]\nname = "test"\n\n[dependencies]\nserde = "1.0"\ntokio = { version = "1.0", features = ["full"] }');
      const deps = getRustDeps(dir);
      assert.ok(deps.includes('serde'));
      assert.ok(deps.includes('tokio'));
    } finally {
      cleanupDir(dir);
    }
  });

  await test('returns empty arrays for missing files', () => {
    const dir = createTempDir();
    try {
      assert.deepStrictEqual(getPackageJsonDeps(dir), []);
      assert.deepStrictEqual(getPythonDeps(dir), []);
      assert.deepStrictEqual(getGoDeps(dir), []);
      assert.deepStrictEqual(getRustDeps(dir), []);
      assert.deepStrictEqual(getComposerDeps(dir), []);
      assert.deepStrictEqual(getElixirDeps(dir), []);
    } finally {
      cleanupDir(dir);
    }
  });

  // Elixir detection
  describe('Elixir Detection');

  await test('detects elixir from mix.exs', () => {
    const dir = createTempDir();
    try {
      writeTestFile(dir, 'mix.exs', 'defmodule Test.MixProject do\n  defp deps do\n    [{:phoenix, "~> 1.7"},\n     {:ecto, "~> 3.0"}]\n  end\nend');
      const result = detectProjectType(dir);
      assert.ok(result.languages.includes('elixir'));
      assert.ok(result.frameworks.includes('phoenix'));
    } finally {
      cleanupDir(dir);
    }
  });

  // Edge cases
  describe('Edge Cases');

  await test('handles non-existent directory gracefully', () => {
    const result = detectProjectType('/tmp/nonexistent-dir-' + Date.now());
    assert.strictEqual(result.primary, 'unknown');
    assert.deepStrictEqual(result.languages, []);
  });

  await test('handles malformed package.json', () => {
    const dir = createTempDir();
    try {
      writeTestFile(dir, 'package.json', 'not valid json{{{');
      const deps = getPackageJsonDeps(dir);
      assert.deepStrictEqual(deps, []);
    } finally {
      cleanupDir(dir);
    }
  });

  await test('handles malformed composer.json', () => {
    const dir = createTempDir();
    try {
      writeTestFile(dir, 'composer.json', '{invalid');
      const deps = getComposerDeps(dir);
      assert.deepStrictEqual(deps, []);
    } finally {
      cleanupDir(dir);
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
