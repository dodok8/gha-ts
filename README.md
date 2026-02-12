# gaji

Type-safe GitHub Actions workflows in TypeScript.

## Overview

`gaji` is a CLI tool that allows developers to write GitHub Actions workflows in TypeScript with full type safety, then compile them to YAML.

## Features

- TypeScript-based workflow authoring
- Automatic type generation from action.yml files
- File watching for development
- Single binary distribution (Rust)

## Installation

### From cargo

```bash
cargo install gaji
```

### From npm

```bash
npm install -g gaji
```

## Quick Start

```bash
# Initialize a new project
gaji init

# Run a one-time dev scan
gaji dev

# Run dev mode and keep watching for changes
gaji dev --watch

# Build workflows to YAML
gaji build
```

## Usage

### Writing Workflows

Create TypeScript files in the `workflows/` directory:

```typescript
import { Workflow, Job } from 'gaji'
import { getAction } from 'gaji/actions'

const checkout = getAction('actions/checkout@v4')
const setupNode = getAction('actions/setup-node@v4')

export const ci = new Workflow('ci', {
  name: 'CI',
  on: {
    push: { branches: ['main'] },
    pull_request: { branches: ['main'] },
  },
})
  .addJob(
    new Job('build', 'ubuntu-latest')
      .addStep(checkout({ name: 'Checkout code' }))
      .addStep(setupNode({
        name: 'Setup Node.js',
        with: { 'node-version': '20' },
      }))
      .addStep({ name: 'Install dependencies', run: 'npm ci' })
      .addStep({ name: 'Run tests', run: 'npm test' })
  )
```

### Commands

- `gaji init` - Initialize a new project
- `gaji dev` - One-time scan and type generation
- `gaji dev --watch` - Keep watching for changes after the initial scan
- `gaji build` - Build TypeScript workflows to YAML
- `gaji add <action>` - Add a new action and generate types
- `gaji clean` - Clean generated files

## License

MIT License
