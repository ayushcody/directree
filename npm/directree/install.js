const os = require('os');
const fs = require('fs');

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
  console.error(`directree unsupported platform: ${platform}-${arch}. Installation may fail at runtime.`);
  process.exit(0);
}

try {
  const exeName = platform === 'win32' ? 'directree.exe' : 'directree';
  const exePath = require.resolve(`${packageName}/bin/${exeName}`);
  if (fs.existsSync(exePath)) {
    console.log(`directree native binary found at ${exePath}`);
  } else {
    console.warn(`directree native binary not found at expected path: ${exePath}`);
  }
} catch (e) {
  console.warn(`directree: Could not resolve optional dependency ${packageName}. Ensure it installed correctly.`);
}
