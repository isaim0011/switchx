#!/usr/bin/env node
const { spawn } = require('child_process');
const path = require('path');
const os = require('os');
const fs = require('fs');

function findBinary() {
  const binaryName = os.platform() === 'win32' ? 'switchx.exe' : 'switchx';
  // 1. Check local cargo target build
  const localBuild = path.resolve(__dirname, '..', '..', 'target', 'release', binaryName);
  if (fs.existsSync(localBuild)) return localBuild;
  const localDebug = path.resolve(__dirname, '..', '..', 'target', 'debug', binaryName);
  if (fs.existsSync(localDebug)) return localDebug;
  return binaryName;
}

const bin = findBinary();
const args = process.argv.slice(2);

const child = spawn(bin, args, { stdio: 'inherit' });
child.on('error', (err) => {
  if (err.code === 'ENOENT') {
    console.log(`\n\x1b[36mSwitchX Native Core not found on PATH.\x1b[0m`);
    console.log(`To install the high-speed Rust binary globally, run:\n`);
    console.log(`  \x1b[32mcargo install switchx\x1b[0m\n`);
    console.log(`Repository: https://github.com/isaim0011/switchx\n`);
  } else {
    console.error(err);
  }
  process.exit(1);
});

child.on('close', (code) => {
  process.exit(code || 0);
});
