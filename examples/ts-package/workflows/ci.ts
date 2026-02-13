import { getAction, Job, Workflow } from "../../../generated/index.js";

// Get actions with full type safety
const checkout = getAction("actions/checkout@v4");
const setupNode = getAction("actions/setup-node@v4");

// Define the CI job
const build = new Job("ubuntu-latest")
  .addStep(
    checkout({
      name: "Checkout code",
    })
  )
  .addStep(
    setupNode({
      name: "Setup Node.js",
      with: {
        "node-version": "20",
      },
    })
  )
  .addStep({
    name: "Install pnpm",
    run: "npm install -g pnpm",
  })
  .addStep({
    name: "Install dependencies",
    run: "pnpm install",
  })
  .addStep({
    name: "Run lint",
    run: "pnpm lint",
  })
  .addStep({
    name: "Run tests",
    run: "pnpm test",
  })
  .addStep({
    name: "Build package",
    run: "pnpm build",
  });

// Create the workflow
const workflow = new Workflow({
  name: "CI",
  on: {
    push: {
      branches: ["main"],
    },
    pull_request: {
      branches: ["main"],
    },
  },
}).addJob("build", build);

// Build the YAML file
workflow.build("ci");
