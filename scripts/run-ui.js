#!/usr/bin/env node

const { spawnSync } = require('child_process');
const path = require('path');

const uiDir = path.resolve(__dirname, '..', 'crates', 'ui');
const trunk = process.platform === 'win32' ? 'trunk.exe' : 'trunk';
const args = [
  'serve',
  'index.html',
  '--address', '127.0.0.1',
  '--prefer-address-family', 'ipv4',
  '--disable-address-lookup', 'true',
  '--port', '8088'
];

if (process.env.RESONANCE_NO_OPEN !== '1') args.push('--open');

const result = spawnSync(trunk, args, {
  cwd: uiDir,
  stdio: 'inherit',
  shell: false
});

if (result.error) {
  console.error(`Unable to start the Resonance UI: ${result.error.message}`);
  process.exitCode = 1;
} else {
  process.exitCode = result.status ?? 1;
}
