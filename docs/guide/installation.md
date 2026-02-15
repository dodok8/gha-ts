# Installation

gaji can be installed in multiple ways depending on your preference and environment.

::: tip No JS Runtime Required
gaji bundles QuickJS internally, so you don't need Node.js or any external JavaScript runtime installed. It works as a standalone binary!
:::

## npm

```bash
npm install -D gaji
```

This installs gaji as a dev dependency:

```bash
gaji --help
```

### Other Package Managers

```bash
# pnpm
pnpm add -D gaji

# yarn
yarn add -D gaji
```

## Cargo

If you have Rust installed, you can install gaji directly from crates.io:

```bash
cargo install gaji
```

This installs the gaji binary globally:

```bash
gaji --help
```

## Binary Download

Download pre-built binaries from [GitHub Releases](https://github.com/dodok8/gaji/releases).

### Linux (x64)

```bash
curl -L https://github.com/dodok8/gaji/releases/latest/download/gaji-linux-x64.tar.gz | tar xz
sudo mv gaji /usr/local/bin/
```

### macOS (ARM64)

```bash
curl -L https://github.com/dodok8/gaji/releases/latest/download/gaji-darwin-arm64.tar.gz | tar xz
sudo mv gaji /usr/local/bin/
```

### macOS (x64)

```bash
curl -L https://github.com/dodok8/gaji/releases/latest/download/gaji-darwin-x64.tar.gz | tar xz
sudo mv gaji /usr/local/bin/
```

### Windows

Download `gaji-win32-x64.tar.gz` from the releases page and extract it to a directory in your PATH.

## Verify Installation

```bash
gaji --version  # Prints the gaji version
```

## Requirements

gaji has **no runtime dependencies**:

- ✅ No Node.js required — gaji bundles QuickJS internally
- ✅ No external JavaScript runtime needed
- ✅ Works with any language or build tool

The only requirement is a package manager if installing via npm:

- **npm/pnpm/yarn** (for npm installation only)

## Updating

### npm

```bash
npm update gaji
```

### Cargo

```bash
cargo install gaji --force
```

### Binary

Download and replace the binary with the latest version from GitHub Releases.

## Next Steps

Once installed, proceed to [Getting Started](./getting-started.md) to set up your first project.
