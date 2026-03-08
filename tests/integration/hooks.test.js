/**
 * Integration tests for hook scripts
 *
 * Tests hook behavior in realistic scenarios with proper input/output handling.
 *
 * Run with: node tests/integration/hooks.test.js
 */

const assert = require('assert');
const path = require('path');
const fs = require('fs');
const os = require('os');
const { spawn } = require('child_process');
const { test, describe } = require('../harness');

/**
 * Run a hook script with simulated Claude Code input
 * @param {string} scriptPath - Path to the hook script
 * @param {object} input - Hook input object (will be JSON stringified)
 * @param {object} env - Environment variables
 * @returns {Promise<{code: number, stdout: string, stderr: string}>}
 */
function runHookWithInput(scriptPath, input = {}, env = {}, timeoutMs = 10000) {
  return new Promise((resolve, reject) => {
    const proc = spawn('node', [scriptPath], {
      env: { ...process.env, ...env },
      stdio: ['pipe', 'pipe', 'pipe']
    });

    let stdout = '';
    let stderr = '';

    proc.stdout.on('data', data => (stdout += data));
    proc.stderr.on('data', data => (stderr += data));

    // Ignore EPIPE/EOF errors (process may exit before we finish writing)
    // Windows uses EOF instead of EPIPE for closed pipe writes
    proc.stdin.on('error', err => {
      if (err.code !== 'EPIPE' && err.code !== 'EOF') {
        reject(err);
      }
    });

    // Send JSON input on stdin (simulating Claude Code hook invocation)
    if (input && Object.keys(input).length > 0) {
      proc.stdin.write(JSON.stringify(input));
    }
    proc.stdin.end();

    const timer = setTimeout(() => {
      proc.kill('SIGKILL');
      reject(new Error(`Hook timed out after ${timeoutMs}ms`));
    }, timeoutMs);

    proc.on('close', code => {
      clearTimeout(timer);
      resolve({ code, stdout, stderr });
    });

    proc.on('error', err => {
      clearTimeout(timer);
      reject(err);
    });
  });
}

/**
 * Run an inline hook command (like those in hooks.json)
 * @param {string} command - The node -e "..." command
 * @param {object} input - Hook input object
 * @param {object} env - Environment variables
 */
function _runInlineHook(command, input = {}, env = {}, timeoutMs = 10000) {
  return new Promise((resolve, reject) => {
    // Extract the code from node -e "..."
    const match = command.match(/^node -e "(.+)"$/s);
    if (!match) {
      reject(new Error('Invalid inline hook command format'));
      return;
    }

    const proc = spawn('node', ['-e', match[1]], {
      env: { ...process.env, ...env },
      stdio: ['pipe', 'pipe', 'pipe']
    });

    let stdout = '';
    let stderr = '';
    let timer;

    proc.stdout.on('data', data => (stdout += data));
    proc.stderr.on('data', data => (stderr += data));

    // Ignore EPIPE errors (process may exit before we finish writing)
    proc.stdin.on('error', err => {
      if (err.code !== 'EPIPE') {
        if (timer) clearTimeout(timer);
        reject(err);
      }
    });

    if (input && Object.keys(input).length > 0) {
      proc.stdin.write(JSON.stringify(input));
    }
    proc.stdin.end();

    timer = setTimeout(() => {
      proc.kill('SIGKILL');
      reject(new Error(`Inline hook timed out after ${timeoutMs}ms`));
    }, timeoutMs);

    proc.on('close', code => {
      clearTimeout(timer);
      resolve({ code, stdout, stderr });
    });

    proc.on('error', err => {
      clearTimeout(timer);
      reject(err);
    });
  });
}

// Create a temporary test directory
function createTestDir() {
  return fs.mkdtempSync(path.join(os.tmpdir(), 'hook-integration-test-'));
}

// Clean up test directory
function cleanupTestDir(testDir) {
  fs.rmSync(testDir, { recursive: true, force: true });
}

// Test suite
async function runTests() {
  describe('Hook Integration Tests');

  const scriptsDir = path.join(__dirname, '..', '..', 'dist', 'hooks');
  const hooksJsonPath = path.join(__dirname, '..', '..', 'hooks', 'hooks.json');
  const hooks = JSON.parse(fs.readFileSync(hooksJsonPath, 'utf8'));

  // ==========================================
  // Input Format Tests
  // ==========================================
  describe('Hook Input Format Handling');

  await test('hooks handle empty stdin gracefully', async () => {
    const result = await runHookWithInput(path.join(scriptsDir, 'session-start.js'), {});
    assert.strictEqual(result.code, 0, `Should exit 0, got ${result.code}`);
  });

  await test('hooks handle malformed JSON input', async () => {
    const proc = spawn('node', [path.join(scriptsDir, 'session-start.js')], {
      stdio: ['pipe', 'pipe', 'pipe']
    });

    let code = null;
    proc.stdin.write('{ invalid json }');
    proc.stdin.end();

    await new Promise(resolve => {
      proc.on('close', c => {
        code = c;
        resolve();
      });
    });

    // Hook should not crash on malformed input (exit 0)
    assert.strictEqual(code, 0, 'Should handle malformed JSON gracefully');
  });

  await test('hooks parse valid tool_input correctly', async () => {
    // Test the console.log warning hook with valid input
    const command =
      "node -e \"const fs=require('fs');let d='';process.stdin.on('data',c=>d+=c);process.stdin.on('end',()=>{const i=JSON.parse(d);const p=i.tool_input?.file_path||'';console.log('Path:',p)})\"";
    const match = command.match(/^node -e "(.+)"$/s);

    const proc = spawn('node', ['-e', match[1]], {
      stdio: ['pipe', 'pipe', 'pipe']
    });

    let stdout = '';
    proc.stdout.on('data', data => (stdout += data));

    proc.stdin.write(
      JSON.stringify({
        tool_input: { file_path: '/test/path.js' }
      })
    );
    proc.stdin.end();

    await new Promise(resolve => proc.on('close', resolve));

    assert.ok(stdout.includes('/test/path.js'), 'Should extract file_path from input');
  });

  // ==========================================
  // Output Format Tests
  // ==========================================
  describe('Hook Output Format');

  await test('hooks output messages to stderr (not stdout)', async () => {
    const result = await runHookWithInput(path.join(scriptsDir, 'session-start.js'), {});
    // Session-start should write info to stderr
    assert.ok(result.stderr.length > 0, 'Should have stderr output');
    assert.ok(result.stderr.includes('[SessionStart]'), 'Should have [SessionStart] prefix');
  });

  await test('PreCompact hook logs to stderr', async () => {
    const result = await runHookWithInput(path.join(scriptsDir, 'pre-compact.js'), {});
    assert.ok(result.stderr.includes('[PreCompact]'), 'Should output to stderr with prefix');
  });

  await test('blocking hooks output BLOCKED message', async () => {
    // Test the dev server blocking hook script directly
    const hookScript = path.join(scriptsDir, 'pre-bash-dev-server-block.js');

    const proc = spawn('node', [hookScript], {
      stdio: ['pipe', 'pipe', 'pipe']
    });

    let stderr = '';
    let code = null;
    proc.stderr.on('data', data => (stderr += data));

    // Send a dev server command so the hook triggers the block
    proc.stdin.write(
      JSON.stringify({
        tool_input: { command: 'npm run dev' }
      })
    );
    proc.stdin.end();

    await new Promise(resolve => {
      proc.on('close', c => {
        code = c;
        resolve();
      });
    });

    // Hook only blocks on non-Windows platforms (tmux is Unix-only)
    if (process.platform === 'win32') {
      assert.strictEqual(code, 0, 'On Windows, hook should not block (exit 0)');
    } else {
      assert.ok(stderr.includes('BLOCKED'), 'Blocking hook should output BLOCKED');
      assert.strictEqual(code, 2, 'Blocking hook should exit with code 2');
    }
  });

  // ==========================================
  // Exit Code Tests
  // ==========================================
  describe('Hook Exit Codes');

  await test('non-blocking hooks exit with code 0', async () => {
    const result = await runHookWithInput(path.join(scriptsDir, 'session-end.js'), {});
    assert.strictEqual(result.code, 0, 'Non-blocking hook should exit 0');
  });

  await test('blocking hooks exit with code 2', async () => {
    // The dev server blocker blocks when a dev server command is detected
    const hookScript = path.join(scriptsDir, 'pre-bash-dev-server-block.js');

    const proc = spawn('node', [hookScript], {
      stdio: ['pipe', 'pipe', 'pipe']
    });

    let code = null;
    proc.stdin.write(
      JSON.stringify({
        tool_input: { command: 'yarn dev' }
      })
    );
    proc.stdin.end();

    await new Promise(resolve => {
      proc.on('close', c => {
        code = c;
        resolve();
      });
    });

    // Hook only blocks on non-Windows platforms (tmux is Unix-only)
    if (process.platform === 'win32') {
      assert.strictEqual(code, 0, 'On Windows, hook should not block (exit 0)');
    } else {
      assert.strictEqual(code, 2, 'Blocking hook should exit 2');
    }
  });

  await test('hooks handle missing files gracefully', async () => {
    const testDir = createTestDir();
    const transcriptPath = path.join(testDir, 'nonexistent.jsonl');

    try {
      const result = await runHookWithInput(path.join(scriptsDir, 'evaluate-session.js'), { transcript_path: transcriptPath });

      // Should not crash, just skip processing
      assert.strictEqual(result.code, 0, 'Should exit 0 for missing file');
    } finally {
      cleanupTestDir(testDir);
    }
  });

  // ==========================================
  // Realistic Scenario Tests
  // ==========================================
  describe('Realistic Scenarios');

  await test('suggest-compact increments and triggers at threshold', async () => {
    const sessionId = 'integration-test-' + Date.now();
    const counterFile = path.join(os.tmpdir(), `claude-tool-count-${sessionId}`);

    try {
      // Set counter just below threshold
      fs.writeFileSync(counterFile, '49');

      const result = await runHookWithInput(path.join(scriptsDir, 'suggest-compact.js'), {}, { CLAUDE_SESSION_ID: sessionId, COMPACT_THRESHOLD: '50' });

      assert.ok(result.stderr.includes('50 tool calls'), 'Should suggest compact at threshold');
    } finally {
      if (fs.existsSync(counterFile)) fs.unlinkSync(counterFile);
    }
  });

  await test('evaluate-session processes transcript with sufficient messages', async () => {
    const testDir = createTestDir();
    const transcriptPath = path.join(testDir, 'transcript.jsonl');

    // Create a transcript with 15 user messages
    const messages = Array(15)
      .fill(null)
      .map((_, i) => ({
        type: 'user',
        content: `Test message ${i + 1}`
      }));

    fs.writeFileSync(transcriptPath, messages.map(m => JSON.stringify(m)).join('\n'));

    try {
      const result = await runHookWithInput(path.join(scriptsDir, 'evaluate-session.js'), { transcript_path: transcriptPath });

      assert.ok(result.stderr.includes('15 messages'), 'Should process session');
    } finally {
      cleanupTestDir(testDir);
    }
  });

  await test('PostToolUse PR hook extracts PR URL', async () => {
    // Test the PR hook script directly
    const hookScript = path.join(scriptsDir, 'post-bash-pr-created.js');

    const proc = spawn('node', [hookScript], {
      stdio: ['pipe', 'pipe', 'pipe']
    });

    let stderr = '';
    proc.stderr.on('data', data => (stderr += data));

    // Simulate gh pr create output
    proc.stdin.write(
      JSON.stringify({
        tool_input: { command: 'gh pr create --title "Test"' },
        tool_output: { output: 'Creating pull request...\nhttps://github.com/owner/repo/pull/123' }
      })
    );
    proc.stdin.end();

    await new Promise(resolve => proc.on('close', resolve));

    assert.ok(stderr.includes('PR created') || stderr.includes('github.com'), 'Should extract and log PR URL');
  });

  // ==========================================
  // Session End Transcript Parsing Tests
  // ==========================================
  describe('Session End Transcript Parsing');

  await test('session-end extracts summary from mixed JSONL formats', async () => {
    const testDir = createTestDir();
    const transcriptPath = path.join(testDir, 'mixed-transcript.jsonl');

    // Create transcript with both direct tool_use and nested assistant message formats
    const lines = [
      JSON.stringify({ type: 'user', content: 'Fix the login bug' }),
      JSON.stringify({ type: 'tool_use', name: 'Read', input: { file_path: 'src/auth.ts' } }),
      JSON.stringify({ type: 'assistant', message: { content: [{ type: 'tool_use', name: 'Edit', input: { file_path: 'src/auth.ts' } }] } }),
      JSON.stringify({ type: 'user', content: 'Now add tests' }),
      JSON.stringify({
        type: 'assistant',
        message: {
          content: [
            { type: 'tool_use', name: 'Write', input: { file_path: 'tests/auth.test.ts' } },
            { type: 'text', text: 'Here are the tests' }
          ]
        }
      }),
      JSON.stringify({ type: 'user', content: 'Looks good, commit' })
    ];
    fs.writeFileSync(transcriptPath, lines.join('\n'));

    try {
      const result = await runHookWithInput(path.join(scriptsDir, 'session-end.js'), { transcript_path: transcriptPath }, { HOME: testDir, USERPROFILE: testDir });

      assert.strictEqual(result.code, 0, 'Should exit 0');
      assert.ok(result.stderr.includes('[SessionEnd]'), 'Should have SessionEnd log');

      // Verify a session file was created
      const sessionsDir = path.join(testDir, '.claude', 'sessions');
      if (fs.existsSync(sessionsDir)) {
        const files = fs.readdirSync(sessionsDir).filter(f => f.endsWith('.tmp'));
        assert.ok(files.length > 0, 'Should create a session file');

        // Verify session content includes tasks from user messages
        const content = fs.readFileSync(path.join(sessionsDir, files[0]), 'utf8');
        assert.ok(content.includes('Fix the login bug'), 'Should include first user message');
        assert.ok(content.includes('auth.ts'), 'Should include modified files');
      }
    } finally {
      cleanupTestDir(testDir);
    }
  });

  await test('session-end handles transcript with malformed lines gracefully', async () => {
    const testDir = createTestDir();
    const transcriptPath = path.join(testDir, 'malformed-transcript.jsonl');

    const lines = [
      JSON.stringify({ type: 'user', content: 'Task 1' }),
      '{broken json here',
      JSON.stringify({ type: 'user', content: 'Task 2' }),
      '{"truncated":',
      JSON.stringify({ type: 'user', content: 'Task 3' })
    ];
    fs.writeFileSync(transcriptPath, lines.join('\n'));

    try {
      const result = await runHookWithInput(path.join(scriptsDir, 'session-end.js'), { transcript_path: transcriptPath }, { HOME: testDir, USERPROFILE: testDir });

      assert.strictEqual(result.code, 0, 'Should exit 0 despite malformed lines');
      // Should still process the valid lines
      assert.ok(result.stderr.includes('[SessionEnd]'), 'Should have SessionEnd log');
      assert.ok(result.stderr.includes('unparseable'), 'Should warn about unparseable lines');
    } finally {
      cleanupTestDir(testDir);
    }
  });

  await test('session-end creates session file with nested user messages', async () => {
    const testDir = createTestDir();
    const transcriptPath = path.join(testDir, 'nested-transcript.jsonl');

    // Claude Code JSONL format uses nested message.content arrays
    const lines = [
      JSON.stringify({ type: 'user', message: { role: 'user', content: [{ type: 'text', text: 'Refactor the utils module' }] } }),
      JSON.stringify({ type: 'assistant', message: { content: [{ type: 'tool_use', name: 'Read', input: { file_path: 'lib/utils.js' } }] } }),
      JSON.stringify({ type: 'user', message: { role: 'user', content: 'Approve the changes' } })
    ];
    fs.writeFileSync(transcriptPath, lines.join('\n'));

    try {
      const result = await runHookWithInput(path.join(scriptsDir, 'session-end.js'), { transcript_path: transcriptPath }, { HOME: testDir, USERPROFILE: testDir });

      assert.strictEqual(result.code, 0, 'Should exit 0');

      // Check session file was created
      const sessionsDir = path.join(testDir, '.claude', 'sessions');
      if (fs.existsSync(sessionsDir)) {
        const files = fs.readdirSync(sessionsDir).filter(f => f.endsWith('.tmp'));
        assert.ok(files.length > 0, 'Should create session file');
        const content = fs.readFileSync(path.join(sessionsDir, files[0]), 'utf8');
        assert.ok(content.includes('Refactor the utils module') || content.includes('Approve'), 'Should extract user messages from nested format');
      }
    } finally {
      cleanupTestDir(testDir);
    }
  });

  // ==========================================
  // Error Handling Tests
  // ==========================================
  describe('Error Handling');

  await test('hooks do not crash on unexpected input structure', async () => {
    const result = await runHookWithInput(path.join(scriptsDir, 'suggest-compact.js'), { unexpected: { nested: { deeply: 'value' } } });

    assert.strictEqual(result.code, 0, 'Should handle unexpected input structure');
  });

  await test('hooks handle null and missing values in input', async () => {
    const result = await runHookWithInput(path.join(scriptsDir, 'session-start.js'), { tool_input: null });

    assert.strictEqual(result.code, 0, 'Should handle null/missing values gracefully');
  });

  await test('hooks handle very large input without hanging', async () => {
    const largeInput = {
      tool_input: { file_path: '/test.js' },
      tool_output: { output: 'x'.repeat(100000) }
    };

    const startTime = Date.now();
    const result = await runHookWithInput(path.join(scriptsDir, 'session-start.js'), largeInput);
    const elapsed = Date.now() - startTime;

    assert.strictEqual(result.code, 0, 'Should complete successfully');
    assert.ok(elapsed < 5000, `Should complete in <5s, took ${elapsed}ms`);
  });

  await test('hooks survive stdin exceeding 1MB limit', async () => {
    // The post-edit-console-warn hook reads stdin up to 1MB then passes through
    // Send > 1MB to verify truncation doesn't crash the hook
    const oversizedInput = JSON.stringify({
      tool_input: { file_path: '/test.js' },
      tool_output: { output: 'x'.repeat(1200000) } // ~1.2MB
    });

    const proc = spawn('node', [path.join(scriptsDir, 'post-edit-console-warn.js')], {
      stdio: ['pipe', 'pipe', 'pipe']
    });

    let code = null;
    // MUST drain stdout/stderr to prevent backpressure blocking the child process
    proc.stdout.on('data', () => {});
    proc.stderr.on('data', () => {});
    proc.stdin.on('error', err => {
      if (err.code !== 'EPIPE' && err.code !== 'EOF') throw err;
    });
    proc.stdin.write(oversizedInput);
    proc.stdin.end();

    await new Promise(resolve => {
      proc.on('close', c => {
        code = c;
        resolve();
      });
    });

    assert.strictEqual(code, 0, 'Should exit 0 despite oversized input');
  });

  await test('hooks handle truncated JSON from overflow gracefully', async () => {
    // session-end parses stdin JSON. If input is > 1MB and truncated mid-JSON,
    // JSON.parse should fail and fall back to env var
    const proc = spawn('node', [path.join(scriptsDir, 'session-end.js')], {
      stdio: ['pipe', 'pipe', 'pipe']
    });

    let code = null;
    let stderr = '';
    // MUST drain stdout to prevent backpressure blocking the child process
    proc.stdout.on('data', () => {});
    proc.stderr.on('data', data => (stderr += data));
    proc.stdin.on('error', err => {
      if (err.code !== 'EPIPE' && err.code !== 'EOF') throw err;
    });

    // Build a string that will be truncated mid-JSON at 1MB
    const bigValue = 'x'.repeat(1200000);
    proc.stdin.write(`{"transcript_path":"/tmp/none","padding":"${bigValue}"}`);
    proc.stdin.end();

    await new Promise(resolve => {
      proc.on('close', c => {
        code = c;
        resolve();
      });
    });

    // Should exit 0 even if JSON parse fails (falls back to env var or null)
    assert.strictEqual(code, 0, 'Should not crash on truncated JSON');
  });

  // ==========================================
  // Round 51: Timeout Enforcement
  // ==========================================
  describe('Round 51: Timeout Enforcement');

  await test('runHookWithInput kills hanging hooks after timeout', async () => {
    const testDir = createTestDir();
    const hangingHookPath = path.join(testDir, 'hanging-hook.js');
    fs.writeFileSync(hangingHookPath, 'setInterval(() => {}, 100);');

    try {
      const startTime = Date.now();
      let error = null;

      try {
        await runHookWithInput(hangingHookPath, {}, {}, 500);
      } catch (err) {
        error = err;
      }

      const elapsed = Date.now() - startTime;
      assert.ok(error, 'Should throw timeout error');
      assert.ok(error.message.includes('timed out'), 'Error should mention timeout');
      assert.ok(elapsed >= 450, `Should wait at least ~500ms, waited ${elapsed}ms`);
      assert.ok(elapsed < 2000, `Should not wait much longer than 500ms, waited ${elapsed}ms`);
    } finally {
      cleanupTestDir(testDir);
    }
  });

  // ==========================================
  // Round 51: hooks.json Schema Validation
  // ==========================================
  describe('Round 51: hooks.json Schema Validation');

  await test('hooks.json async hook has valid timeout field', async () => {
    const asyncHook = hooks.hooks.PostToolUse.find(h => h.hooks && h.hooks[0] && h.hooks[0].async === true);

    assert.ok(asyncHook, 'Should have at least one async hook defined');
    assert.strictEqual(asyncHook.hooks[0].async, true, 'async field should be true');
    assert.ok(asyncHook.hooks[0].timeout, 'Should have timeout field');
    assert.strictEqual(typeof asyncHook.hooks[0].timeout, 'number', 'Timeout should be a number');
    assert.ok(asyncHook.hooks[0].timeout > 0, 'Timeout should be positive');

    const isValid = asyncHook.hooks[0].command.startsWith('node ') || asyncHook.hooks[0].command.startsWith('bash ');
    assert.ok(isValid, 'Async hook command should be node or bash format');
  });

  await test('all hook commands in hooks.json are valid format', async () => {
    for (const [hookType, hookArray] of Object.entries(hooks.hooks)) {
      for (const hookDef of hookArray) {
        assert.ok(hookDef.hooks, `${hookType} entry should have hooks array`);

        for (const hook of hookDef.hooks) {
          assert.ok(hook.command, `Hook in ${hookType} should have command field`);

          const isNode = hook.command.startsWith('node ');
          const isBash = hook.command.startsWith('bash ');

          assert.ok(isNode || isBash, `Hook command in ${hookType} should start with node or bash, got: ${hook.command.substring(0, 80)}`);
        }
      }
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
