#!/usr/bin/env node

const os = require('os');
const path = require('path');
const { spawnSync } = require('child_process');

const platform = os.platform();
const arch = os.arch();

const knownPlatforms = {
  'win32-x64': '@directree/win32-x64',
  'darwin-arm64': '@directree/darwin-arm64',
  'darwin-x64': '@directree/darwin-x64',
  'linux-x64': '@directree/linux-x64',
};

const packageName = knownPlatforms[`${platform}-${arch}`];

if (!packageName) {
  console.error(`Unsupported platform: ${platform}-${arch}`);
  process.exit(1);
}

const exeName = platform === 'win32' ? 'directree.exe' : 'directree';

try {
  const exePath = require.resolve(`${packageName}/bin/${exeName}`);
  const result = spawnSync(exePath, process.argv.slice(2), { stdio: 'inherit' });
  if (result.error) {
    throw result.error;
  }
  process.exit(result.status ?? 0);
} catch (e) {
  console.error(`Failed to start directree: ${e.message}`);
  process.exit(1);
}
