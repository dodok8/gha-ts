// Build gaji workflows and commit generated YAML
import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const rustToolchain = getAction("dtolnay/rust-toolchain@stable");
const rustCache = getAction("Swatinem/rust-cache@v2");

const buildWorkflows = new Job("ubuntu-latest")
  .addStep(checkout({
    with: {
      token: "${{ secrets.PAT }}",
      "fetch-depth": 1,
    },
  }))
  .addStep(rustToolchain({}))
  .addStep(rustCache({}))
  .addStep({
    name: "Build gaji",
    run: "cargo build --release",
  })
  .addStep({
    name: "Generate Type",
    run: "./target/release/gaji dev",
  })
  .addStep({
    name: "Generate workflows",
    run: "./target/release/gaji build",
  })
  .addStep({
    name: "Commit and push",
    run: [
      'git config user.name "github-actions[bot]"',
      'git config user.email "github-actions[bot]@users.noreply.github.com"',
      "git add .github/workflows/",
      'git diff --cached --quiet || (git commit -m "chore: update generated workflows" && git push)',
    ].join("\n"),
  });

const workflow = new Workflow({
  name: "Update Workflows",
  on: {
    push: {
      branches: ["main"],
      paths: ["workflows/**"],
    },
  },
  permissions: {
    contents: "write",
  },
}).addJob("build-workflows", buildWorkflows);

workflow.build("update-workflows");
