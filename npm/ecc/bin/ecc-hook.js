#!/usr/bin/env node

"use strict";

const { execFileSync } = require("child_process");
const { join } = require("path");

// Re-use the same binary resolution as ecc.js
const PLATFORMS = {
  "darwin-arm64": "@lebocqtitouan/ecc-darwin-arm64",
  "darwin-x64": "@lebocqtitouan/ecc-darwin-x64",
  "linux-x64": "@lebocqtitouan/ecc-linux-x64",
  "linux-arm64": "@lebocqtitouan/ecc-linux-arm64",
  "win32-x64": "@lebocqtitouan/ecc-win32-x64",
};

function getBinaryPath() {
  const key = `${process.platform}-${process.arch}`;
  const pkg = PLATFORMS[key];
  if (!pkg) {
    throw new Error(`Unsupported platform: ${key}`);
  }
  const binDir = require.resolve(`${pkg}/package.json`);
  const ext = process.platform === "win32" ? ".exe" : "";
  return join(binDir, "..", `ecc${ext}`);
}

try {
  // Forward stdin for hook payloads
  execFileSync(getBinaryPath(), ["hook", ...process.argv.slice(2)], {
    stdio: "inherit",
  });
} catch (err) {
  if (err.status !== undefined) {
    process.exit(err.status);
  }
  throw err;
}
