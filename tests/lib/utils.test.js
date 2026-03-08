/**
 * Tests for scripts/lib/utils.js
 *
 * Run with: node tests/lib/utils.test.js
 */

const assert = require('assert');
const path = require('path');
const fs = require('fs');

// Import the module
const utils = require('../../src/lib/utils');
const { test, describe } = require('../harness');

// Test suite
async function runTests() {
  describe('Testing utils.js');

  // Platform detection tests
  describe('Platform Detection');

  await test('isWindows/isMacOS/isLinux are booleans', () => {
    assert.strictEqual(typeof utils.isWindows, 'boolean');
    assert.strictEqual(typeof utils.isMacOS, 'boolean');
    assert.strictEqual(typeof utils.isLinux, 'boolean');
  });

  await test('exactly one platform should be true', () => {
    const platforms = [utils.isWindows, utils.isMacOS, utils.isLinux];
    const trueCount = platforms.filter(p => p).length;
    // Note: Could be 0 on other platforms like FreeBSD
    assert.ok(trueCount <= 1, 'More than one platform is true');
  });

  // Directory functions tests
  describe('Directory Functions');

  await test('getHomeDir returns valid path', () => {
    const home = utils.getHomeDir();
    assert.strictEqual(typeof home, 'string');
    assert.ok(home.length > 0, 'Home dir should not be empty');
    assert.ok(fs.existsSync(home), 'Home dir should exist');
  });

  await test('getClaudeDir returns path under home', () => {
    const claudeDir = utils.getClaudeDir();
    const homeDir = utils.getHomeDir();
    assert.ok(claudeDir.startsWith(homeDir), 'Claude dir should be under home');
    assert.ok(claudeDir.includes('.claude'), 'Should contain .claude');
  });

  await test('getSessionsDir returns path under Claude dir', () => {
    const sessionsDir = utils.getSessionsDir();
    const claudeDir = utils.getClaudeDir();
    assert.ok(sessionsDir.startsWith(claudeDir), 'Sessions should be under Claude dir');
    assert.ok(sessionsDir.includes('sessions'), 'Should contain sessions');
  });

  await test('getTempDir returns valid temp directory', () => {
    const tempDir = utils.getTempDir();
    assert.strictEqual(typeof tempDir, 'string');
    assert.ok(tempDir.length > 0, 'Temp dir should not be empty');
  });

  await test('ensureDir creates directory', () => {
    const testDir = path.join(utils.getTempDir(), `utils-test-${Date.now()}`);
    try {
      utils.ensureDir(testDir);
      assert.ok(fs.existsSync(testDir), 'Directory should be created');
    } finally {
      fs.rmSync(testDir, { recursive: true, force: true });
    }
  });

  // Date/Time functions tests
  describe('Date/Time Functions');

  await test('getDateString returns YYYY-MM-DD format', () => {
    const date = utils.getDateString();
    assert.ok(/^\d{4}-\d{2}-\d{2}$/.test(date), `Expected YYYY-MM-DD, got ${date}`);
  });

  await test('getTimeString returns HH:MM format', () => {
    const time = utils.getTimeString();
    assert.ok(/^\d{2}:\d{2}$/.test(time), `Expected HH:MM, got ${time}`);
  });

  await test('getDateTimeString returns full datetime format', () => {
    const dt = utils.getDateTimeString();
    assert.ok(/^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$/.test(dt), `Expected YYYY-MM-DD HH:MM:SS, got ${dt}`);
  });

  // Project name tests
  describe('Project Name Functions');

  await test('getGitRepoName returns string or null', () => {
    const repoName = utils.getGitRepoName();
    assert.ok(repoName === null || typeof repoName === 'string');
  });

  await test('getProjectName returns non-empty string', () => {
    const name = utils.getProjectName();
    assert.ok(name && name.length > 0);
  });

  // Session ID tests
  describe('Session ID Functions');

  await test('getSessionIdShort falls back to project name', () => {
    const original = process.env.CLAUDE_SESSION_ID;
    delete process.env.CLAUDE_SESSION_ID;
    try {
      const shortId = utils.getSessionIdShort();
      assert.strictEqual(shortId, utils.getProjectName());
    } finally {
      if (original) process.env.CLAUDE_SESSION_ID = original;
    }
  });

  await test('getSessionIdShort returns last 8 characters', () => {
    const original = process.env.CLAUDE_SESSION_ID;
    process.env.CLAUDE_SESSION_ID = 'test-session-abc12345';
    try {
      assert.strictEqual(utils.getSessionIdShort(), 'abc12345');
    } finally {
      if (original) process.env.CLAUDE_SESSION_ID = original;
      else delete process.env.CLAUDE_SESSION_ID;
    }
  });

  await test('getSessionIdShort handles short session IDs', () => {
    const original = process.env.CLAUDE_SESSION_ID;
    process.env.CLAUDE_SESSION_ID = 'short';
    try {
      assert.strictEqual(utils.getSessionIdShort(), 'short');
    } finally {
      if (original) process.env.CLAUDE_SESSION_ID = original;
      else delete process.env.CLAUDE_SESSION_ID;
    }
  });

  // File operations tests
  describe('File Operations');

  await test('readFile returns null for non-existent file', () => {
    const content = utils.readFile('/non/existent/file/path.txt');
    assert.strictEqual(content, null);
  });

  await test('writeFile and readFile work together', () => {
    const testFile = path.join(utils.getTempDir(), `utils-test-${Date.now()}.txt`);
    const testContent = 'Hello, World!';
    try {
      utils.writeFile(testFile, testContent);
      const read = utils.readFile(testFile);
      assert.strictEqual(read, testContent);
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  await test('appendFile adds content to file', () => {
    const testFile = path.join(utils.getTempDir(), `utils-test-${Date.now()}.txt`);
    try {
      utils.writeFile(testFile, 'Line 1\n');
      utils.appendFile(testFile, 'Line 2\n');
      const content = utils.readFile(testFile);
      assert.strictEqual(content, 'Line 1\nLine 2\n');
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  await test('replaceInFile replaces text', () => {
    const testFile = path.join(utils.getTempDir(), `utils-test-${Date.now()}.txt`);
    try {
      utils.writeFile(testFile, 'Hello, World!');
      utils.replaceInFile(testFile, /World/, 'Universe');
      const content = utils.readFile(testFile);
      assert.strictEqual(content, 'Hello, Universe!');
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  await test('countInFile counts occurrences', () => {
    const testFile = path.join(utils.getTempDir(), `utils-test-${Date.now()}.txt`);
    try {
      utils.writeFile(testFile, 'foo bar foo baz foo');
      const count = utils.countInFile(testFile, /foo/g);
      assert.strictEqual(count, 3);
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  await test('grepFile finds matching lines', () => {
    const testFile = path.join(utils.getTempDir(), `utils-test-${Date.now()}.txt`);
    try {
      utils.writeFile(testFile, 'line 1 foo\nline 2 bar\nline 3 foo');
      const matches = utils.grepFile(testFile, /foo/);
      assert.strictEqual(matches.length, 2);
      assert.strictEqual(matches[0].lineNumber, 1);
      assert.strictEqual(matches[1].lineNumber, 3);
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  // findFiles tests
  describe('findFiles');

  await test('findFiles returns empty for non-existent directory', () => {
    const results = utils.findFiles('/non/existent/dir', '*.txt');
    assert.strictEqual(results.length, 0);
  });

  await test('findFiles finds matching files', () => {
    const testDir = path.join(utils.getTempDir(), `utils-test-${Date.now()}`);
    try {
      fs.mkdirSync(testDir);
      fs.writeFileSync(path.join(testDir, 'test1.txt'), 'content');
      fs.writeFileSync(path.join(testDir, 'test2.txt'), 'content');
      fs.writeFileSync(path.join(testDir, 'test.md'), 'content');

      const txtFiles = utils.findFiles(testDir, '*.txt');
      assert.strictEqual(txtFiles.length, 2);

      const mdFiles = utils.findFiles(testDir, '*.md');
      assert.strictEqual(mdFiles.length, 1);
    } finally {
      fs.rmSync(testDir, { recursive: true });
    }
  });

  // Edge case tests for defensive code
  describe('Edge Cases');

  await test('findFiles returns empty for null/undefined dir', () => {
    assert.deepStrictEqual(utils.findFiles(null, '*.txt'), []);
    assert.deepStrictEqual(utils.findFiles(undefined, '*.txt'), []);
    assert.deepStrictEqual(utils.findFiles('', '*.txt'), []);
  });

  await test('findFiles returns empty for null/undefined pattern', () => {
    assert.deepStrictEqual(utils.findFiles('/tmp', null), []);
    assert.deepStrictEqual(utils.findFiles('/tmp', undefined), []);
    assert.deepStrictEqual(utils.findFiles('/tmp', ''), []);
  });

  await test('findFiles supports maxAge filter', () => {
    const testDir = path.join(utils.getTempDir(), `utils-test-maxage-${Date.now()}`);
    try {
      fs.mkdirSync(testDir);
      fs.writeFileSync(path.join(testDir, 'recent.txt'), 'content');
      const results = utils.findFiles(testDir, '*.txt', { maxAge: 1 });
      assert.strictEqual(results.length, 1);
      assert.ok(results[0].path.endsWith('recent.txt'));
    } finally {
      fs.rmSync(testDir, { recursive: true, force: true });
    }
  });

  await test('findFiles supports recursive option', () => {
    const testDir = path.join(utils.getTempDir(), `utils-test-recursive-${Date.now()}`);
    const subDir = path.join(testDir, 'sub');
    try {
      fs.mkdirSync(subDir, { recursive: true });
      fs.writeFileSync(path.join(testDir, 'top.txt'), 'content');
      fs.writeFileSync(path.join(subDir, 'nested.txt'), 'content');
      // Without recursive: only top level
      const shallow = utils.findFiles(testDir, '*.txt', { recursive: false });
      assert.strictEqual(shallow.length, 1);
      // With recursive: finds nested too
      const deep = utils.findFiles(testDir, '*.txt', { recursive: true });
      assert.strictEqual(deep.length, 2);
    } finally {
      fs.rmSync(testDir, { recursive: true, force: true });
    }
  });

  await test('countInFile handles invalid regex pattern', () => {
    const testFile = path.join(utils.getTempDir(), `utils-test-${Date.now()}.txt`);
    try {
      utils.writeFile(testFile, 'test content');
      const count = utils.countInFile(testFile, '(unclosed');
      assert.strictEqual(count, 0);
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  await test('countInFile handles non-string non-regex pattern', () => {
    const testFile = path.join(utils.getTempDir(), `utils-test-${Date.now()}.txt`);
    try {
      utils.writeFile(testFile, 'test content');
      const count = utils.countInFile(testFile, 42);
      assert.strictEqual(count, 0);
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  await test('countInFile enforces global flag on RegExp', () => {
    const testFile = path.join(utils.getTempDir(), `utils-test-${Date.now()}.txt`);
    try {
      utils.writeFile(testFile, 'foo bar foo baz foo');
      // RegExp without global flag — countInFile should still count all
      const count = utils.countInFile(testFile, /foo/);
      assert.strictEqual(count, 3);
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  await test('grepFile handles invalid regex pattern', () => {
    const testFile = path.join(utils.getTempDir(), `utils-test-${Date.now()}.txt`);
    try {
      utils.writeFile(testFile, 'test content');
      const matches = utils.grepFile(testFile, '[invalid');
      assert.deepStrictEqual(matches, []);
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  await test('replaceInFile returns false for non-existent file', () => {
    const result = utils.replaceInFile('/non/existent/file.txt', 'foo', 'bar');
    assert.strictEqual(result, false);
  });

  await test('countInFile returns 0 for non-existent file', () => {
    const count = utils.countInFile('/non/existent/file.txt', /foo/g);
    assert.strictEqual(count, 0);
  });

  await test('grepFile returns empty for non-existent file', () => {
    const matches = utils.grepFile('/non/existent/file.txt', /foo/);
    assert.deepStrictEqual(matches, []);
  });

  await test('commandExists rejects unsafe command names', () => {
    assert.strictEqual(utils.commandExists('cmd; rm -rf'), false);
    assert.strictEqual(utils.commandExists('$(whoami)'), false);
    assert.strictEqual(utils.commandExists('cmd && echo hi'), false);
  });

  await test('ensureDir is idempotent', () => {
    const testDir = path.join(utils.getTempDir(), `utils-test-idem-${Date.now()}`);
    try {
      const result1 = utils.ensureDir(testDir);
      const result2 = utils.ensureDir(testDir);
      assert.strictEqual(result1, testDir);
      assert.strictEqual(result2, testDir);
      assert.ok(fs.existsSync(testDir));
    } finally {
      fs.rmSync(testDir, { recursive: true, force: true });
    }
  });

  // System functions tests
  describe('System Functions');

  await test('commandExists finds node', () => {
    const exists = utils.commandExists('node');
    assert.strictEqual(exists, true);
  });

  await test('commandExists returns false for fake command', () => {
    const exists = utils.commandExists('nonexistent_command_12345');
    assert.strictEqual(exists, false);
  });

  await test('runCommand executes simple command', () => {
    const result = utils.runCommand('node --version');
    assert.strictEqual(result.success, true);
    assert.ok(result.output.startsWith('v'), 'Should start with v');
  });

  await test('runCommand handles failed command', () => {
    const result = utils.runCommand('node --invalid-flag-12345');
    assert.strictEqual(result.success, false);
  });

  // output() and log() tests
  describe('output() and log()');

  await test('output() writes string to stdout', () => {
    // Capture stdout by temporarily replacing console.log
    let captured = null;
    const origLog = console.log;
    console.log = v => {
      captured = v;
    };
    try {
      utils.output('hello');
      assert.strictEqual(captured, 'hello');
    } finally {
      console.log = origLog;
    }
  });

  await test('output() JSON-stringifies objects', () => {
    let captured = null;
    const origLog = console.log;
    console.log = v => {
      captured = v;
    };
    try {
      utils.output({ key: 'value', num: 42 });
      assert.strictEqual(captured, '{"key":"value","num":42}');
    } finally {
      console.log = origLog;
    }
  });

  await test('output() JSON-stringifies null (typeof null === "object")', () => {
    let captured = null;
    const origLog = console.log;
    console.log = v => {
      captured = v;
    };
    try {
      utils.output(null);
      // typeof null === 'object' in JS, so it goes through JSON.stringify
      assert.strictEqual(captured, 'null');
    } finally {
      console.log = origLog;
    }
  });

  await test('output() handles arrays as objects', () => {
    let captured = null;
    const origLog = console.log;
    console.log = v => {
      captured = v;
    };
    try {
      utils.output([1, 2, 3]);
      assert.strictEqual(captured, '[1,2,3]');
    } finally {
      console.log = origLog;
    }
  });

  await test('log() writes to stderr', () => {
    let captured = null;
    const origError = console.error;
    console.error = v => {
      captured = v;
    };
    try {
      utils.log('test message');
      assert.strictEqual(captured, 'test message');
    } finally {
      console.error = origError;
    }
  });

  // isGitRepo() tests
  describe('isGitRepo()');

  await test('isGitRepo returns true in a git repo', () => {
    // We're running from within the ECC repo, so this should be true
    assert.strictEqual(utils.isGitRepo(), true);
  });

  // getGitModifiedFiles() tests
  describe('getGitModifiedFiles()');

  await test('getGitModifiedFiles returns an array', () => {
    const files = utils.getGitModifiedFiles();
    assert.ok(Array.isArray(files));
  });

  await test('getGitModifiedFiles filters by regex patterns', () => {
    const files = utils.getGitModifiedFiles(['\\.NONEXISTENT_EXTENSION$']);
    assert.ok(Array.isArray(files));
    assert.strictEqual(files.length, 0);
  });

  await test('getGitModifiedFiles skips invalid patterns', () => {
    // Mix of valid and invalid patterns — should not throw
    const files = utils.getGitModifiedFiles(['(unclosed', '\\.js$', '[invalid']);
    assert.ok(Array.isArray(files));
  });

  await test('getGitModifiedFiles skips non-string patterns', () => {
    const files = utils.getGitModifiedFiles([null, undefined, 42, '', '\\.js$']);
    assert.ok(Array.isArray(files));
  });

  // getLearnedSkillsDir() test
  describe('getLearnedSkillsDir()');

  await test('getLearnedSkillsDir returns path under Claude dir', () => {
    const dir = utils.getLearnedSkillsDir();
    assert.ok(dir.includes('.claude'));
    assert.ok(dir.includes('skills'));
    assert.ok(dir.includes('learned'));
  });

  // replaceInFile behavior tests
  describe('replaceInFile (behavior)');

  await test('replaces first match when regex has no g flag', () => {
    const testFile = path.join(utils.getTempDir(), `utils-test-${Date.now()}.txt`);
    try {
      utils.writeFile(testFile, 'foo bar foo baz foo');
      utils.replaceInFile(testFile, /foo/, 'qux');
      const content = utils.readFile(testFile);
      // Without g flag, only first 'foo' should be replaced
      assert.strictEqual(content, 'qux bar foo baz foo');
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  await test('replaces all matches when regex has g flag', () => {
    const testFile = path.join(utils.getTempDir(), `utils-test-${Date.now()}.txt`);
    try {
      utils.writeFile(testFile, 'foo bar foo baz foo');
      utils.replaceInFile(testFile, /foo/g, 'qux');
      const content = utils.readFile(testFile);
      assert.strictEqual(content, 'qux bar qux baz qux');
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  await test('replaces with string search (first occurrence)', () => {
    const testFile = path.join(utils.getTempDir(), `utils-test-${Date.now()}.txt`);
    try {
      utils.writeFile(testFile, 'hello world hello');
      utils.replaceInFile(testFile, 'hello', 'goodbye');
      const content = utils.readFile(testFile);
      // String.replace with string search only replaces first
      assert.strictEqual(content, 'goodbye world hello');
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  await test('replaces all occurrences with string when options.all is true', () => {
    const testFile = path.join(utils.getTempDir(), `utils-test-${Date.now()}.txt`);
    try {
      utils.writeFile(testFile, 'hello world hello again hello');
      utils.replaceInFile(testFile, 'hello', 'goodbye', { all: true });
      const content = utils.readFile(testFile);
      assert.strictEqual(content, 'goodbye world goodbye again goodbye');
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  await test('options.all is ignored for regex patterns', () => {
    const testFile = path.join(utils.getTempDir(), `utils-test-${Date.now()}.txt`);
    try {
      utils.writeFile(testFile, 'foo bar foo');
      // all option should be ignored for regex; only g flag matters
      utils.replaceInFile(testFile, /foo/, 'qux', { all: true });
      const content = utils.readFile(testFile);
      assert.strictEqual(content, 'qux bar foo', 'Regex without g should still replace first only');
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  await test('replaces with capture groups', () => {
    const testFile = path.join(utils.getTempDir(), `utils-test-${Date.now()}.txt`);
    try {
      utils.writeFile(testFile, '**Last Updated:** 10:30');
      utils.replaceInFile(testFile, /\*\*Last Updated:\*\*.*/, '**Last Updated:** 14:45');
      const content = utils.readFile(testFile);
      assert.strictEqual(content, '**Last Updated:** 14:45');
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  // writeFile edge cases
  describe('writeFile (edge cases)');

  await test('writeFile overwrites existing content', () => {
    const testFile = path.join(utils.getTempDir(), `utils-test-${Date.now()}.txt`);
    try {
      utils.writeFile(testFile, 'original');
      utils.writeFile(testFile, 'replaced');
      const content = utils.readFile(testFile);
      assert.strictEqual(content, 'replaced');
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  await test('writeFile handles unicode content', () => {
    const testFile = path.join(utils.getTempDir(), `utils-test-${Date.now()}.txt`);
    try {
      const unicode = '日本語テスト 🚀 émojis';
      utils.writeFile(testFile, unicode);
      const content = utils.readFile(testFile);
      assert.strictEqual(content, unicode);
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  // findFiles with regex special characters in pattern
  describe('findFiles (regex chars)');

  await test('findFiles handles regex special chars in pattern', () => {
    const testDir = path.join(utils.getTempDir(), `utils-test-regex-${Date.now()}`);
    try {
      fs.mkdirSync(testDir);
      // Create files with regex-special characters in names
      fs.writeFileSync(path.join(testDir, 'file(1).txt'), 'content');
      fs.writeFileSync(path.join(testDir, 'file+2.txt'), 'content');
      fs.writeFileSync(path.join(testDir, 'file[3].txt'), 'content');

      // These patterns should match literally, not as regex metacharacters
      const parens = utils.findFiles(testDir, 'file(1).txt');
      assert.strictEqual(parens.length, 1, 'Should match file(1).txt literally');

      const plus = utils.findFiles(testDir, 'file+2.txt');
      assert.strictEqual(plus.length, 1, 'Should match file+2.txt literally');

      const brackets = utils.findFiles(testDir, 'file[3].txt');
      assert.strictEqual(brackets.length, 1, 'Should match file[3].txt literally');
    } finally {
      fs.rmSync(testDir, { recursive: true, force: true });
    }
  });

  await test('findFiles wildcard still works with special chars', () => {
    const testDir = path.join(utils.getTempDir(), `utils-test-glob-${Date.now()}`);
    try {
      fs.mkdirSync(testDir);
      fs.writeFileSync(path.join(testDir, 'app(v2).js'), 'content');
      fs.writeFileSync(path.join(testDir, 'app(v3).ts'), 'content');

      const jsFiles = utils.findFiles(testDir, '*.js');
      assert.strictEqual(jsFiles.length, 1);
      assert.ok(jsFiles[0].path.endsWith('app(v2).js'));
    } finally {
      fs.rmSync(testDir, { recursive: true, force: true });
    }
  });

  // readStdinJson tests (via subprocess — safe hardcoded inputs)
  // Use execFileSync with input option instead of shell echo|pipe for Windows compat
  describe('readStdinJson()');

  const stdinScript = 'const u=require("./dist/lib/utils");u.readStdinJson({timeoutMs:2000}).then(d=>{process.stdout.write(JSON.stringify(d))})';
  const stdinOpts = { encoding: 'utf8', cwd: path.join(__dirname, '..', '..'), timeout: 5000 };

  await test('readStdinJson parses valid JSON from stdin', () => {
    const { execFileSync } = require('child_process');
    const result = execFileSync('node', ['-e', stdinScript], { ...stdinOpts, input: '{"tool_input":{"command":"ls"}}' });
    const parsed = JSON.parse(result);
    assert.deepStrictEqual(parsed, { tool_input: { command: 'ls' } });
  });

  await test('readStdinJson returns {} for invalid JSON', () => {
    const { execFileSync } = require('child_process');
    const result = execFileSync('node', ['-e', stdinScript], { ...stdinOpts, input: 'not json' });
    assert.deepStrictEqual(JSON.parse(result), {});
  });

  await test('readStdinJson returns {} for empty stdin', () => {
    const { execFileSync } = require('child_process');
    const result = execFileSync('node', ['-e', stdinScript], { ...stdinOpts, input: '' });
    assert.deepStrictEqual(JSON.parse(result), {});
  });

  await test('readStdinJson handles nested objects', () => {
    const { execFileSync } = require('child_process');
    const result = execFileSync('node', ['-e', stdinScript], { ...stdinOpts, input: '{"a":{"b":1},"c":[1,2]}' });
    const parsed = JSON.parse(result);
    assert.deepStrictEqual(parsed, { a: { b: 1 }, c: [1, 2] });
  });

  // grepFile with global regex (regression: g flag causes alternating matches)
  describe('grepFile (global regex fix)');

  await test('grepFile with /g flag finds ALL matching lines (not alternating)', () => {
    const testFile = path.join(utils.getTempDir(), `utils-test-grep-g-${Date.now()}.txt`);
    try {
      // 4 consecutive lines matching the same pattern
      utils.writeFile(testFile, 'match-line\nmatch-line\nmatch-line\nmatch-line');
      // Bug: without fix, /match/g would only find lines 1 and 3 (alternating)
      const matches = utils.grepFile(testFile, /match/g);
      assert.strictEqual(matches.length, 4, `Should find all 4 lines, found ${matches.length}`);
      assert.strictEqual(matches[0].lineNumber, 1);
      assert.strictEqual(matches[1].lineNumber, 2);
      assert.strictEqual(matches[2].lineNumber, 3);
      assert.strictEqual(matches[3].lineNumber, 4);
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  await test('grepFile preserves regex flags other than g (e.g. case-insensitive)', () => {
    const testFile = path.join(utils.getTempDir(), `utils-test-grep-flags-${Date.now()}.txt`);
    try {
      utils.writeFile(testFile, 'FOO\nfoo\nFoO\nbar');
      const matches = utils.grepFile(testFile, /foo/gi);
      assert.strictEqual(matches.length, 3, `Should find 3 case-insensitive matches, found ${matches.length}`);
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  // commandExists edge cases
  describe('commandExists Edge Cases');

  await test('commandExists rejects empty string', () => {
    assert.strictEqual(utils.commandExists(''), false, 'Empty string should not be a valid command');
  });

  await test('commandExists rejects command with spaces', () => {
    assert.strictEqual(utils.commandExists('my command'), false, 'Commands with spaces should be rejected');
  });

  await test('commandExists rejects command with path separators', () => {
    assert.strictEqual(utils.commandExists('/usr/bin/node'), false, 'Commands with / should be rejected');
    assert.strictEqual(utils.commandExists('..\\cmd'), false, 'Commands with \\ should be rejected');
  });

  await test('commandExists rejects shell metacharacters', () => {
    assert.strictEqual(utils.commandExists('cmd;ls'), false, 'Semicolons should be rejected');
    assert.strictEqual(utils.commandExists('$(whoami)'), false, 'Subshell syntax should be rejected');
    assert.strictEqual(utils.commandExists('cmd|cat'), false, 'Pipes should be rejected');
  });

  await test('commandExists allows dots and underscores', () => {
    // These are valid chars per the regex check — the command might not exist
    // but it shouldn't be rejected by the validator
    const dotResult = utils.commandExists('definitely.not.a.real.tool.12345');
    assert.strictEqual(typeof dotResult, 'boolean', 'Should return boolean, not throw');
  });

  // findFiles edge cases
  describe('findFiles Edge Cases');

  await test('findFiles with ? wildcard matches single character', () => {
    const testDir = path.join(utils.getTempDir(), `ff-qmark-${Date.now()}`);
    utils.ensureDir(testDir);
    try {
      fs.writeFileSync(path.join(testDir, 'a1.txt'), '');
      fs.writeFileSync(path.join(testDir, 'b2.txt'), '');
      fs.writeFileSync(path.join(testDir, 'abc.txt'), '');

      const results = utils.findFiles(testDir, '??.txt');
      const names = results.map(r => path.basename(r.path)).sort();
      assert.deepStrictEqual(names, ['a1.txt', 'b2.txt'], 'Should match exactly 2-char basenames');
    } finally {
      fs.rmSync(testDir, { recursive: true, force: true });
    }
  });

  await test('findFiles sorts by mtime (newest first)', () => {
    const testDir = path.join(utils.getTempDir(), `ff-sort-${Date.now()}`);
    utils.ensureDir(testDir);
    try {
      const f1 = path.join(testDir, 'old.txt');
      const f2 = path.join(testDir, 'new.txt');
      fs.writeFileSync(f1, 'old');
      // Set older mtime on first file
      const past = new Date(Date.now() - 60000);
      fs.utimesSync(f1, past, past);
      fs.writeFileSync(f2, 'new');

      const results = utils.findFiles(testDir, '*.txt');
      assert.strictEqual(results.length, 2);
      assert.ok(path.basename(results[0].path) === 'new.txt', `Newest file should be first, got ${path.basename(results[0].path)}`);
    } finally {
      fs.rmSync(testDir, { recursive: true, force: true });
    }
  });

  await test('findFiles with maxAge filters old files', () => {
    const testDir = path.join(utils.getTempDir(), `ff-age-${Date.now()}`);
    utils.ensureDir(testDir);
    try {
      const recent = path.join(testDir, 'recent.txt');
      const old = path.join(testDir, 'old.txt');
      fs.writeFileSync(recent, 'new');
      fs.writeFileSync(old, 'old');
      // Set mtime to 30 days ago
      const past = new Date(Date.now() - 30 * 24 * 60 * 60 * 1000);
      fs.utimesSync(old, past, past);

      const results = utils.findFiles(testDir, '*.txt', { maxAge: 7 });
      assert.strictEqual(results.length, 1, 'Should only return recent file');
      assert.ok(results[0].path.includes('recent.txt'), 'Should return the recent file');
    } finally {
      fs.rmSync(testDir, { recursive: true, force: true });
    }
  });

  // ensureDir edge cases
  describe('ensureDir Edge Cases');

  await test('ensureDir is safe for concurrent calls (EEXIST race)', () => {
    const testDir = path.join(utils.getTempDir(), `ensure-race-${Date.now()}`, 'nested');
    try {
      // Call concurrently — both should succeed without throwing
      const results = [utils.ensureDir(testDir), utils.ensureDir(testDir)];
      assert.strictEqual(results[0], testDir);
      assert.strictEqual(results[1], testDir);
      assert.ok(fs.existsSync(testDir));
    } finally {
      fs.rmSync(path.dirname(testDir), { recursive: true, force: true });
    }
  });

  await test('ensureDir returns the directory path', () => {
    const testDir = path.join(utils.getTempDir(), `ensure-ret-${Date.now()}`);
    try {
      const result = utils.ensureDir(testDir);
      assert.strictEqual(result, testDir, 'Should return the directory path');
    } finally {
      fs.rmSync(testDir, { recursive: true, force: true });
    }
  });

  // runCommand edge cases
  describe('runCommand Edge Cases');

  await test('runCommand returns trimmed output', () => {
    // Windows echo includes quotes in output, use node to ensure consistent behavior
    const result = utils.runCommand('node -e "process.stdout.write(\'  hello  \')"');
    assert.strictEqual(result.success, true);
    assert.strictEqual(result.output, 'hello', 'Should trim leading/trailing whitespace');
  });

  await test('runCommand captures stderr on failure', () => {
    const result = utils.runCommand('node -e "process.exit(1)"');
    assert.strictEqual(result.success, false);
    assert.ok(typeof result.output === 'string', 'Output should be a string on failure');
  });

  // getGitModifiedFiles edge cases
  describe('getGitModifiedFiles Edge Cases');

  await test('getGitModifiedFiles returns array with empty patterns', () => {
    const files = utils.getGitModifiedFiles([]);
    assert.ok(Array.isArray(files), 'Should return array');
  });

  // replaceInFile edge cases
  describe('replaceInFile Edge Cases');

  await test('replaceInFile with regex capture groups works correctly', () => {
    const testFile = path.join(utils.getTempDir(), `replace-capture-${Date.now()}.txt`);
    try {
      utils.writeFile(testFile, 'version: 1.0.0');
      const result = utils.replaceInFile(testFile, /version: (\d+)\.(\d+)\.(\d+)/, 'version: $1.$2.99');
      assert.strictEqual(result, true);
      assert.strictEqual(utils.readFile(testFile), 'version: 1.0.99');
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  // readStdinJson (function API, not actual stdin — more thorough edge cases)
  describe('readStdinJson Edge Cases');

  await test('readStdinJson type check: returns a Promise', () => {
    // readStdinJson returns a Promise regardless of stdin state
    const result = utils.readStdinJson({ timeoutMs: 100 });
    assert.ok(result instanceof Promise, 'Should return a Promise');
    // Don't await — just verify it's a Promise type
  });

  // ── Round 28: readStdinJson maxSize truncation and edge cases ──
  describe('readStdinJson maxSize truncation');

  await test('readStdinJson maxSize stops accumulating after threshold (chunk-level guard)', () => {
    if (process.platform === 'win32') {
      console.log('    (skipped — stdin chunking behavior differs on Windows)');
      return true;
    }
    const { execFileSync } = require('child_process');
    // maxSize is a chunk-level guard: once data.length >= maxSize, no MORE chunks are added.
    // A single small chunk that arrives when data.length < maxSize is added in full.
    // To test multi-chunk behavior, we send >64KB (Node default highWaterMark=16KB)
    // which should arrive in multiple chunks. With maxSize=100, only the first chunk(s)
    // totaling under 100 bytes should be captured; subsequent chunks are dropped.
    const script = 'const u=require("./dist/lib/utils");u.readStdinJson({timeoutMs:2000,maxSize:100}).then(d=>{process.stdout.write(JSON.stringify(d))})';
    // Generate 100KB of data (arrives in multiple chunks)
    const bigInput = '{"k":"' + 'X'.repeat(100000) + '"}';
    const result = execFileSync('node', ['-e', script], { ...stdinOpts, input: bigInput });
    // Truncated mid-string → invalid JSON → resolves to {}
    assert.deepStrictEqual(JSON.parse(result), {});
  });

  await test('readStdinJson with maxSize large enough preserves valid JSON', () => {
    const { execFileSync } = require('child_process');
    const script = 'const u=require("./dist/lib/utils");u.readStdinJson({timeoutMs:2000,maxSize:1024}).then(d=>{process.stdout.write(JSON.stringify(d))})';
    const input = JSON.stringify({ key: 'value' });
    const result = execFileSync('node', ['-e', script], { ...stdinOpts, input });
    assert.deepStrictEqual(JSON.parse(result), { key: 'value' });
  });

  await test('readStdinJson resolves {} for whitespace-only stdin', () => {
    const { execFileSync } = require('child_process');
    const result = execFileSync('node', ['-e', stdinScript], { ...stdinOpts, input: '   \n  \t  ' });
    // data.trim() is empty → resolves {}
    assert.deepStrictEqual(JSON.parse(result), {});
  });

  await test('readStdinJson handles JSON with trailing whitespace/newlines', () => {
    const { execFileSync } = require('child_process');
    const result = execFileSync('node', ['-e', stdinScript], { ...stdinOpts, input: '{"a":1}  \n\n' });
    assert.deepStrictEqual(JSON.parse(result), { a: 1 });
  });

  await test('readStdinJson handles JSON with BOM prefix (returns {})', () => {
    const { execFileSync } = require('child_process');
    // BOM (\uFEFF) before JSON makes it invalid for JSON.parse
    const result = execFileSync('node', ['-e', stdinScript], { ...stdinOpts, input: '\uFEFF{"a":1}' });
    // BOM prefix makes JSON.parse fail → resolve {}
    assert.deepStrictEqual(JSON.parse(result), {});
  });

  // ── Round 31: ensureDir error propagation ──
  describe('ensureDir Error Propagation (Round 31)');

  await test('ensureDir wraps non-EEXIST errors with descriptive message', () => {
    // Attempting to create a dir under a file should fail with ENOTDIR, not EEXIST
    const testFile = path.join(utils.getTempDir(), `ensure-err-${Date.now()}.txt`);
    try {
      fs.writeFileSync(testFile, 'blocking file');
      const badPath = path.join(testFile, 'subdir');
      assert.throws(
        () => utils.ensureDir(badPath),
        err => err.message.includes('Failed to create directory'),
        'Should throw with descriptive "Failed to create directory" message'
      );
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  await test('ensureDir error includes the directory path', () => {
    const testFile = path.join(utils.getTempDir(), `ensure-err2-${Date.now()}.txt`);
    try {
      fs.writeFileSync(testFile, 'blocker');
      const badPath = path.join(testFile, 'nested', 'dir');
      try {
        utils.ensureDir(badPath);
        assert.fail('Should have thrown');
      } catch (err) {
        assert.ok(err.message.includes(badPath), 'Error should include the target path');
      }
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  // ── Round 31: runCommand stderr preference on failure ──
  describe('runCommand failure output (Round 31)');

  await test('runCommand returns stderr content on failure when stderr exists', () => {
    const result = utils.runCommand('node -e "process.stderr.write(\'custom error\'); process.exit(1)"');
    assert.strictEqual(result.success, false);
    assert.ok(result.output.includes('custom error'), 'Should include stderr output');
  });

  await test('runCommand falls back to err.message when no stderr', () => {
    // An invalid command that won't produce stderr through child process
    const result = utils.runCommand('nonexistent_cmd_xyz_12345');
    assert.strictEqual(result.success, false);
    assert.ok(result.output.length > 0, 'Should have some error output');
  });

  // ── Round 31: getGitModifiedFiles with empty patterns ──
  describe('getGitModifiedFiles empty patterns (Round 31)');

  await test('getGitModifiedFiles with empty array returns all modified files', () => {
    // With an empty patterns array, every file should match (no filter applied)
    const withEmpty = utils.getGitModifiedFiles([]);
    const withNone = utils.getGitModifiedFiles();
    // Both should return the same list (no filtering)
    assert.deepStrictEqual(withEmpty, withNone, 'Empty patterns array should behave same as no patterns');
  });

  // ── Round 33: readStdinJson error event handling ──
  describe('readStdinJson error event (Round 33)');

  await test('readStdinJson resolves {} when stdin emits error (via broken pipe)', () => {
    // Spawn a subprocess that reads from stdin, but close the pipe immediately
    // to trigger an error or early-end condition
    const { execFileSync } = require('child_process');
    const script = 'const u=require("./dist/lib/utils");u.readStdinJson({timeoutMs:2000}).then(d=>{process.stdout.write(JSON.stringify(d))})';
    // Pipe stdin from /dev/null — this sends EOF immediately (no data)
    const result = execFileSync('node', ['-e', script], {
      encoding: 'utf8',
      input: '', // empty stdin triggers 'end' with empty data
      timeout: 5000,
      cwd: path.join(__dirname, '..', '..')
    });
    const parsed = JSON.parse(result);
    assert.deepStrictEqual(parsed, {}, 'Should resolve to {} for empty stdin (end event path)');
  });

  await test('readStdinJson error handler is guarded by settled flag', () => {
    // If 'end' fires first setting settled=true, then a late 'error' should be ignored
    // We test this by verifying the code structure works: send valid JSON, the end event
    // fires, settled=true, any late error is safely ignored
    const { execFileSync } = require('child_process');
    const script = 'const u=require("./dist/lib/utils");u.readStdinJson({timeoutMs:2000}).then(d=>{process.stdout.write(JSON.stringify(d))})';
    const result = execFileSync('node', ['-e', script], {
      encoding: 'utf8',
      input: '{"test":"settled-guard"}',
      timeout: 5000,
      cwd: path.join(__dirname, '..', '..')
    });
    const parsed = JSON.parse(result);
    assert.strictEqual(parsed.test, 'settled-guard', 'Should parse normally when end fires first');
  });

  // replaceInFile returns false when write fails (e.g., read-only file)
  await test('replaceInFile returns false on write failure (read-only file)', () => {
    if (process.platform === 'win32' || process.getuid?.() === 0) {
      console.log('    (skipped — chmod ineffective on Windows/root)');
      return;
    }
    const testDir = path.join(utils.getTempDir(), `utils-test-readonly-${Date.now()}`);
    fs.mkdirSync(testDir, { recursive: true });
    const filePath = path.join(testDir, 'readonly.txt');
    try {
      fs.writeFileSync(filePath, 'hello world', 'utf8');
      fs.chmodSync(filePath, 0o444);
      const result = utils.replaceInFile(filePath, 'hello', 'goodbye');
      assert.strictEqual(result, false, 'Should return false when file is read-only');
      // Verify content unchanged
      const content = fs.readFileSync(filePath, 'utf8');
      assert.strictEqual(content, 'hello world', 'Original content should be preserved');
    } finally {
      fs.chmodSync(filePath, 0o644);
      fs.rmSync(testDir, { recursive: true, force: true });
    }
  });

  // ── Round 69: getGitModifiedFiles with ALL invalid patterns ──
  describe('getGitModifiedFiles all-invalid patterns (Round 69)');

  await test('getGitModifiedFiles with all-invalid patterns skips filtering (returns all files)', () => {
    // When every pattern is invalid regex, compiled.length === 0 at line 386,
    // so the filtering is skipped entirely and all modified files are returned.
    // This differs from the mixed-valid test where at least one pattern compiles.
    const allInvalid = utils.getGitModifiedFiles(['(unclosed', '[bad', '**invalid']);
    const unfiltered = utils.getGitModifiedFiles();
    // Both should return the same list — all-invalid patterns = no filtering
    assert.deepStrictEqual(allInvalid, unfiltered, 'All-invalid patterns should return same result as no patterns (no filtering)');
  });

  // ── Round 71: findFiles recursive scan skips unreadable subdirectory ──
  describe('Round 71: findFiles (unreadable subdirectory in recursive scan)');

  await test('findFiles recursive scan skips unreadable subdirectory silently', () => {
    if (process.platform === 'win32' || process.getuid?.() === 0) {
      console.log('    (skipped — chmod ineffective on Windows/root)');
      return;
    }
    const tmpDir = path.join(utils.getTempDir(), `ecc-findfiles-r71-${Date.now()}`);
    const readableSubdir = path.join(tmpDir, 'readable');
    const unreadableSubdir = path.join(tmpDir, 'unreadable');
    fs.mkdirSync(readableSubdir, { recursive: true });
    fs.mkdirSync(unreadableSubdir, { recursive: true });

    // Create files in both subdirectories
    fs.writeFileSync(path.join(readableSubdir, 'found.txt'), 'data');
    fs.writeFileSync(path.join(unreadableSubdir, 'hidden.txt'), 'data');

    // Make the subdirectory unreadable — readdirSync will throw EACCES
    fs.chmodSync(unreadableSubdir, 0o000);

    try {
      const results = utils.findFiles(tmpDir, '*.txt', { recursive: true });
      // Should find the readable file but silently skip the unreadable dir
      assert.ok(results.length >= 1, 'Should find at least the readable file');
      const paths = results.map(r => r.path);
      assert.ok(
        paths.some(p => p.includes('found.txt')),
        'Should find readable/found.txt'
      );
      assert.ok(!paths.some(p => p.includes('hidden.txt')), 'Should not find unreadable/hidden.txt');
    } finally {
      fs.chmodSync(unreadableSubdir, 0o755);
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 79: countInFile with valid string pattern ──
  describe('Round 79: countInFile (valid string pattern)');

  await test('countInFile counts occurrences using a plain string pattern', () => {
    const testFile = path.join(utils.getTempDir(), `utils-test-count-str-${Date.now()}.txt`);
    try {
      utils.writeFile(testFile, 'apple banana apple cherry apple');
      // Pass a plain string (not RegExp) — exercises typeof pattern === 'string'
      // branch at utils.js:441-442 which creates new RegExp(pattern, 'g')
      const count = utils.countInFile(testFile, 'apple');
      assert.strictEqual(count, 3, 'String pattern should count all occurrences');
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  // ── Round 79: grepFile with valid string pattern ──
  describe('Round 79: grepFile (valid string pattern)');

  await test('grepFile finds matching lines using a plain string pattern', () => {
    const testFile = path.join(utils.getTempDir(), `utils-test-grep-str-${Date.now()}.txt`);
    try {
      utils.writeFile(testFile, 'line1 alpha\nline2 beta\nline3 alpha\nline4 gamma');
      // Pass a plain string (not RegExp) — exercises the else branch
      // at utils.js:468-469 which creates new RegExp(pattern)
      const matches = utils.grepFile(testFile, 'alpha');
      assert.strictEqual(matches.length, 2, 'String pattern should find 2 matching lines');
      assert.strictEqual(matches[0].lineNumber, 1, 'First match at line 1');
      assert.strictEqual(matches[1].lineNumber, 3, 'Second match at line 3');
      assert.ok(matches[0].content.includes('alpha'), 'Content should include pattern');
    } finally {
      fs.unlinkSync(testFile);
    }
  });

  // ── Round 84: findFiles inner statSync catch (TOCTOU — broken symlink) ──
  describe('Round 84: findFiles (inner statSync catch — broken symlink)');

  await test('findFiles skips broken symlinks that match the pattern', () => {
    // findFiles at utils.js:170-173: readdirSync returns entries including broken
    // symlinks (entry.isFile() returns false for broken symlinks, but the test also
    // verifies the overall robustness). On some systems, broken symlinks can be
    // returned by readdirSync and pass through isFile() depending on the driver.
    // More importantly: if statSync throws inside the inner loop, catch continues.
    //
    // To reliably trigger the statSync catch: create a real file, list it, then
    // simulate the race. Since we can't truly race, we use a broken symlink which
    // will at minimum verify the function doesn't crash on unusual dir entries.
    const tmpDir = path.join(utils.getTempDir(), `ecc-r84-findfiles-toctou-${Date.now()}`);
    fs.mkdirSync(tmpDir, { recursive: true });

    // Create a real file and a broken symlink, both matching *.txt
    const realFile = path.join(tmpDir, 'real.txt');
    fs.writeFileSync(realFile, 'content');
    const brokenLink = path.join(tmpDir, 'broken.txt');
    fs.symlinkSync('/nonexistent/path/does/not/exist', brokenLink);

    try {
      const results = utils.findFiles(tmpDir, '*.txt');
      // The real file should be found; the broken symlink should be skipped
      const paths = results.map(r => r.path);
      assert.ok(
        paths.some(p => p.includes('real.txt')),
        'Should find the real file'
      );
      assert.ok(!paths.some(p => p.includes('broken.txt')), 'Should not include broken symlink in results');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 85: getSessionIdShort fallback parameter ──
  describe('getSessionIdShort fallback (Round 85)');

  await test('getSessionIdShort uses fallback when getProjectName returns null (CWD at root)', () => {
    if (process.platform === 'win32') {
      console.log('    (skipped — root CWD differs on Windows)');
      return;
    }
    // Spawn a subprocess at CWD=/ with CLAUDE_SESSION_ID empty.
    // At /, git rev-parse --show-toplevel fails → getGitRepoName() = null.
    // path.basename('/') = '' → '' || null = null → getProjectName() = null.
    // So getSessionIdShort('my-custom-fallback') = null || 'my-custom-fallback'.
    const utilsPath = path.join(__dirname, '..', '..', 'dist', 'lib', 'utils.js');
    const script = `
      const utils = require('${utilsPath.replace(/'/g, "\\'")}');
      process.stdout.write(utils.getSessionIdShort('my-custom-fallback'));
    `;
    const { spawnSync } = require('child_process');
    const result = spawnSync('node', ['-e', script], {
      encoding: 'utf8',
      cwd: '/',
      env: { ...process.env, CLAUDE_SESSION_ID: '' },
      timeout: 10000
    });
    assert.strictEqual(result.status, 0, `Should exit 0, got status ${result.status}. stderr: ${result.stderr}`);
    assert.strictEqual(result.stdout, 'my-custom-fallback', `At CWD=/ with no session ID, should use the fallback parameter. Got: "${result.stdout}"`);
  });

  // ── Round 88: replaceInFile with empty replacement (deletion) ──
  describe('Round 88: replaceInFile with empty replacement string (deletion)');

  await test('replaceInFile with empty string replacement deletes matched text', () => {
    const tmpDir = path.join(utils.getTempDir(), `ecc-r88-replace-empty-${Date.now()}`);
    fs.mkdirSync(tmpDir, { recursive: true });
    const tmpFile = path.join(tmpDir, 'delete-test.txt');
    try {
      fs.writeFileSync(tmpFile, 'hello REMOVE_ME world');
      const result = utils.replaceInFile(tmpFile, 'REMOVE_ME ', '');
      assert.strictEqual(result, true, 'Should return true on successful replacement');
      const content = fs.readFileSync(tmpFile, 'utf8');
      assert.strictEqual(content, 'hello world', 'Empty replacement should delete the matched text');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 88: countInFile with valid file but zero matches ──
  describe('Round 88: countInFile with existing file but non-matching pattern');

  await test('countInFile returns 0 for valid file with no pattern matches', () => {
    const tmpDir = path.join(utils.getTempDir(), `ecc-r88-count-zero-${Date.now()}`);
    fs.mkdirSync(tmpDir, { recursive: true });
    const tmpFile = path.join(tmpDir, 'no-match.txt');
    try {
      fs.writeFileSync(tmpFile, 'apple banana cherry');
      const count = utils.countInFile(tmpFile, 'ZZZZNOTHERE');
      assert.strictEqual(count, 0, 'Should return 0 when regex matches nothing in existing file');
      const countRegex = utils.countInFile(tmpFile, /ZZZZNOTHERE/g);
      assert.strictEqual(countRegex, 0, 'Should return 0 for RegExp with no matches in existing file');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 92: countInFile with object pattern type ──
  describe('Round 92: countInFile (non-string non-RegExp pattern)');

  await test('countInFile returns 0 for object pattern (neither string nor RegExp)', () => {
    // utils.js line 443-444: The else branch returns 0 when pattern is
    // not instanceof RegExp and typeof !== 'string'. An object like {invalid: true}
    // triggers this early return without throwing.
    const testFile = path.join(utils.getTempDir(), `utils-test-obj-pattern-${Date.now()}.txt`);
    try {
      utils.writeFile(testFile, 'some test content to match against');
      const count = utils.countInFile(testFile, { invalid: 'object' });
      assert.strictEqual(count, 0, 'Object pattern should return 0');
    } finally {
      try {
        fs.unlinkSync(testFile);
      } catch {
        /* best-effort */
      }
    }
  });

  // ── Round 93: countInFile with /pattern/i (g flag appended) ──
  describe('Round 93: countInFile (case-insensitive RegExp, g flag auto-appended)');

  await test('countInFile with /pattern/i appends g flag and counts case-insensitively', () => {
    // utils.js line 440: pattern.flags = 'i', 'i'.includes('g') → false,
    // so new RegExp(source, 'i' + 'g') → /pattern/ig
    const testFile = path.join(utils.getTempDir(), `utils-test-ci-flag-${Date.now()}.txt`);
    try {
      utils.writeFile(testFile, 'Foo foo FOO fOo bar baz');
      const count = utils.countInFile(testFile, /foo/i);
      assert.strictEqual(count, 4, 'Case-insensitive regex with auto-appended g should match all 4 occurrences');
    } finally {
      try {
        fs.unlinkSync(testFile);
      } catch {
        /* best-effort */
      }
    }
  });

  // ── Round 93: countInFile with /pattern/gi (g flag already present) ──
  describe('Round 93: countInFile (case-insensitive RegExp, g flag preserved)');

  await test('countInFile with /pattern/gi preserves existing flags and counts correctly', () => {
    // utils.js line 440: pattern.flags = 'gi', 'gi'.includes('g') → true,
    // so new RegExp(source, 'gi') — flags preserved unchanged
    const testFile = path.join(utils.getTempDir(), `utils-test-gi-flag-${Date.now()}.txt`);
    try {
      utils.writeFile(testFile, 'Foo foo FOO fOo bar baz');
      const count = utils.countInFile(testFile, /foo/gi);
      assert.strictEqual(count, 4, 'Case-insensitive regex with pre-existing g should match all 4 occurrences');
    } finally {
      try {
        fs.unlinkSync(testFile);
      } catch {
        /* best-effort */
      }
    }
  });

  // ── Round 95: countInFile with regex alternation (no g flag) ──
  describe('Round 95: countInFile (regex alternation without g flag)');

  await test('countInFile with /apple|banana/ (alternation, no g) counts all matches', () => {
    const tmpDir = path.join(utils.getTempDir(), `ecc-r95-alternation-${Date.now()}`);
    fs.mkdirSync(tmpDir, { recursive: true });
    const testFile = path.join(tmpDir, 'alternation.txt');
    try {
      utils.writeFile(testFile, 'apple banana apple cherry banana apple');
      // /apple|banana/ has alternation but no g flag — countInFile should auto-append g
      const count = utils.countInFile(testFile, /apple|banana/);
      assert.strictEqual(count, 5, 'Should find 3 apples + 2 bananas = 5 total (g flag auto-appended to alternation regex)');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 97: getSessionIdShort with whitespace-only CLAUDE_SESSION_ID ──
  describe('Round 97: getSessionIdShort (whitespace-only session ID)');

  await test('getSessionIdShort returns whitespace when CLAUDE_SESSION_ID is all spaces', () => {
    // utils.js line 116: if (sessionId && sessionId.length > 0) — '   ' is truthy
    // and has length > 0, so it passes the check instead of falling back.
    const original = process.env.CLAUDE_SESSION_ID;
    try {
      process.env.CLAUDE_SESSION_ID = '          '; // 10 spaces
      const result = utils.getSessionIdShort('fallback');
      // slice(-8) on 10 spaces returns 8 spaces — not the expected fallback
      assert.strictEqual(result, '        ', 'Whitespace-only ID should return 8 trailing spaces (no trim check)');
      assert.strictEqual(result.trim().length, 0, 'Result should be entirely whitespace (demonstrating the missing trim)');
    } finally {
      if (original !== undefined) {
        process.env.CLAUDE_SESSION_ID = original;
      } else {
        delete process.env.CLAUDE_SESSION_ID;
      }
    }
  });

  // ── Round 97: countInFile with same RegExp object called twice (lastIndex reuse) ──
  describe('Round 97: countInFile (RegExp lastIndex reuse validation)');

  await test('countInFile returns consistent count when same RegExp object is reused', () => {
    // utils.js lines 438-440: Always creates a new RegExp to prevent lastIndex
    // state bugs. Without this defense, a global regex's lastIndex would persist
    // between calls, causing alternating match/miss behavior.
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r97-lastindex-'));
    const testFile = path.join(tmpDir, 'test.txt');
    try {
      fs.writeFileSync(testFile, 'foo bar foo baz foo\nfoo again foo');
      const sharedRegex = /foo/g;
      // First call
      const count1 = utils.countInFile(testFile, sharedRegex);
      // Second call with SAME regex object — would fail without defensive new RegExp
      const count2 = utils.countInFile(testFile, sharedRegex);
      assert.strictEqual(count1, 5, 'First call should find 5 matches');
      assert.strictEqual(count2, 5, 'Second call with same RegExp should also find 5 (lastIndex reset by defensive code)');
      assert.strictEqual(count1, count2, 'Both calls must return identical counts (proves lastIndex is not shared)');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 98: findFiles with maxAge: -1 (negative boundary — excludes everything) ──
  describe('Round 98: findFiles (maxAge: -1 — negative boundary excludes all)');

  await test('findFiles with maxAge: -1 excludes all files (ageInDays always >= 0)', () => {
    // utils.js line 176-178: `if (maxAge !== null) { ageInDays = ...; if (ageInDays <= maxAge) }`
    // With maxAge: -1, the condition requires ageInDays <= -1. Since ageInDays =
    // (Date.now() - mtimeMs) / 86400000 is always >= 0 for real files, nothing passes.
    // This negative boundary deterministically excludes everything.
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r98-maxage-neg-'));
    try {
      fs.writeFileSync(path.join(tmpDir, 'fresh.txt'), 'created just now');
      const results = utils.findFiles(tmpDir, '*.txt', { maxAge: -1 });
      assert.strictEqual(results.length, 0, 'maxAge: -1 should exclude all files (ageInDays is always >= 0)');
      // Contrast: maxAge: null (default) should include the file
      const noMaxAge = utils.findFiles(tmpDir, '*.txt');
      assert.strictEqual(noMaxAge.length, 1, 'No maxAge (null default) should include the file (proving it exists)');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 99: replaceInFile returns true even when pattern not found ──
  describe('Round 99: replaceInFile (no-match still returns true)');

  await test('replaceInFile returns true and rewrites file even when search does not match', () => {
    // utils.js lines 405-417: replaceInFile reads content, calls content.replace(search, replace),
    // and writes back the result. When the search pattern doesn't match anything,
    // String.replace() returns the original string unchanged, but the function still
    // writes it back to disk (changing mtime) and returns true. This means callers
    // cannot distinguish "replacement made" from "no match found."
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r99-no-match-'));
    const testFile = path.join(tmpDir, 'test.txt');
    try {
      fs.writeFileSync(testFile, 'hello world');
      const result = utils.replaceInFile(testFile, 'NONEXISTENT_PATTERN', 'replacement');
      assert.strictEqual(result, true, 'replaceInFile returns true even when pattern is not found (no match guard)');
      const content = fs.readFileSync(testFile, 'utf8');
      assert.strictEqual(content, 'hello world', 'Content should be unchanged since pattern did not match');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 99: grepFile with CR-only line endings (\r without \n) ──
  describe('Round 99: grepFile (CR-only line endings — classic Mac format)');

  await test('grepFile treats CR-only file as a single line (splits on \\n only)', () => {
    // utils.js line 474: `content.split('\\n')` splits only on \\n (LF).
    // A file using \\r (CR) line endings (classic Mac format) has no \\n characters,
    // so split('\\n') returns the entire content as a single element array.
    // This means grepFile reports everything on "line 1" regardless of \\r positions.
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r99-cr-only-'));
    const testFile = path.join(tmpDir, 'cr-only.txt');
    try {
      // Write file with CR-only line endings (no LF)
      fs.writeFileSync(testFile, 'alpha\rbeta\rgamma');
      const matches = utils.grepFile(testFile, 'beta');
      assert.strictEqual(matches.length, 1, 'Should find exactly 1 match (entire file is one "line")');
      assert.strictEqual(matches[0].lineNumber, 1, 'Match should be reported on line 1 (no \\n splitting occurred)');
      assert.ok(matches[0].content.includes('\r'), 'Content should contain \\r characters (unsplit)');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 100: findFiles with both maxAge AND recursive (interaction test) ──
  describe('Round 100: findFiles (maxAge + recursive combined — untested interaction)');

  await test('findFiles with maxAge AND recursive filters age across subdirectories', () => {
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r100-maxage-recur-'));
    const subDir = path.join(tmpDir, 'nested');
    try {
      fs.mkdirSync(subDir);
      // Create files: one in root, one in subdirectory
      const rootFile = path.join(tmpDir, 'root.txt');
      const nestedFile = path.join(subDir, 'nested.txt');
      fs.writeFileSync(rootFile, 'root file');
      fs.writeFileSync(nestedFile, 'nested file');

      // maxAge: 1 with recursive: true — both files are fresh (ageInDays ≈ 0)
      const results = utils.findFiles(tmpDir, '*.txt', { maxAge: 1, recursive: true });
      assert.strictEqual(results.length, 2, 'Both root and nested files should match (fresh, maxAge: 1, recursive: true)');

      // maxAge: -1 with recursive: true — no files should match (age always >= 0)
      const noResults = utils.findFiles(tmpDir, '*.txt', { maxAge: -1, recursive: true });
      assert.strictEqual(noResults.length, 0, 'maxAge: -1 should exclude all files even in subdirectories');

      // maxAge: 1 with recursive: false — only root file
      const rootOnly = utils.findFiles(tmpDir, '*.txt', { maxAge: 1, recursive: false });
      assert.strictEqual(rootOnly.length, 1, 'recursive: false should only find root-level file');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 101: output() with circular reference object throws (no try/catch around JSON.stringify) ──
  describe('Round 101: output() (circular reference — JSON.stringify crash)');

  await test('output() throws TypeError on circular reference object (JSON.stringify has no try/catch)', () => {
    const circular = { a: 1 };
    circular.self = circular; // Creates circular reference

    assert.throws(() => utils.output(circular), { name: 'TypeError' }, 'JSON.stringify of circular object should throw TypeError (no try/catch in output())');
  });

  // ── Round 103: countInFile with boolean false pattern (non-string non-RegExp) ──
  describe('Round 103: countInFile (boolean false — explicit type guard returns 0)');

  await test('countInFile returns 0 for boolean false pattern (else branch at line 443)', () => {
    // utils.js lines 438-444: countInFile checks `instanceof RegExp` then `typeof === "string"`.
    // Boolean `false` fails both checks and falls to the `else return 0` at line 443.
    // This is the correct rejection path for non-string non-RegExp patterns, but was
    // previously untested with boolean specifically (only null, undefined, object tested).
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r103-bool-pattern-'));
    const testFile = path.join(tmpDir, 'test.txt');
    try {
      fs.writeFileSync(testFile, 'false is here\nfalse again\ntrue as well');
      // Even though "false" appears in the content, boolean `false` is rejected by type guard
      const count = utils.countInFile(testFile, false);
      assert.strictEqual(count, 0, 'Boolean false should return 0 (typeof false === "boolean", not "string")');
      // Contrast: string "false" should match normally
      const stringCount = utils.countInFile(testFile, 'false');
      assert.strictEqual(stringCount, 2, 'String "false" should match 2 times (proving content exists but type guard blocked boolean)');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 103: grepFile with numeric 0 pattern (implicit RegExp coercion) ──
  describe('Round 103: grepFile (numeric 0 — implicit toString via RegExp constructor)');

  await test('grepFile with numeric 0 implicitly coerces to /0/ via RegExp constructor', () => {
    // utils.js line 468: grepFile's non-RegExp path does `regex = new RegExp(pattern)`.
    // Unlike countInFile (which has explicit type guards), grepFile passes any value
    // to the RegExp constructor, which calls toString() on it.  So new RegExp(0)
    // becomes /0/, and grepFile actually searches for lines containing "0".
    // This contrasts with countInFile(file, 0) which returns 0 (type-rejected).
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r103-grep-numeric-'));
    const testFile = path.join(tmpDir, 'test.txt');
    try {
      fs.writeFileSync(testFile, 'line with 0 zero\nno digit here\n100 bottles');
      const matches = utils.grepFile(testFile, 0);
      assert.strictEqual(matches.length, 2, 'grepFile(file, 0) should find 2 lines containing "0" (RegExp(0) → /0/)');
      assert.strictEqual(matches[0].lineNumber, 1, 'First match on line 1 ("line with 0 zero")');
      assert.strictEqual(matches[1].lineNumber, 3, 'Second match on line 3 ("100 bottles")');
      // Contrast: countInFile with numeric 0 returns 0 (type-rejected)
      const count = utils.countInFile(testFile, 0);
      assert.strictEqual(count, 0, 'countInFile(file, 0) returns 0 — API inconsistency with grepFile');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 105: grepFile with sticky (y) flag — not stripped, causes stateful .test() ──
  describe('Round 105: grepFile (sticky y flag — not stripped like g, stateful .test() bug)');

  await test('grepFile with /pattern/y sticky flag misses lines due to lastIndex state', () => {
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r105-grep-sticky-'));
    const testFile = path.join(tmpDir, 'test.txt');
    try {
      fs.writeFileSync(testFile, 'hello world\nhello again\nhello third');
      // grepFile line 466: `pattern.flags.replace('g', '')` strips g but not y.
      // With /hello/y (sticky), .test() advances lastIndex after each successful
      // match. On the next line, .test() starts at lastIndex (not 0), so it fails
      // unless the match happens at that exact position.
      const stickyResults = utils.grepFile(testFile, /hello/y);
      // Without the bug, all 3 lines should match. With sticky flag preserved,
      // line 1 matches (lastIndex advances to 5), line 2 fails (no 'hello' at
      // position 5 of "hello again"), line 3 also likely fails.
      // The g-flag version (properly stripped) should find all 3:
      const globalResults = utils.grepFile(testFile, /hello/g);
      assert.strictEqual(globalResults.length, 3, 'g-flag regex should find all 3 lines (g is stripped, stateless)');
      // Sticky flag causes fewer matches — demonstrating the bug
      assert.ok(stickyResults.length < 3, `Sticky y flag causes stateful .test() — found ${stickyResults.length}/3 lines ` + '(y flag not stripped like g, so lastIndex advances between lines)');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 107: grepFile with ^$ pattern — empty line matching after split ──
  describe('Round 107: grepFile (empty line matching — ^$ on split lines, trailing \\n creates extra empty element)');

  await test('grepFile matches empty lines with ^$ pattern including trailing newline phantom line', () => {
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r107-grep-empty-'));
    const testFile = path.join(tmpDir, 'test.txt');
    try {
      // 'line1\n\nline3\n\n'.split('\n') → ['line1','','line3','',''] (5 elements, 3 empty)
      fs.writeFileSync(testFile, 'line1\n\nline3\n\n');
      const results = utils.grepFile(testFile, /^$/);
      assert.strictEqual(results.length, 3, 'Should match 3 empty lines: line 2, line 4, and trailing phantom line 5');
      assert.strictEqual(results[0].lineNumber, 2, 'First empty line at position 2');
      assert.strictEqual(results[1].lineNumber, 4, 'Second empty line at position 4');
      assert.strictEqual(results[2].lineNumber, 5, 'Third empty line is the trailing phantom from split');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 107: replaceInFile where replacement re-introduces search pattern (single-pass) ──
  describe('Round 107: replaceInFile (replacement contains search pattern — String.replace is single-pass)');

  await test('replaceInFile does not re-scan replacement text (single-pass, no infinite loop)', () => {
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r107-replace-reintr-'));
    const testFile = path.join(tmpDir, 'test.txt');
    try {
      fs.writeFileSync(testFile, 'foo bar baz');
      // Replace "foo" with "foo extra foo" — should only replace the first occurrence
      const result = utils.replaceInFile(testFile, 'foo', 'foo extra foo');
      assert.strictEqual(result, true, 'replaceInFile should return true');
      const content = utils.readFile(testFile);
      assert.strictEqual(content, 'foo extra foo bar baz', 'Only the original "foo" is replaced — replacement text is not re-scanned');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 106: countInFile with named capture groups — match(g) ignores group details ──
  describe('Round 106: countInFile (named capture groups — String.match(g) returns full matches only)');

  await test('countInFile with named capture groups counts matches not groups', () => {
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r106-count-named-'));
    const testFile = path.join(tmpDir, 'test.txt');
    try {
      fs.writeFileSync(testFile, 'foo bar baz\nfoo qux\nbar foo end');
      // Named capture group — should still count 3 matches for "foo"
      const count = utils.countInFile(testFile, /(?<word>foo)/);
      assert.strictEqual(count, 3, 'Named capture group should not inflate count — match(g) returns full matches only');
      // Compare with plain pattern
      const plainCount = utils.countInFile(testFile, /foo/);
      assert.strictEqual(plainCount, 3, 'Plain regex should also find 3 matches');
      assert.strictEqual(count, plainCount, 'Named group pattern and plain pattern should return identical counts');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 106: grepFile with multiline (m) flag — preserved, unlike g which is stripped ──
  describe('Round 106: grepFile (multiline m flag — preserved in regex, unlike g which is stripped)');

  await test('grepFile preserves multiline (m) flag and anchors work on split lines', () => {
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r106-grep-multiline-'));
    const testFile = path.join(tmpDir, 'test.txt');
    try {
      fs.writeFileSync(testFile, 'hello\nworld hello\nhello world');
      // With m flag + anchors: ^hello$ should match only exact "hello" line
      const mResults = utils.grepFile(testFile, /^hello$/m);
      assert.strictEqual(mResults.length, 1, 'With m flag, ^hello$ should match only line 1 (exact "hello")');
      assert.strictEqual(mResults[0].lineNumber, 1);
      // Without m flag: same behavior since grepFile splits lines individually
      const noMResults = utils.grepFile(testFile, /^hello$/);
      assert.strictEqual(noMResults.length, 1, 'Without m flag, same result — grepFile splits lines so anchors are per-line already');
      assert.strictEqual(mResults.length, noMResults.length, 'm flag is preserved but irrelevant — line splitting makes anchors per-line already');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 109: appendFile creating new file in non-existent directory (ensureDir + appendFileSync) ──
  describe('Round 109: appendFile (new file creation — ensureDir creates parent, appendFileSync creates file)');

  await test('appendFile creates parent directory and new file when neither exist', () => {
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r109-append-new-'));
    const nestedPath = path.join(tmpDir, 'deep', 'nested', 'dir', 'newfile.txt');
    try {
      // Parent directory 'deep/nested/dir' does not exist yet
      assert.ok(!fs.existsSync(path.join(tmpDir, 'deep')), 'Parent "deep" should not exist before appendFile');
      utils.appendFile(nestedPath, 'first line\n');
      assert.ok(fs.existsSync(nestedPath), 'File should be created by appendFile');
      assert.strictEqual(utils.readFile(nestedPath), 'first line\n', 'Content should match what was appended');
      // Append again to verify it adds to existing file
      utils.appendFile(nestedPath, 'second line\n');
      assert.strictEqual(utils.readFile(nestedPath), 'first line\nsecond line\n', 'Second append should add to existing file');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 108: grepFile with Unicode/emoji content — UTF-16 string matching on split lines ──
  describe('Round 108: grepFile (Unicode/emoji — regex matching on UTF-16 split lines)');

  await test('grepFile finds Unicode emoji patterns across lines', () => {
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r108-grep-unicode-'));
    const testFile = path.join(tmpDir, 'test.txt');
    try {
      fs.writeFileSync(testFile, '🎉 celebration\nnormal line\n🎉 party\n日本語テスト');
      const emojiResults = utils.grepFile(testFile, /🎉/);
      assert.strictEqual(emojiResults.length, 2, 'Should find emoji on 2 lines (lines 1 and 3)');
      assert.strictEqual(emojiResults[0].lineNumber, 1);
      assert.strictEqual(emojiResults[1].lineNumber, 3);
      const cjkResults = utils.grepFile(testFile, /日本語/);
      assert.strictEqual(cjkResults.length, 1, 'Should find CJK characters on line 4');
      assert.strictEqual(cjkResults[0].lineNumber, 4);
      assert.ok(cjkResults[0].content.includes('日本語テスト'), 'Matched line should contain full CJK text');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 110: findFiles root directory unreadable — silent empty return (not throw) ──
  describe('Round 110: findFiles (root directory unreadable — EACCES on readdirSync caught silently)');

  await test('findFiles returns empty array when root directory exists but is unreadable', () => {
    if (process.platform === 'win32' || process.getuid?.() === 0) {
      console.log('    (skipped — chmod ineffective on Windows/root)');
      return true;
    }
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r110-unreadable-root-'));
    const unreadableDir = path.join(tmpDir, 'no-read');
    fs.mkdirSync(unreadableDir);
    fs.writeFileSync(path.join(unreadableDir, 'secret.txt'), 'hidden');
    try {
      fs.chmodSync(unreadableDir, 0o000);
      // Verify dir exists but is unreadable
      assert.ok(fs.existsSync(unreadableDir), 'Directory should exist');
      // findFiles should NOT throw — catch block at line 188 handles EACCES
      const results = utils.findFiles(unreadableDir, '*');
      assert.ok(Array.isArray(results), 'Should return an array');
      assert.strictEqual(results.length, 0, 'Should return empty array when root dir is unreadable (not throw)');
      // Also test with recursive flag
      const recursiveResults = utils.findFiles(unreadableDir, '*', { recursive: true });
      assert.strictEqual(recursiveResults.length, 0, 'Recursive search on unreadable root should also return empty array');
    } finally {
      // Restore permissions before cleanup
      try {
        fs.chmodSync(unreadableDir, 0o755);
      } catch (_e) {
        /* ignore permission errors */
      }
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 113: replaceInFile with zero-width regex — inserts between every character ──
  describe('Round 113: replaceInFile (zero-width regex /(?:)/g — matches every position)');

  await test('replaceInFile with zero-width regex /(?:)/g inserts replacement at every position', () => {
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r113-zero-width-'));
    const testFile = path.join(tmpDir, 'test.txt');
    try {
      fs.writeFileSync(testFile, 'abc');
      // /(?:)/g matches at every position boundary: before 'a', between 'a'-'b', etc.
      // "abc".replace(/(?:)/g, 'X') → "XaXbXcX" (7 chars from 3)
      const result = utils.replaceInFile(testFile, /(?:)/g, 'X');
      assert.strictEqual(result, true, 'Should succeed (no error)');
      const content = utils.readFile(testFile);
      assert.strictEqual(content, 'XaXbXcX', 'Zero-width regex inserts at every position boundary');

      // Also test with /^/gm (start of each line)
      fs.writeFileSync(testFile, 'line1\nline2\nline3');
      utils.replaceInFile(testFile, /^/gm, '> ');
      const prefixed = utils.readFile(testFile);
      assert.strictEqual(prefixed, '> line1\n> line2\n> line3', '/^/gm inserts at start of each line');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 114: replaceInFile options.all is silently ignored for RegExp search ──
  describe('Round 114: replaceInFile (options.all silently ignored for RegExp search)');

  await test('replaceInFile ignores options.all when search is a RegExp — falls through to .replace()', () => {
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r114-all-regex-'));
    const testFile = path.join(tmpDir, 'test.txt');
    try {
      // File with repeated pattern: "foo bar foo baz foo"
      fs.writeFileSync(testFile, 'foo bar foo baz foo');

      // With options.all=true and a non-global RegExp:
      // Line 411: (options.all && typeof search === 'string') → false (RegExp !== string)
      // Falls through to content.replace(regex, replace) — only replaces FIRST match
      const result = utils.replaceInFile(testFile, /foo/, 'QUX', { all: true });
      assert.strictEqual(result, true, 'Should succeed');
      const content = utils.readFile(testFile);
      assert.strictEqual(content, 'QUX bar foo baz foo', 'Non-global RegExp with options.all=true should still only replace FIRST match');

      // Contrast: global RegExp replaces all regardless of options.all
      fs.writeFileSync(testFile, 'foo bar foo baz foo');
      utils.replaceInFile(testFile, /foo/g, 'QUX', { all: true });
      const globalContent = utils.readFile(testFile);
      assert.strictEqual(globalContent, 'QUX bar QUX baz QUX', 'Global RegExp replaces all matches (options.all irrelevant for RegExp)');

      // String with options.all=true — uses replaceAll, replaces ALL occurrences
      fs.writeFileSync(testFile, 'foo bar foo baz foo');
      utils.replaceInFile(testFile, 'foo', 'QUX', { all: true });
      const allContent = utils.readFile(testFile);
      assert.strictEqual(allContent, 'QUX bar QUX baz QUX', 'String with options.all=true uses replaceAll — replaces ALL occurrences');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 114: output with object containing BigInt — JSON.stringify throws ──
  describe('Round 114: output (object containing BigInt — JSON.stringify throws)');

  await test('output throws TypeError when object contains BigInt values (JSON.stringify cannot serialize)', () => {
    // Capture original console.log to prevent actual output during test
    const originalLog = console.log;

    try {
      // Plain BigInt — typeof is 'bigint', not 'object', so goes to else branch
      // console.log can handle BigInt directly (prints "42n")
      let captured = null;
      console.log = val => {
        captured = val;
      };
      utils.output(BigInt(42));
      // Node.js console.log prints BigInt as-is
      assert.strictEqual(captured, BigInt(42), 'Plain BigInt goes to else branch, logged directly');

      // Object containing BigInt — typeof is 'object', so JSON.stringify is called
      // JSON.stringify(BigInt) throws: "Do not know how to serialize a BigInt"
      console.log = originalLog; // restore before throw test
      assert.throws(
        () => utils.output({ value: BigInt(42) }),
        err => err instanceof TypeError && /BigInt/.test(err.message),
        'Object with BigInt should throw TypeError from JSON.stringify'
      );

      // Array containing BigInt — also typeof 'object'
      assert.throws(
        () => utils.output([BigInt(1), BigInt(2)]),
        err => err instanceof TypeError && /BigInt/.test(err.message),
        'Array with BigInt should also throw TypeError from JSON.stringify'
      );
    } finally {
      console.log = originalLog;
    }
  });

  // ── Round 115: countInFile with empty string pattern — matches at every position boundary ──
  describe('Round 115: countInFile (empty string pattern — matches at every zero-width position)');

  await test('countInFile with empty string pattern returns content.length + 1 (matches between every char)', () => {
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r115-empty-pattern-'));
    const testFile = path.join(tmpDir, 'test.txt');
    try {
      // "hello" is 5 chars → 6 zero-width positions: |h|e|l|l|o|
      fs.writeFileSync(testFile, 'hello');
      const count = utils.countInFile(testFile, '');
      assert.strictEqual(count, 6, 'Empty string pattern creates /(?:)/g which matches at 6 position boundaries in "hello"');

      // Empty file → "" has 1 zero-width position (the empty string itself)
      fs.writeFileSync(testFile, '');
      const emptyCount = utils.countInFile(testFile, '');
      assert.strictEqual(emptyCount, 1, 'Empty file still has 1 zero-width position boundary');

      // Single char → 2 positions: |a|
      fs.writeFileSync(testFile, 'a');
      const singleCount = utils.countInFile(testFile, '');
      assert.strictEqual(singleCount, 2, 'Single character file has 2 position boundaries');

      // Newlines count as characters too
      fs.writeFileSync(testFile, 'a\nb');
      const newlineCount = utils.countInFile(testFile, '');
      assert.strictEqual(newlineCount, 4, '"a\\nb" is 3 chars → 4 position boundaries');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 117: grepFile with CRLF content — split('\n') leaves \r, anchored patterns fail ──
  describe('Round 117: grepFile (CRLF content — trailing \\r breaks anchored regex patterns)');

  await test('grepFile with CRLF content: unanchored patterns work but anchored $ fails due to trailing \\r', () => {
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r117-grep-crlf-'));
    const testFile = path.join(tmpDir, 'test.txt');
    try {
      // Write CRLF content
      fs.writeFileSync(testFile, 'hello\r\nworld\r\nfoo bar\r\n');

      // Unanchored pattern works — 'hello' matches in 'hello\r'
      const unanchored = utils.grepFile(testFile, 'hello');
      assert.strictEqual(unanchored.length, 1, 'Unanchored pattern should find 1 match');
      assert.strictEqual(unanchored[0].lineNumber, 1, 'Should be on line 1');
      assert.ok(unanchored[0].content.endsWith('\r'), 'Line content should have trailing \\r from split("\\n") on CRLF');

      // Anchored pattern /^hello$/ does NOT match 'hello\r' because $ is before \r
      const anchored = utils.grepFile(testFile, /^hello$/);
      assert.strictEqual(anchored.length, 0, 'Anchored /^hello$/ should NOT match "hello\\r" — $ fails before \\r');

      // But /^hello\r?$/ or /^hello/ work
      const withOptCr = utils.grepFile(testFile, /^hello\r?$/);
      assert.strictEqual(withOptCr.length, 1, '/^hello\\r?$/ matches "hello\\r" because \\r? consumes the trailing CR');

      // Contrast: LF-only content works with anchored patterns
      fs.writeFileSync(testFile, 'hello\nworld\nfoo bar\n');
      const lfAnchored = utils.grepFile(testFile, /^hello$/);
      assert.strictEqual(lfAnchored.length, 1, 'LF-only content: anchored /^hello$/ matches normally');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 116: replaceInFile with null/undefined replacement — JS coerces to string ──
  describe('Round 116: replaceInFile (null/undefined replacement — JS coerces to string "null"/"undefined")');

  await test('replaceInFile with null replacement coerces to string "null" via String.replace ToString', () => {
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r116-null-replace-'));
    const testFile = path.join(tmpDir, 'test.txt');
    try {
      // null replacement → String.replace coerces null to "null"
      fs.writeFileSync(testFile, 'hello world');
      const result = utils.replaceInFile(testFile, 'world', null);
      assert.strictEqual(result, true, 'Should succeed');
      const content = utils.readFile(testFile);
      assert.strictEqual(content, 'hello null', 'null replacement is coerced to string "null" by String.replace');

      // undefined replacement → coerced to "undefined"
      fs.writeFileSync(testFile, 'hello world');
      utils.replaceInFile(testFile, 'world', undefined);
      const undefinedContent = utils.readFile(testFile);
      assert.strictEqual(undefinedContent, 'hello undefined', 'undefined replacement is coerced to string "undefined" by String.replace');

      // Contrast: empty string replacement works as expected
      fs.writeFileSync(testFile, 'hello world');
      utils.replaceInFile(testFile, 'world', '');
      const emptyContent = utils.readFile(testFile);
      assert.strictEqual(emptyContent, 'hello ', 'Empty string replacement correctly removes matched text');

      // options.all with null replacement
      fs.writeFileSync(testFile, 'foo bar foo baz foo');
      utils.replaceInFile(testFile, 'foo', null, { all: true });
      const allContent = utils.readFile(testFile);
      assert.strictEqual(allContent, 'null bar null baz null', 'replaceAll also coerces null to "null" for every occurrence');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 116: ensureDir with null path — throws wrapped TypeError ──
  describe('Round 116: ensureDir (null path — fs.existsSync(null) throws TypeError)');

  await test('ensureDir with null path throws wrapped Error from TypeError (ERR_INVALID_ARG_TYPE)', () => {
    // fs.existsSync(null) throws TypeError in modern Node.js
    // Caught by ensureDir catch block, err.code !== 'EEXIST' → re-thrown as wrapped Error
    assert.throws(
      () => utils.ensureDir(null),
      err => {
        // Should be a wrapped Error (not raw TypeError)
        assert.ok(err instanceof Error, 'Should throw an Error');
        assert.ok(err.message.includes('Failed to create directory'), 'Error message should include "Failed to create directory"');
        return true;
      },
      'ensureDir(null) should throw wrapped Error'
    );

    // undefined path — same behavior
    assert.throws(
      () => utils.ensureDir(undefined),
      err => err instanceof Error && err.message.includes('Failed to create directory'),
      'ensureDir(undefined) should also throw wrapped Error'
    );
  });

  // ── Round 118: writeFile with non-string content — TypeError propagates (no try/catch) ──
  describe('Round 118: writeFile (non-string content — TypeError propagates uncaught)');

  await test('writeFile with null/number content throws TypeError because fs.writeFileSync rejects non-string data', () => {
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r118-writefile-type-'));
    const testFile = path.join(tmpDir, 'test.txt');
    try {
      // null content → TypeError from fs.writeFileSync (data must be string/Buffer/etc.)
      assert.throws(
        () => utils.writeFile(testFile, null),
        err => err instanceof TypeError,
        'writeFile(path, null) should throw TypeError (no try/catch in writeFile)'
      );

      // undefined content → TypeError
      assert.throws(
        () => utils.writeFile(testFile, undefined),
        err => err instanceof TypeError,
        'writeFile(path, undefined) should throw TypeError'
      );

      // number content → TypeError (numbers not valid for fs.writeFileSync)
      assert.throws(
        () => utils.writeFile(testFile, 42),
        err => err instanceof TypeError,
        'writeFile(path, 42) should throw TypeError (number not a valid data type)'
      );

      // Contrast: string content works fine
      utils.writeFile(testFile, 'valid string content');
      assert.strictEqual(utils.readFile(testFile), 'valid string content', 'String content should write and read back correctly');

      // Empty string is valid
      utils.writeFile(testFile, '');
      assert.strictEqual(utils.readFile(testFile), '', 'Empty string should write correctly');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 119: appendFile with non-string content — TypeError propagates (no try/catch) ──
  describe('Round 119: appendFile (non-string content — TypeError propagates like writeFile)');

  await test('appendFile with null/number content throws TypeError (no try/catch wrapper)', () => {
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r119-appendfile-type-'));
    const testFile = path.join(tmpDir, 'test.txt');
    try {
      // Create file with initial content
      fs.writeFileSync(testFile, 'initial');

      // null content → TypeError from fs.appendFileSync
      assert.throws(
        () => utils.appendFile(testFile, null),
        err => err instanceof TypeError,
        'appendFile(path, null) should throw TypeError'
      );

      // undefined content → TypeError
      assert.throws(
        () => utils.appendFile(testFile, undefined),
        err => err instanceof TypeError,
        'appendFile(path, undefined) should throw TypeError'
      );

      // number content → TypeError
      assert.throws(
        () => utils.appendFile(testFile, 42),
        err => err instanceof TypeError,
        'appendFile(path, 42) should throw TypeError'
      );

      // Verify original content is unchanged after failed appends
      assert.strictEqual(utils.readFile(testFile), 'initial', 'File content should be unchanged after failed appends');

      // Contrast: string append works
      utils.appendFile(testFile, ' appended');
      assert.strictEqual(utils.readFile(testFile), 'initial appended', 'String append should work correctly');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 120: replaceInFile with empty string search — prepend vs insert-between-every-char ──
  describe('Round 120: replaceInFile (empty string search — replace vs replaceAll dramatic difference)');

  await test('replaceInFile with empty search: replace prepends at pos 0; replaceAll inserts between every char', () => {
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r120-empty-search-'));
    const testFile = path.join(tmpDir, 'test.txt');
    try {
      // Without options.all: .replace('', 'X') prepends at position 0
      fs.writeFileSync(testFile, 'hello');
      utils.replaceInFile(testFile, '', 'X');
      const prepended = utils.readFile(testFile);
      assert.strictEqual(prepended, 'Xhello', 'replace("", "X") should prepend X at position 0 only');

      // With options.all: .replaceAll('', 'X') inserts between every character
      fs.writeFileSync(testFile, 'hello');
      utils.replaceInFile(testFile, '', 'X', { all: true });
      const insertedAll = utils.readFile(testFile);
      assert.strictEqual(insertedAll, 'XhXeXlXlXoX', 'replaceAll("", "X") inserts X at every position boundary');

      // Empty file + empty search
      fs.writeFileSync(testFile, '');
      utils.replaceInFile(testFile, '', 'X');
      const emptyReplace = utils.readFile(testFile);
      assert.strictEqual(emptyReplace, 'X', 'Empty content + empty search: single insertion at position 0');

      // Empty file + empty search + all
      fs.writeFileSync(testFile, '');
      utils.replaceInFile(testFile, '', 'X', { all: true });
      const emptyAll = utils.readFile(testFile);
      assert.strictEqual(emptyAll, 'X', 'Empty content + replaceAll("", "X"): single position boundary → "X"');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 121: findFiles with ? glob pattern — single character wildcard ──
  describe('Round 121: findFiles (? glob pattern — converted to . regex for single char match)');

  await test('findFiles with ? glob matches single character only — test?.txt matches test1 but not test12', () => {
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r121-glob-question-'));
    try {
      // Create test files
      fs.writeFileSync(path.join(tmpDir, 'test1.txt'), 'a');
      fs.writeFileSync(path.join(tmpDir, 'testA.txt'), 'b');
      fs.writeFileSync(path.join(tmpDir, 'test12.txt'), 'c');
      fs.writeFileSync(path.join(tmpDir, 'test.txt'), 'd');

      // ? matches exactly one character
      const results = utils.findFiles(tmpDir, 'test?.txt');
      const names = results.map(r => path.basename(r.path)).sort();
      assert.ok(names.includes('test1.txt'), 'Should match test1.txt (? = single digit)');
      assert.ok(names.includes('testA.txt'), 'Should match testA.txt (? = single letter)');
      assert.ok(!names.includes('test12.txt'), 'Should NOT match test12.txt (12 is two chars)');
      assert.ok(!names.includes('test.txt'), 'Should NOT match test.txt (no char for ?)');

      // Multiple ? marks
      fs.writeFileSync(path.join(tmpDir, 'ab.txt'), 'e');
      fs.writeFileSync(path.join(tmpDir, 'abc.txt'), 'f');
      const multiResults = utils.findFiles(tmpDir, '??.txt');
      const multiNames = multiResults.map(r => path.basename(r.path));
      assert.ok(multiNames.includes('ab.txt'), '?? should match 2-char filename');
      assert.ok(!multiNames.includes('abc.txt'), '?? should NOT match 3-char filename');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 122: findFiles dot extension escaping — *.txt must not match filetxt ──
  describe('Round 122: findFiles (dot escaping — *.txt matches file.txt but not filetxt)');

  await test('findFiles escapes dots in glob pattern so *.txt only matches literal .txt extension', () => {
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r122-dot-escape-'));
    try {
      fs.writeFileSync(path.join(tmpDir, 'file.txt'), 'a');
      fs.writeFileSync(path.join(tmpDir, 'filetxt'), 'b');
      fs.writeFileSync(path.join(tmpDir, 'file.txtx'), 'c');
      fs.writeFileSync(path.join(tmpDir, 'notes.txt'), 'd');

      const results = utils.findFiles(tmpDir, '*.txt');
      const names = results.map(r => path.basename(r.path)).sort();

      assert.ok(names.includes('file.txt'), 'Should match file.txt');
      assert.ok(names.includes('notes.txt'), 'Should match notes.txt');
      assert.ok(!names.includes('filetxt'), 'Should NOT match filetxt (dot is escaped to literal, not wildcard)');
      assert.ok(!names.includes('file.txtx'), 'Should NOT match file.txtx ($ anchor requires exact end)');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 123: countInFile with overlapping patterns — match(g) is non-overlapping ──
  describe('Round 123: countInFile (overlapping patterns — String.match(/g/) is non-overlapping)');

  await test('countInFile counts non-overlapping matches only — "aaa" with /aa/g returns 1 not 2', () => {
    // utils.js line 449: `content.match(regex)` with 'g' flag returns an array of
    // non-overlapping matches. After matching "aa" starting at index 0, the engine
    // advances to index 2, where only one "a" remains — no second match.
    // This is standard JS regex behavior but can surprise users expecting overlap.
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r123-overlap-'));
    const testFile = path.join(tmpDir, 'test.txt');
    try {
      // "aaa" — a human might count 2 occurrences of "aa" (at 0,1) but match(g) finds 1
      fs.writeFileSync(testFile, 'aaa');
      const count1 = utils.countInFile(testFile, 'aa');
      assert.strictEqual(count1, 1, '"aaa".match(/aa/g) returns ["aa"] — only 1 non-overlapping match');

      // "aaaa" — 2 non-overlapping matches (at 0,2), not 3 overlapping (at 0,1,2)
      fs.writeFileSync(testFile, 'aaaa');
      const count2 = utils.countInFile(testFile, 'aa');
      assert.strictEqual(count2, 2, '"aaaa".match(/aa/g) returns ["aa","aa"] — 2 non-overlapping, not 3 overlapping');

      // "abab" with /aba/g — only 1 match (at 0), not 2 (overlapping at 0,2)
      fs.writeFileSync(testFile, 'ababab');
      const count3 = utils.countInFile(testFile, 'aba');
      assert.strictEqual(count3, 1, '"ababab".match(/aba/g) returns 1 — after match at 0, next try starts at 3');

      // RegExp object behaves the same
      fs.writeFileSync(testFile, 'aaa');
      const count4 = utils.countInFile(testFile, /aa/);
      assert.strictEqual(count4, 1, 'RegExp /aa/ also gives 1 non-overlapping match on "aaa" (g flag auto-added)');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 123: replaceInFile with $& and $$ substitution tokens in replacement string ──
  describe('Round 123: replaceInFile ($& and $$ substitution tokens in replacement)');

  await test('replaceInFile replacement string interprets $& as matched text and $$ as literal $', () => {
    // JS String.replace() interprets special patterns in the replacement string:
    //   $&  → inserts the entire matched substring
    //   $$  → inserts a literal "$" character
    //   $'  → inserts the portion after the matched substring
    //   $`  → inserts the portion before the matched substring
    // This is different from capture groups ($1, $2) already tested in Round 91.
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r123-dollar-'));
    const testFile = path.join(tmpDir, 'test.txt');
    try {
      // $& — inserts the matched text itself
      fs.writeFileSync(testFile, 'hello world');
      utils.replaceInFile(testFile, 'world', '[$&]');
      assert.strictEqual(utils.readFile(testFile), 'hello [world]', '$& in replacement inserts the matched text "world" → "[world]"');

      // $$ — inserts a literal $ sign
      fs.writeFileSync(testFile, 'price is 100');
      utils.replaceInFile(testFile, '100', '$$100');
      assert.strictEqual(utils.readFile(testFile), 'price is $100', '$$ becomes literal $ → "100" replaced with "$100"');

      // $& with options.all — applies to each match
      fs.writeFileSync(testFile, 'foo bar foo');
      utils.replaceInFile(testFile, 'foo', '($&)', { all: true });
      assert.strictEqual(utils.readFile(testFile), '(foo) bar (foo)', '$& in replaceAll inserts each respective matched text');

      // Combined $$ and $& in same replacement (3 $ + &)
      fs.writeFileSync(testFile, 'item costs 50');
      utils.replaceInFile(testFile, '50', '$$$&');
      // In replacement string: $$ → "$" then $& → "50" so result is "$50"
      assert.strictEqual(utils.readFile(testFile), 'item costs $50', '$$$& (3 dollars + ampersand) means literal $ followed by matched text → "$50"');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 124: findFiles matches dotfiles (unlike shell glob where * excludes hidden files) ──
  describe('Round 124: findFiles (* glob matches dotfiles — unlike shell globbing)');

  await test('findFiles with * pattern matches dotfiles because .* regex includes hidden files', () => {
    // In shell: `ls *` excludes .hidden files. In findFiles, `*` → `.*` regex which
    // matches ANY filename including those starting with `.`. This is a behavioral
    // difference from shell globbing that could surprise users.
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r124-dotfiles-'));
    try {
      // Create normal and hidden files
      fs.writeFileSync(path.join(tmpDir, 'normal.txt'), 'visible');
      fs.writeFileSync(path.join(tmpDir, '.hidden'), 'hidden');
      fs.writeFileSync(path.join(tmpDir, '.gitignore'), 'ignore');
      fs.writeFileSync(path.join(tmpDir, 'README.md'), 'readme');

      // * matches ALL files including dotfiles
      const allResults = utils.findFiles(tmpDir, '*');
      const names = allResults.map(r => path.basename(r.path)).sort();
      assert.ok(names.includes('.hidden'), '* should match .hidden (unlike shell glob)');
      assert.ok(names.includes('.gitignore'), '* should match .gitignore');
      assert.ok(names.includes('normal.txt'), '* should match normal.txt');
      assert.strictEqual(names.length, 4, 'Should find all 4 files including 2 dotfiles');

      // *.txt does NOT match dotfiles (because they don't end with .txt)
      const txtResults = utils.findFiles(tmpDir, '*.txt');
      assert.strictEqual(txtResults.length, 1, '*.txt should only match normal.txt, not dotfiles');

      // .* pattern specifically matches only dotfiles
      const dotResults = utils.findFiles(tmpDir, '.*');
      const dotNames = dotResults.map(r => path.basename(r.path)).sort();
      assert.ok(dotNames.includes('.hidden'), '.* matches .hidden');
      assert.ok(dotNames.includes('.gitignore'), '.* matches .gitignore');
      assert.ok(!dotNames.includes('normal.txt'), '.* should NOT match normal.txt (needs leading dot)');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 125: readFile with binary content — returns garbled UTF-8, not null ──
  describe('Round 125: readFile (binary/non-UTF8 content — garbled, not null)');

  await test('readFile with binary content returns garbled string (not null) because UTF-8 decode does not throw', () => {
    // utils.js line 285: fs.readFileSync(filePath, 'utf8') — binary data gets UTF-8 decoded.
    // Invalid byte sequences become U+FFFD replacement characters. The function does
    // NOT return null for binary files (only returns null on ENOENT/permission errors).
    // This means grepFile/countInFile would operate on corrupted content silently.
    const tmpDir = fs.mkdtempSync(path.join(utils.getTempDir(), 'r125-binary-'));
    const testFile = path.join(tmpDir, 'binary.dat');
    try {
      // Write raw binary data (invalid UTF-8 sequences)
      const binaryData = Buffer.from([0x00, 0x80, 0xff, 0xfe, 0x48, 0x65, 0x6c, 0x6c, 0x6f]);
      fs.writeFileSync(testFile, binaryData);

      const content = utils.readFile(testFile);
      assert.ok(content !== null, 'readFile should NOT return null for binary files');
      assert.ok(typeof content === 'string', 'readFile always returns a string (or null for missing files)');
      // The string contains "Hello" (bytes 0x48-0x6F) somewhere in the garbled output
      assert.ok(content.includes('Hello'), 'ASCII subset of binary data should survive UTF-8 decode');
      // Content length may differ from byte length due to multi-byte replacement chars
      assert.ok(content.length > 0, 'Non-empty content from binary file');

      // grepFile on binary file — still works but on garbled content
      const matches = utils.grepFile(testFile, 'Hello');
      assert.strictEqual(matches.length, 1, 'grepFile finds "Hello" even in binary file (ASCII bytes survive)');

      // Non-existent file — returns null (contrast with binary)
      const missing = utils.readFile(path.join(tmpDir, 'no-such-file.txt'));
      assert.strictEqual(missing, null, 'Missing file returns null (not garbled content)');
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  // ── Round 125: output() with undefined, NaN, Infinity — non-object primitives logged directly ──
  describe('Round 125: output() (undefined/NaN/Infinity — typeof checks and JSON.stringify)');

  await test('output() handles undefined, NaN, Infinity as non-objects — logs directly', () => {
    // utils.js line 273: `if (typeof data === 'object')` — undefined/NaN/Infinity are NOT objects.
    // typeof undefined → "undefined", typeof NaN → "number", typeof Infinity → "number"
    // All three bypass JSON.stringify and go to console.log(data) directly.
    const origLog = console.log;
    const logged = [];
    console.log = (...args) => logged.push(args);
    try {
      // undefined — typeof "undefined", logged directly
      utils.output(undefined);
      assert.strictEqual(logged[0][0], undefined, 'output(undefined) logs undefined (not "undefined" string)');

      // NaN — typeof "number", logged directly
      utils.output(NaN);
      assert.ok(Number.isNaN(logged[1][0]), 'output(NaN) logs NaN directly (typeof "number", not "object")');

      // Infinity — typeof "number", logged directly
      utils.output(Infinity);
      assert.strictEqual(logged[2][0], Infinity, 'output(Infinity) logs Infinity directly');

      // Object containing NaN — JSON.stringify converts NaN to null
      utils.output({ value: NaN, count: Infinity });
      const parsed = JSON.parse(logged[3][0]);
      assert.strictEqual(parsed.value, null, 'JSON.stringify converts NaN to null inside objects');
      assert.strictEqual(parsed.count, null, 'JSON.stringify converts Infinity to null inside objects');
    } finally {
      console.log = origLog;
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
