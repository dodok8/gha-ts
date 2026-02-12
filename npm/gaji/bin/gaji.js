#!/usr/bin/env node

const { execFileSync } = require("child_process");
const path = require("path");
const fs = require("fs");

const PLATFORMS = {
  "linux-x64": "@gaji/linux-x64",
  "linux-arm64": "@gaji/linux-arm64",
  "darwin-x64": "@gaji/darwin-x64",
  "darwin-arm64": "@gaji/darwin-arm64",
  "win32-x64": "@gaji/win32-x64",
};

function getBinaryPath() {
  const override = process.env.GAJI_BINARY_PATH;
  if (override) {
    return override;
  }

  const platformKey = `${process.platform}-${process.arch}`;
  const packageName = PLATFORMS[platformKey];

  if (!packageName) {
    console.error(
      `Unsupported platform: ${platformKey}\n` +
        `gaji supports: ${Object.keys(PLATFORMS).join(", ")}\n` +
        `You can install from source: cargo install gaji`
    );
    process.exit(1);
  }

  const binName = process.platform === "win32" ? "gaji.exe" : "gaji";

  try {
    const pkgPath = require.resolve(`${packageName}/package.json`);
    const binPath = path.join(path.dirname(pkgPath), "bin", binName);
    if (fs.existsSync(binPath)) {
      return binPath;
    }
  } catch (_) {
    // Package not found
  }

  console.error(
    `Could not find gaji binary for ${platformKey}.\n` +
      `The platform package ${packageName} may not have been installed.\n` +
      `Try: npm install ${packageName}\n` +
      `Or install from source: cargo install gaji`
  );
  process.exit(1);
}

try {
  const binPath = getBinaryPath();
  execFileSync(binPath, process.argv.slice(2), {
    stdio: "inherit",
    env: process.env,
  });
} catch (e) {
  if (e.status !== undefined) {
    process.exit(e.status);
  }
  console.error(e.message);
  process.exit(1);
}
