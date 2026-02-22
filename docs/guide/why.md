# Why gaji?

**gaji** stands for **G**itHub **A**ctions **J**ustified **I**mprovements.

The name also comes from the Korean word "가지" (gaji), meaning eggplant 🍆 - a versatile vegetable.

## Problems with GitHub Actions

There are three main drawbacks when working with GitHub Actions:

1. YAML is a language for representing data, not for expressing operations with inputs, outputs, and side effects.
2. YAML has no type checking. GitHub Actions frequently depend on external repositories (even `actions/checkout@v5` is external), yet there is no validation of the inputs they require. Users must manually read the documentation and enter everything in the correct format.
3. It is difficult to reproduce locally.

These drawbacks combine to make GitHub Actions a platform where you can't catch even a simple typo until you actually run it. gaji addresses the first and second problems by automatically fetching action.yml from external repositories and generating types.

### Errors in YAML

```yaml
name: CI
on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5

      - uses: actions/setup-node@v4
        with:
          node-versoin: '20'  # Typo in key! No error until runtime ❌
          cache: 'npm'

      - run: npm ci
      - run: npm test
```

### With gaji...

```ts twoslash
// @filename: workflows/example.ts
// ---cut---
import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

const build = new Job("ubuntu-latest")
  .steps(s => s
    .add(checkout({}))
    .add(setupNode({
      with: {
        "node-version": "20",  // ✅ Correct key name, caught at compile time, full autocomplete and type checking
        cache: "npm",
      },
    }))
    .add({ run: "npm ci" })
    .add({ run: "npm test" })
  );

const workflow = new Workflow({
  name: "CI",
  on: { push: { branches: ["main"] } },
}).jobs(j => j
    .add("build", build)
  );

workflow.build("ci");
```

## Special Thanks

### gaji Brand

- Name suggestions: [kiwiyou](https://github.com/kiwiyou), [RanolP](https://github.com/ranolp)
- Logo design: [sij411](https://github.com/sij411)

### Inspiration

- Client Devops Team@Toss: Without the experience on this team, I would never have thought deeply about YAML and GitHub Actions. The product below was also introduced to me through a teammate.
- [emmanuelnk/github-actions-workflow-ts](https://github.com/emmanuelnk/github-actions-workflow-ts): The idea of writing GitHub Actions in TypeScript came from here.
