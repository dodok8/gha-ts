# Example TypeScript Package with gaji

This is an example TypeScript package that demonstrates how to use `gaji` to create type-safe GitHub Actions workflows.

## Project Structure

```
ts-package/
├── src/
│   └── index.ts          # Example TypeScript source
├── workflows/
│   └── ci.ts             # Type-safe GitHub Actions workflow
├── generated/            # Auto-generated action types (by gaji)
├── .github/
│   └── workflows/
│       └── ci.yml        # Generated YAML workflow (by gaji build)
├── package.json
├── tsconfig.json
└── README.md
```

## Setup

1. Install dependencies:
   ```bash
   pnpm install
   ```

2. Initialize gaji (if not already done):
   ```bash
   pnpm gha:dev
   ```

## Workflow Development

### Recommended Workflow (Best Practice)

1. **Start watch mode**:
   ```bash
   pnpm gha:watch
   ```
   This will watch for changes in your workflow TypeScript files and automatically generate types for new actions.

2. **Edit `workflows/ci.ts`**:
   - Add or modify workflow steps
   - Use `getAction()` with full type safety
   - When you add a new action, gaji will automatically fetch its schema and generate types

3. **Build workflows**:
   ```bash
   pnpm gha:build
   ```
   This compiles your TypeScript workflows to YAML files in `.github/workflows/`.

4. **Review the generated YAML**:
   Check `.github/workflows/ci.yml` to ensure:
   - Commands are correct
   - Step order is as expected
   - All required fields are present

5. **Commit both TypeScript and YAML**:
   ```bash
   git add workflows/ .github/workflows/
   git commit -m "Update CI workflow"
   ```

### Why Commit Both?

You should commit **both** the TypeScript source (`workflows/*.ts`) and the generated YAML (`.github/workflows/*.yml`) because:

- **TypeScript**: Source of truth for your workflows
- **YAML**: What GitHub Actions actually executes

### ⚠️ Important: Avoid Auto-compilation in CI

While it's technically possible to create a GitHub Actions workflow that automatically compiles TypeScript workflows to YAML on push, **this is NOT recommended** because:

1. **Race Condition**: The auto-compilation workflow might try to run while the YAML file is being updated, causing failures.
2. **Complexity**: Adds unnecessary complexity to your CI/CD pipeline.
3. **Debugging**: Harder to debug workflow issues when the YAML is constantly being regenerated.

**Best Practice**: Always compile and review workflows locally before committing.

## Example Workflow

The included `workflows/ci.ts` demonstrates:

- Type-safe action usage with `getAction()`
- Node.js setup with specific version
- pnpm installation and usage
- Running linter, tests, and build steps

## Learn More

- [gaji Documentation](../../docs/)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
