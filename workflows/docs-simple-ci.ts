// Docs example: Simple CI
// From docs/examples/simple-ci.md
import { getAction, Job, Workflow } from "../generated/index.js";

// Add actions
const checkout = getAction("actions/checkout@v4");
const setupNode = getAction("actions/setup-node@v4");

// Define the test job
const test = new Job("ubuntu-latest")
  .addStep(checkout({
    name: "Checkout code",
  }))
  .addStep(setupNode({
    name: "Setup Node.js",
    with: {
      "node-version": "20",
      cache: "npm",
    },
  }))
  .addStep({
    name: "Install dependencies",
    run: "npm ci",
  })
  .addStep({
    name: "Run linter",
    run: "npm run lint",
  })
  .addStep({
    name: "Run tests",
    run: "npm test",
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
}).addJob("test", test);

// Build to YAML
workflow.build("ci");
