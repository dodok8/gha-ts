// Docs example: Composite Action
// From docs/examples/composite-action.md
import { CompositeAction, getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v4");
const setupNode = getAction("actions/setup-node@v4");

const setupEnv = new CompositeAction({
  name: "Setup Environment",
  description: "Setup Node.js and install dependencies",
  inputs: {
    "node-version": {
      description: "Node.js version to use",
      required: false,
      default: "20",
    },
    "cache-dependency-path": {
      description: "Path to dependency file",
      required: false,
      default: "package-lock.json",
    },
  },
  outputs: {
    "node-version": {
      description: "Installed Node.js version",
      value: "${{ steps.setup-node.outputs.node-version }}",
    },
  },
})
  .addStep(checkout({
    name: "Checkout code",
  }))
  .addStep(setupNode({
    id: "setup-node",
    name: "Setup Node.js",
    with: {
      "node-version": "${{ inputs.node-version }}",
      cache: "npm",
      "cache-dependency-path": "${{ inputs.cache-dependency-path }}",
    },
  }))
  .addStep({
    name: "Install dependencies",
    run: "npm ci",
    shell: "bash",
  });

setupEnv.build("setup-env");

// Use the composite action in a workflow
const test = new Job("ubuntu-latest")
  .addStep({
    name: "Setup environment",
    uses: "./.github/actions/setup-env",
    with: {
      "node-version": "20",
    },
  })
  .addStep({
    name: "Run tests",
    run: "npm test",
  });

const workflow = new Workflow({
  name: "CI with Composite Action",
  on: { push: { branches: ["main"] } },
}).addJob("test", test);

workflow.build("ci-composite");
