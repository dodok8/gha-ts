<div align="center">
  <img src="logo.png" alt="gaji logo" width="200"/>
  <h1>gaji</h1>
  <p>Type-safe GitHub Actions workflows in TypeScript</p>
  <p><em>GitHub Actions Justified Improvements</em></p>
  <p>🍆 Named after the Korean word "가지" (gaji, eggplant) - a versatile vegetable</p>
  <p>
    <a href="https://crates.io/crates/gaji"><img src="https://img.shields.io/crates/v/gaji" alt="crates.io"></a>
    <a href="https://www.npmjs.com/package/gaji"><img src="https://img.shields.io/npm/v/gaji" alt="npm"></a>
  </p>
</div>

## Overview

`gaji` is a CLI tool that allows developers to write GitHub Actions workflows in TypeScript with full type safety, then compile them to YAML. It automatically fetches `action.yml` definitions and generates typed wrappers, providing autocomplete and type checking for action inputs and outputs.

## Features

- TypeScript-based workflow authoring with full type safety
- Automatic type generation from `action.yml` files
- Composite action and reusable workflow support
- File watching for development (`--watch`)
- Built-in QuickJS execution with `npx tsx` fallback
- GitHub Enterprise support
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

new Workflow({
  name: "CI",
  on: {
    push: { branches: ["main"] },
    pull_request: { branches: ["main"] },
  },
}).jobs(j => j
  .add("build",
    new Job("ubuntu-latest")
      .steps(s => s
        .add(checkout({ name: "Checkout code", with: { "fetch-depth": 1 } }))
        .add(setupNode({ with: { "node-version": "22" } }))
        .add({ name: "Install dependencies", run: "npm ci" })
        .add({ name: "Run tests", run: "npm test" })
      )
  )
).build("ci");
```

Run `gaji build` and it outputs `.github/workflows/ci.yml`.

### Recommended Development Workflow

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

#### ⚠️ Important: Auto-compilation in CI

While you can create a workflow that auto-compiles TypeScript to YAML on push, **this is NOT recommended**. Always compile and review workflows locally before committing.

If you're willing to handle the complexity of GitHub Actions triggers (e.g., filtering `paths`, managing PAT tokens, avoiding infinite loops), you can set up an auto-compilation workflow. See [`workflows/update-workflows.ts`](https://github.com/dodok8/gaji/blob/main/workflows/update-workflows.ts) for a working example.

### Composite Actions

Define reusable composite actions and reference them in workflows:

```typescript
import { Action, ActionRef, Job, Workflow } from "../generated/index.js";

const action = new Action({
  name: "Setup",
  description: "Setup the project environment",
  inputs: {
    "node-version": { description: "Node.js version", required: false, default: "20" },
  },
})
  .steps(s => s
    .add({ name: "Install deps", run: "npm ci", shell: "bash" })
  );
action.build("setup");

// Reference the composite action in a workflow
new Workflow({
  name: "CI",
  on: { push: {} },
}).jobs(j => j
  .add("build",
    new Job("ubuntu-latest")
      .steps(s => s
        .add(ActionRef.from(action).toJSON())
      )
  )
).build("ci");
```

### Reusable Workflows

Call reusable workflows using `WorkflowCall`:

```typescript
import { WorkflowCall, Workflow } from "../generated/index.js";

new Workflow({
  name: "Release",
  on: { push: { tags: ["v*"] } },
}).jobs(j => j
  .add("deploy",
    new WorkflowCall("./.github/workflows/deploy.yml", {
      with: { environment: "production" },
      secrets: "inherit",
      needs: ["build"],
    })
  )
).build("release");
```

### Job Options

The `Job` constructor accepts an optional second argument for configuration:

```typescript
const job = new Job("ubuntu-latest", {
  needs: ["setup"],
  env: { NODE_ENV: "test" },
  "timeout-minutes": 30,
  "continue-on-error": true,
  permissions: { contents: "read" },
  strategy: {
    matrix: { node: ["18", "20", "22"] },
    "fail-fast": false,
  },
})
  .steps(s => s
    .add({ name: "Test", run: "npm test" })
  );
```

Steps are added via the `steps()` callback, and job-level settings are passed through the constructor. This keeps configuration separate from step definitions.

## Commands

### `gaji init`

Initialize a new gaji project. Detects the project state (empty, existing project, or has YAML workflows) and sets up accordingly.

```
gaji init [OPTIONS]
```

| Option | Description |
|---|---|
| `--force` | Overwrite existing files |
| `--skip-examples` | Skip example workflow creation |
| `--migrate` | Migrate existing YAML workflows to TypeScript |
| `-i, --interactive` | Interactive mode |

### `gaji dev`

Start development mode. Scans workflow files for action references and generates types.

```
gaji dev [OPTIONS]
```

| Option | Description |
|---|---|
| `-i, --input <PATH>...` | Workflow directories or individual `.ts` files (falls back to `workflows_dir` in config) |
| `--watch` | Keep watching for changes after the initial scan |

### `gaji build`

Build TypeScript workflows to YAML.

```
gaji build [OPTIONS]
```

| Option | Description |
|---|---|
| `-i, --input <PATH>...` | Workflow directories or individual `.ts` files (falls back to `workflows_dir` in config) |
| `-o, --output <DIR>` | Output directory for YAML files (falls back to `output_dir` in config) |
| `--dry-run` | Preview YAML output without writing files |

Output files are placed in subdirectories based on type:
- Workflows: `.github/workflows/<id>.yml`
- Composite actions: `.github/actions/<id>/action.yml`

### `gaji add <action>`

Add a new action and generate types.

```bash
gaji add actions/checkout@v5
gaji add actions/setup-node@v4
```

### `gaji clean`

Clean generated files.

```
gaji clean [OPTIONS]
```

| Option | Description |
|---|---|
| `--cache` | Also clean cache |

## Configuration

### `gaji.config.ts`

Project-level configuration file. Created automatically by `gaji init`.

```typescript
import { defineConfig } from "./generated/index.js";

export default defineConfig({
    workflows: "workflows",    // TypeScript workflow source directory
    output: ".github",         // Output directory for generated YAML
    generated: "generated",    // Directory for generated type definitions
    watch: {
        debounce: 300,         // Debounce delay for file watcher (ms)
        ignore: ["node_modules", ".git", "generated"],
    },
    build: {
        validate: true,        // Validate workflow YAML (requires 'on' and 'jobs')
        format: true,          // Format YAML output
    },
});
```

### `gaji.config.local.ts`

Local overrides for sensitive values. Add this to `.gitignore`.

```typescript
import { defineConfig } from "./generated/index.js";

export default defineConfig({
    github: {
        token: "ghp_your_token_here",
        apiUrl: "https://github.example.com",  // for GitHub Enterprise
    },
});
```

Token resolution priority: `GITHUB_TOKEN` env var > `gaji.config.local.ts` > `gaji.config.ts`

gaji also reads `.gaji.toml` / `.gaji.local.toml` as a fallback for existing projects.

## Documentation

📚 **[Full Documentation](https://gaji.gaebalgom.work)** (English & 한국어)

- [Getting Started](https://gaji.gaebalgom.work/guide/getting-started)
- [Writing Workflows](https://gaji.gaebalgom.work/guide/writing-workflows)
- [CLI Reference](https://gaji.gaebalgom.work/reference/cli)
- [API Reference](https://gaji.gaebalgom.work/reference/api)
- [Examples](examples/)

## Examples

Check out the [examples/](examples/) directory for complete working examples:

- **[ts-package](examples/ts-package/)** - TypeScript package with gaji CI workflow using pnpm

## License

MIT License

## Special Thanks

### gaji Brand

- Name suggestions: [kiwiyou](https://github.com/kiwiyou), [RanolP](https://github.com/ranolp)
- Logo design: [sij411](https://github.com/sij411)

### Inspiration

- Client Devops Team@Toss: Without the experience on this team, I would never have thought deeply about YAML and GitHub Actions. The product below was also introduced to me through a teammate.
- [emmanuelnk/github-actions-workflow-ts](https://github.com/emmanuelnk/github-actions-workflow-ts): The idea of writing GitHub Actions in TypeScript came from here.