---
layout: home

hero:
  name: gaji
  text: Type-safe GitHub Actions
  tagline: Write GitHub Actions workflows in TypeScript with full type safety
  image:
    src: /logo.jpg
    alt: gaji
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: View on GitHub
      link: https://github.com/dodok8/gaji

features:
  - icon: 🔒
    title: Full Type Safety
    details: Write workflows in TypeScript with autocomplete and type checking for every action input and output.

  - icon: ⚡
    title: Fast Development
    details: File watching with automatic type generation. Change your workflow and see results immediately.

  - icon: 🎯
    title: Zero Configuration
    details: Automatic action.yml fetching and type generation. Just use getAction() and you're ready to go.

  - icon: 🦀
    title: Single Binary
    details: Built with Rust for maximum performance. Bundles QuickJS internally - no Node.js or external JS runtime required.

  - icon: 📄
    title: Standalone TypeScript
    details: Generated workflow files are completely standalone. Run with any TS runtime (tsx, ts-node, Deno) to output workflow JSON.

  - icon: 📦
    title: Works Everywhere
    details: Use with any language or build tool. Supports existing projects without restructuring your repository.

  - icon: 🔄
    title: Easy Migration
    details: Migrate existing YAML workflows to TypeScript with a single command.
---

## What is gaji?

**gaji** stands for **G**itHub **A**ctions **J**ustified **I**mprovements.

The name also comes from the Korean word "가지" (gaji), meaning eggplant 🍆 - a versatile ingredient that makes everything better, just like this tool makes GitHub Actions workflows better!

## Why gaji?

Writing GitHub Actions workflows in YAML is error-prone and lacks type safety. gaji lets you write workflows in TypeScript, giving you:

### Before (YAML)

```yaml
name: CI
on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-node@v4
        with:
          node-versoin: '20'  # Typo in key! No error until runtime ❌
          cache: 'npm'

      - run: npm ci
      - run: npm test
```

### After (TypeScript with gaji)

```typescript
import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v4");
const setupNode = getAction("actions/setup-node@v4");

const build = new Job("ubuntu-latest")
  .addStep(checkout({}))
  .addStep(setupNode({
    with: {
      "node-version": "20",  // ✅ Correct key name caught at compile time
      cache: "npm",          // ✅ Full autocomplete and type checking
    },
  }))
  .addStep({ run: "npm ci" })
  .addStep({ run: "npm test" });

const workflow = new Workflow({
  name: "CI",
  on: { push: { branches: ["main"] } },
}).addJob("build", build);

workflow.build("ci");
```

## Quick Start

### Installation

```bash
# Install via npm
npm install -D gaji

# Or install via cargo
cargo install gaji
```

::: tip No JS Runtime Required
gaji bundles QuickJS internally, so you don't need Node.js or any external JavaScript runtime. Works with any language or build tool!
:::

### Setup

```bash
# Initialize
gaji init

# Add actions
gaji add actions/checkout@v4
gaji add actions/setup-node@v4

# Generate types
gaji dev

# Build workflows
gaji build
```

Generated YAML appears in `.github/workflows/` and is ready to use!

::: tip Standalone TypeScript Files
Your workflow TypeScript files are completely standalone and self-contained. You can run them directly with any TypeScript runtime to see the workflow JSON:

```bash
# Using tsx
npx tsx workflows/ci.ts

# Using ts-node
npx ts-node workflows/ci.ts

# Using Deno
deno run workflows/ci.ts
```

This makes it easy to debug, inspect, or integrate workflows into other tools!
:::

## Recommended Workflow

1. **Start watch mode**: `gaji dev --watch`
2. **Edit TypeScript workflows** in `workflows/*.ts`
3. **Build to YAML**: `gaji build`
4. **Review generated YAML** in `.github/workflows/`
5. **Commit both TypeScript and YAML**

::: warning Important
While you can create a workflow that auto-compiles TypeScript to YAML on push, **this is NOT recommended** due to race condition issues. Always compile and review locally before committing.
:::

## Learn More

- [Getting Started Guide](/guide/getting-started)
- [CLI Reference](/reference/cli)
- [Examples](/examples/simple-ci)
