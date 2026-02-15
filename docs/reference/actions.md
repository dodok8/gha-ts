# Actions Reference

How to use GitHub Actions with gaji.

## Adding Actions

To use an action in your workflow, fetch its `action.yml` and generate TypeScript types.

```bash
gaji add actions/checkout@v5
```

You can also run `gaji dev --watch` and skip straight to the next step.

## Using Actions

Import and use actions with `getAction()`:

```ts twoslash
// @filename: workflows/example.ts
// ---cut---
import { getAction, Job } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

// Use in workflow
const job = new Job("ubuntu-latest")
  .addStep(checkout({}))
  .addStep(setupNode({
    with: {
      "node-version": "20",  // ✅ Type-safe!
    },
  }));
```

## Action Reference Format

Actions are referenced using the standard GitHub format:

```
owner/repo@version
```

Examples:
- `actions/checkout@v5`
- `actions/setup-node@v4`
- `docker/setup-buildx-action@v3`
- `softprops/action-gh-release@v1`

### Versions

You can use:
- **Tags**: `@v4`, `@v1.2.3`
- **Branches**: `@main`, `@develop`
- **Commits**: `@a1b2c3d`

## Type Safety

gaji generates types from the action's `action.yml`, giving you:

### Autocomplete

Your editor shows all available inputs:

```typescript
setupNode({
  with: {
  },
})
```

### Type Checking

Invalid inputs are caught immediately:

```typescript
// ❌ Type error - "cache" expects "npm" | "yarn" | "pnpm"
setupNode({
  with: {
    cache: "npn",  // Typo!
  },
})

// ✅ Correct
setupNode({
  with: {
    cache: "npm",
  },
})
```

### Documentation

Hover over inputs to see descriptions and default values.

```typescript
setupNode({
  with: {
    "node-version": "20",  // 📝 Description appears on hover
  },
})
```

## Common Actions

### actions/checkout

Checkout your repository:

```bash
gaji add actions/checkout@v5
```

```typescript
const checkout = getAction("actions/checkout@v5");

// Basic usage
.addStep(checkout({}))

// With options
.addStep(checkout({
  with: {
    repository: "owner/repo",
    ref: "main",
    token: "${{ secrets.GITHUB_TOKEN }}",
    "fetch-depth": 0,
  },
}))
```

### actions/setup-node

Setup Node.js:

```bash
gaji add actions/setup-node@v4
```

```typescript
const setupNode = getAction("actions/setup-node@v4");

.addStep(setupNode({
  with: {
    "node-version": "20",
    cache: "npm",
  },
}))
```

### actions/cache

Cache dependencies:

```bash
gaji add actions/cache@v4
```

```typescript
const cache = getAction("actions/cache@v4");

.addStep(cache({
  with: {
    path: "node_modules",
    key: "${{ runner.os }}-node-${{ hashFiles('**/package-lock.json') }}",
    "restore-keys": "${{ runner.os }}-node-",
  },
}))
```

### actions/upload-artifact

Upload build artifacts:

```bash
gaji add actions/upload-artifact@v4
```

```typescript
const uploadArtifact = getAction("actions/upload-artifact@v4");

.addStep(uploadArtifact({
  with: {
    name: "build-output",
    path: "dist/",
  },
}))
```

### actions/download-artifact

Download artifacts:

```bash
gaji add actions/download-artifact@v4
```

```typescript
const downloadArtifact = getAction("actions/download-artifact@v4");

.addStep(downloadArtifact({
  with: {
    name: "build-output",
    path: "dist/",
  },
}))
```

## Third-Party Actions

gaji works with any GitHub Action:

```bash
# Docker
gaji add docker/setup-buildx-action@v3
gaji add docker/build-push-action@v5

# Rust
gaji add dtolnay/rust-toolchain@stable

# GitHub
gaji add softprops/action-gh-release@v1
```

Example:

```typescript
const setupBuildx = getAction("docker/setup-buildx-action@v3");
const buildPush = getAction("docker/build-push-action@v5");

const job = new Job("ubuntu-latest")
  .addStep(checkout({}))
  .addStep(setupBuildx({}))
  .addStep(buildPush({
    with: {
      context: ".",
      push: true,
      tags: "user/app:latest",
    },
  }));
```

## Local Actions

Reference local composite actions:

```typescript
const myAction = getAction("./my-action");

.addStep(myAction({
  with: {
    input: "value",
  },
}))
```

Make sure to create the action first. See [CompositeAction](./api.md#compositeaction).

## Action Outputs

Use action outputs in subsequent steps.

```typescript
const setupNode = getAction("actions/setup-node@v4");

.addStep(setupNode({
  id: "setup-node",
  with: {
    "node-version": "20",
  },
}))
.addStep({
  run: "echo Node path: ${{ steps.setup-node.outputs.node-path }}",
})
```

## Updating Actions

To update action types, clear the cache and regenerate.

```bash
# Clear cache and regenerate
gaji clean --cache
gaji dev
```

## Troubleshooting

### "Action not found"

Make sure you've added the action.

```bash
gaji add actions/checkout@v5
gaji dev
```

### "Types not updated"

Clear cache and regenerate.

```bash
gaji clean
gaji dev
```

### "Rate limit exceeded"

Set a GitHub token.

```bash
export GITHUB_TOKEN=ghp_your_token_here
gaji dev
```

## Next Steps

- See [Examples](/examples/simple-ci)
- Check the [CLI Reference](./cli.md)
