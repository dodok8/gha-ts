<div align="center">
  <img src="logo.jpg" alt="gaji logo" width="200"/>
  <h1>gaji</h1>
  <p>Type-safe GitHub Actions workflows in TypeScript</p>
  <p><em>GitHub Actions Justified Improvements</em></p>
  <p>🍆 Named after the Korean word "가지" (gaji, eggplant) - a versatile ingredient that makes everything better!</p>
</div>

## Overview

`gaji` is a CLI tool that allows developers to write GitHub Actions workflows in TypeScript with full type safety, then compile them to YAML. It automatically fetches `action.yml` definitions and generates typed wrappers, so you get autocomplete and type checking for every action input and output.

## Features

- TypeScript-based workflow authoring with full type safety
- Automatic type generation from `action.yml` files
- File watching for development (`--watch`)
- Single binary distribution (Rust)

## Installation

### From npm

```bash
npm install -D gaji
```

### From cargo

```bash
cargo install gaji
```

## Quick Start

```bash
# Initialize a new project (creates workflows/ and generated/ directories)
gaji init

# Add actions and generate types
gaji add actions/checkout@v5
gaji add actions/setup-node@v4

# Run a one-time dev scan to generate types
gaji dev

# Build workflows to YAML
gaji build
```

## Usage

### Writing Workflows

Create TypeScript files in the `workflows/` directory:

```typescript
import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

const build = new Job("ubuntu-latest")
  .addStep(checkout({}))
  .addStep(setupNode({
    with: { "node-version": "22" },
  }))
  .addStep({ name: "Install dependencies", run: "npm ci" })
  .addStep({ name: "Run tests", run: "npm test" });

const workflow = new Workflow({
  name: "CI",
  on: {
    push: { branches: ["main"] },
    pull_request: { branches: ["main"] },
  },
}).addJob("build", build);

workflow.build("ci");
```

Run `gaji build` and it outputs `.github/workflows/ci.yml`.

### Recommended Development Workflow

For the best experience, follow this workflow:

1. **Start watch mode**:
   ```bash
   gaji dev --watch
   ```
   Leave this running in a terminal. It will automatically generate types when you add new actions.

2. **Edit your TypeScript workflows** in `workflows/*.ts`:
   - Add or modify steps
   - Use `getAction()` with full type safety
   - Types are automatically generated for new actions

3. **Build to YAML**:
   ```bash
   gaji build
   ```

4. **Review the generated YAML** in `.github/workflows/`:
   - Verify commands are correct
   - Check that step order is as expected
   - Ensure all required fields are present

5. **Commit both TypeScript and YAML**:
   ```bash
   git add workflows/ .github/workflows/
   git commit -m "Update workflows"
   ```

#### Why Commit Both?

You should commit **both** the TypeScript source (`workflows/*.ts`) and the generated YAML (`.github/workflows/*.yml`):

- **TypeScript**: Source of truth for your workflows
- **YAML**: What GitHub Actions actually executes

#### ⚠️ Important: Avoid Auto-compilation in CI

While it's technically possible to create a GitHub Actions workflow that automatically compiles TypeScript to YAML on push, **this is NOT recommended** because:

1. **Race Condition**: The auto-compilation workflow might try to run while the YAML file is being updated, causing failures
2. **Complexity**: Adds unnecessary complexity to your CI/CD pipeline
3. **Debugging**: Harder to debug workflow issues when the YAML is constantly being regenerated

**Best Practice**: Always compile and review workflows locally before committing.

> **Note**: This repository includes an example auto-compile workflow for demonstration purposes only. It's not recommended for production use.

### Commands

- `gaji init` - Initialize a new project
- `gaji dev` - One-time scan and type generation
- `gaji dev --watch` - Keep watching for changes after the initial scan
- `gaji build` - Build TypeScript workflows to YAML
- `gaji add <action>` - Add a new action and generate types
- `gaji clean` - Clean generated files

## Documentation

📚 **[Full Documentation](docs/)** (English & 한국어)

- [Getting Started](docs/guide/getting-started.md)
- [Writing Workflows](docs/guide/writing-workflows.md)
- [CLI Reference](docs/reference/cli.md)
- [API Reference](docs/reference/api.md)
- [Examples](examples/)

## Examples

Check out the [examples/](examples/) directory for complete working examples:

- **[ts-package](examples/ts-package/)** - TypeScript package with gaji CI workflow using pnpm

## License

MIT License
