// Build gaji workflows and commit generated YAML
import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v4");
const rustToolchain = getAction("dtolnay/rust-toolchain@stable");
const rustCache = getAction("Swatinem/rust-cache@v2");

const buildWorkflows = new Job("ubuntu-latest")
  .addStep(checkout({
    with: {
      token: "${{ secrets.GITHUB_TOKEN }}",
    },
  }))
  .addStep(rustToolchain({}))
  .addStep(rustCache({}))
  .addStep({
    name: "Build gaji",
    run: "cargo build --release",
  })
  .addStep({
    name: "Generate workflows",
    run: "./target/release/gaji build",
  })
  .addStep({
    name: "Check for changes",
    id: "changes",
    run: [
      'if git diff --quiet .github/workflows/; then',
      '  echo "changed=false" >> $GITHUB_OUTPUT',
      'else',
      '  echo "changed=true" >> $GITHUB_OUTPUT',
      'fi',
    ].join("\n"),
  })
  .addStep({
    name: "Commit and push",
    "if": "steps.changes.outputs.changed == 'true'",
    run: [
      'git config user.name "github-actions[bot]"',
      'git config user.email "github-actions[bot]@users.noreply.github.com"',
      "git add .github/workflows/",
      'git commit -m "chore: update generated workflows"',
      "git push",
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
}).addJob("build-workflows", buildWorkflows);

workflow.build("update-workflows");
