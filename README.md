# gha-ts

Type-safe GitHub Actions workflows in TypeScript.

## Overview

`gha-ts` is a CLI tool that allows developers to write GitHub Actions workflows in TypeScript with full type safety, then compile them to YAML.

## Features

- TypeScript-based workflow authoring
- Automatic type generation from action.yml files
- File watching for development
- Single binary distribution (Rust)

## Installation

### From cargo

```bash
cargo install gha-ts
```

### From npm

```bash
npm install -g gha-ts
```

## Quick Start

```bash
# Initialize a new project
gha-ts init

# Start development mode (watch for changes)
gha-ts dev

# Build workflows to YAML
gha-ts build
```

## Usage

### Writing Workflows

Create TypeScript files in the `workflows/` directory:

```typescript
import { Workflow, Job } from 'gha-ts'
import { getAction } from 'gha-ts/actions'

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

- `gha-ts init` - Initialize a new project
- `gha-ts dev` - Start development mode with file watching
- `gha-ts build` - Build TypeScript workflows to YAML
- `gha-ts watch` - Watch for file changes
- `gha-ts add <action>` - Add a new action and generate types
- `gha-ts clean` - Clean generated files

## License

MIT License
