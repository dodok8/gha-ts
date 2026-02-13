// Docs example: Matrix Build
// From docs/examples/matrix-build.md
import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v4");
const setupNode = getAction("actions/setup-node@v4");

// Define matrix test job
const test = new Job("${{ matrix.os }}")
  .strategy({
    matrix: {
      os: ["ubuntu-latest", "macos-latest", "windows-latest"],
      node: ["18", "20", "22"],
    },
  })
  .addStep(checkout({
    name: "Checkout code",
  }))
  .addStep(setupNode({
    name: "Setup Node.js ${{ matrix.node }}",
    with: {
      "node-version": "${{ matrix.node }}",
      cache: "npm",
    },
  }))
  .addStep({
    name: "Install dependencies",
    run: "npm ci",
  })
  .addStep({
    name: "Run tests",
    run: "npm test",
  });

// Create workflow
const workflow = new Workflow({
  name: "Matrix Test",
  on: {
    push: {
      branches: ["main"],
    },
    pull_request: {
      branches: ["main"],
    },
  },
}).addJob("test", test);

workflow.build("matrix-test");
