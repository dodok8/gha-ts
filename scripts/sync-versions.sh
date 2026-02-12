#!/bin/bash
set -euo pipefail

# Read version from Cargo.toml
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')

# Override from git tag if GITHUB_REF is set
if [ -n "${GITHUB_REF:-}" ]; then
  TAG_VERSION="${GITHUB_REF#refs/tags/v}"
  if [ "$TAG_VERSION" != "$GITHUB_REF" ]; then
    VERSION="$TAG_VERSION"
  fi
fi

echo "Syncing version to: $VERSION"

# Update main package
cd npm/gaji
node -e "
const fs = require('fs');
const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'));
pkg.version = '$VERSION';
for (const dep of Object.keys(pkg.optionalDependencies || {})) {
  pkg.optionalDependencies[dep] = '$VERSION';
}
fs.writeFileSync('package.json', JSON.stringify(pkg, null, 2) + '\n');
"
echo "  Updated npm/gaji/package.json"
cd ../..

# Update platform packages
for dir in npm/platform-*/; do
  cd "$dir"
  node -e "
const fs = require('fs');
const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'));
pkg.version = '$VERSION';
fs.writeFileSync('package.json', JSON.stringify(pkg, null, 2) + '\n');
"
  echo "  Updated $dir/package.json"
  cd ../..
done

echo "All versions synced to $VERSION"
