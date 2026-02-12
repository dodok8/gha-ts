# gaji

Type-safe GitHub Actions workflows in TypeScript.

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

### Commands

- `gaji init` - Initialize a new project
- `gaji dev` - One-time scan and type generation
- `gaji dev --watch` - Keep watching for changes after the initial scan
- `gaji build` - Build TypeScript workflows to YAML
- `gaji add <action>` - Add a new action and generate types
- `gaji clean` - Clean generated files

## License

MIT License
