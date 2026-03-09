#!/usr/bin/env node
'use strict';

// Thin wrapper that resolves ECC_ROOT from the package location,
// then delegates to dist/hooks/run-with-flags.js.
// Registered as a bin entry so hooks.json can use `ecc-hook` instead of
// relying on an ECC_ROOT environment variable.

const path = require('path');

const pkgRoot = path.resolve(__dirname, '..');
process.env.ECC_ROOT = process.env.ECC_ROOT || pkgRoot;

require('../dist/hooks/run-with-flags.js');
