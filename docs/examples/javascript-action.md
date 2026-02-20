# Example: JavaScript Action

Create Node.js-based GitHub Actions and use them in workflows.

## Defining a JavaScript Action

Create `workflows/hello.ts`:

```ts twoslash
// @filename: workflows/example.ts
// ---cut---
import { NodeAction } from "../generated/index.js";

const action = new NodeAction(
  {
    name: "Hello World",
    description: "Greet someone and record the time",
    inputs: {
      "who-to-greet": {
        description: "Who to greet",
        required: true,
        default: "World",
      },
    },
    outputs: {
      time: {
        description: "The time we greeted you",
      },
    },
  },
  {
    using: "node20",
    main: "dist/index.js",
  },
);

action.build("hello-world");
```

Build it:

```bash
gaji build
```

This generates `.github/actions/hello-world/action.yml`.

## Using the Action in a Workflow

You can define the action and a workflow that uses it in the same file:

```ts twoslash
// @filename: workflows/example.ts
// ---cut---
import { ActionRef, NodeAction, Job, Workflow } from "../generated/index.js";

// Define the action
const action = new NodeAction(
  {
    name: "Hello World",
    description: "Greet someone and record the time",
    inputs: {
      "who-to-greet": {
        description: "Who to greet",
        required: true,
        default: "World",
      },
    },
    outputs: {
      time: {
        description: "The time we greeted you",
      },
    },
  },
  {
    using: "node20",
    main: "dist/index.js",
  },
);

action.build("hello-world");

// Use the action in a workflow
const helloWorldJob = new Job("ubuntu-latest")
  .addStep({
    name: "Hello world action step",
    id: "hello",
    ...ActionRef.from(action).toJSON(),
    with: {
      "who-to-greet": "Mona the Octocat",
    },
  })
  .addStep({
    name: "Get the output time",
    run: 'echo "The time was ${{ steps.hello.outputs.time }}"',
  });

const workflow = new Workflow({
  name: "Use JavaScript Action",
  on: {
    push: { branches: ["main"] },
  },
}).addJob("hello_world_job", helloWorldJob);

workflow.build("use-js-action");
```

This generates both:
- `.github/actions/hello-world/action.yml` - The action definition
- `.github/workflows/use-js-action.yml` - The workflow that uses it

## Pre/Post Scripts

JavaScript actions support lifecycle hooks:

```typescript
const action = new NodeAction(
  {
    name: "Setup and Cleanup",
    description: "Action with pre and post scripts",
  },
  {
    using: "node20",
    main: "dist/index.js",
    pre: "dist/setup.js",
    post: "dist/cleanup.js",
    "post-if": "always()",
  },
);

action.build("setup-cleanup");
```

## Referencing with `ActionRef`

Use `ActionRef.from()` to reference a locally defined action in a workflow step:

```typescript
const step = {
  name: "Run my action",
  id: "my-step",
  ...ActionRef.from(action).toJSON(),
  with: { input1: "value1" },
};
```

This resolves to `uses: ./.github/actions/<action-id>`.

## Next Steps

- See [API Reference](/reference/api)
- Learn about [Composite Actions](./composite-action.md)
