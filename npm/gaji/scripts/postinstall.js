const PLATFORMS = {
  "linux-x64": "@gaji/linux-x64",
  "linux-arm64": "@gaji/linux-arm64",
  "darwin-x64": "@gaji/darwin-x64",
  "darwin-arm64": "@gaji/darwin-arm64",
  "win32-x64": "@gaji/win32-x64",
};

const platformKey = `${process.platform}-${process.arch}`;
const packageName = PLATFORMS[platformKey];

if (!packageName) {
  console.warn(
    `[gaji] Warning: Unsupported platform ${platformKey}. ` +
      `The CLI binary will not be available. ` +
      `Install from source: cargo install gaji`
  );
  process.exit(0);
}

try {
  require.resolve(`${packageName}/package.json`);
} catch (_) {
  console.warn(
    `[gaji] Warning: Platform package ${packageName} was not installed. ` +
      `This can happen with --no-optional. ` +
      `The CLI binary will not work, but the runtime library is still available.`
  );
}
