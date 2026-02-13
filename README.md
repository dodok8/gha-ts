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

const build = new Job("ubuntu-latest")
  .addStep(checkout({
    name: "Checkout code",
    with: { "fetch-depth": 1 },
  }))
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

### Composite Actions

Define reusable composite actions and reference them in workflows:

```typescript
import { CompositeAction, CallAction, Job, Workflow } from "../generated/index.js";

const action = new CompositeAction({
  name: "Setup",
  description: "Setup the project environment",
  inputs: {
    "node-version": { description: "Node.js version", required: false, default: "20" },
  },
});
action.addStep({ name: "Install deps", run: "npm ci", shell: "bash" });
action.build("setup");

// Reference the composite action in a workflow
const job = new Job("ubuntu-latest")
  .addStep(CallAction.from(action).toJSON());

const workflow = new Workflow({
  name: "CI",
  on: { push: {} },
}).addJob("build", job);

workflow.build("ci");
```

### Reusable Workflows

Call reusable workflows using `CallJob`:

```typescript
import { CallJob, Workflow } from "../generated/index.js";

const deploy = new CallJob("./.github/workflows/deploy.yml")
  .with({ environment: "production" })
  .secrets("inherit")
  .needs(["build"]);

const workflow = new Workflow({
  name: "Release",
  on: { push: { tags: ["v*"] } },
}).addJob("deploy", deploy);

workflow.build("release");
```

### Job Options

The `Job` constructor accepts an optional second argument for additional configuration:

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
});
```

Builder methods are also available:

```typescript
const job = new Job("ubuntu-latest")
  .addStep({ name: "Test", run: "npm test" })
  .needs(["setup"])
  .env({ CI: "true" })
  .when("github.event_name == 'push'")
  .permissions({ contents: "read" })
  .outputs({ result: "${{ steps.test.outputs.result }}" })
  .strategy({ matrix: { os: ["ubuntu-latest", "macos-latest"] } })
  .continueOnError(true)
  .timeoutMinutes(30);
```

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
| `-d, --dir <DIR>` | Directory to scan (default: `workflows`) |
| `--watch` | Keep watching for changes after the initial scan |

### `gaji build`

Build TypeScript workflows to YAML.

```
gaji build [OPTIONS]
```

| Option | Description |
|---|---|
| `-i, --input <DIR>` | Input directory containing TypeScript workflows (default: `workflows`) |
| `-o, --output <DIR>` | Output directory for YAML files (default: `.github`) |
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

### `.gaji.toml`

Project-level configuration file. Created automatically by `gaji init`.

```toml
[project]
workflows_dir = "workflows"    # TypeScript workflow source directory
output_dir = ".github"         # Output directory for generated YAML
generated_dir = "generated"    # Directory for generated type definitions

[watch]
debounce_ms = 300              # Debounce delay for file watcher
ignored_patterns = ["node_modules", ".git", "generated"]

[build]
validate = true                # Validate workflow YAML (requires 'on' and 'jobs')
format = true                  # Format YAML output

[github]
token = "ghp_..."              # GitHub token (prefer .gaji.local.toml for this)
api_url = "https://github.example.com"  # GitHub Enterprise URL
```

### `.gaji.local.toml`

Local overrides for sensitive values. Add this to `.gitignore`.

```toml
[github]
token = "ghp_your_token_here"
api_url = "https://github.example.com"  # for GitHub Enterprise
```

Token resolution priority: `GITHUB_TOKEN` env var > `.gaji.local.toml` > `.gaji.toml`

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
