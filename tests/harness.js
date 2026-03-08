/**
 * Shared test harness for single-process runner.
 * Replaces per-file inline test() helpers with centralized tracking.
 */

const results = { passed: 0, failed: 0, errors: [] };

/**
 * Run a single test (sync or async). Tracks pass/fail automatically.
 */
async function test(name, fn) {
  try {
    const result = fn();
    if (result && typeof result.then === 'function') {
      await result;
    }
    console.log(`  \u2713 ${name}`);
    results.passed++;
    return true;
  } catch (err) {
    console.log(`  \u2717 ${name}`);
    console.log(`    Error: ${err.message}`);
    results.failed++;
    results.errors.push({ name, error: err.message });
    return false;
  }
}

/**
 * Print a describe/section header.
 */
function describe(suiteName) {
  console.log(`\n${suiteName}:`);
}

/**
 * Get current results snapshot.
 */
function getResults() {
  return { passed: results.passed, failed: results.failed, errors: [...results.errors] };
}

/**
 * Reset counters between test files.
 */
function resetCounters() {
  results.passed = 0;
  results.failed = 0;
  results.errors = [];
}

module.exports = { test, describe, getResults, resetCounters };
