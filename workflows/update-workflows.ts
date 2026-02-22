// Build gaji workflows and commit generated YAML
import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const rustToolchain = getAction("dtolnay/rust-toolchain@stable");
const rustCache = getAction("Swatinem/rust-cache@v2");

const buildWorkflows = new Job("ubuntu-latest").steps((s) =>
  s
    .add(checkout({
      with: {
        token: "${{ secrets.PAT }}",
        "fetch-depth": 1,
      },
    }))
    .add(rustToolchain({}))
    .add(rustCache({}))
    .add({
      name: "Build gaji",
      run: "cargo build --release",
    })
    .add({
      name: "Generate Type",
      run: "./target/release/gaji dev",
    })
    .add({
      name: "Generate workflows",
      run: "./target/release/gaji build",
    })
    .add({
      name: "Commit and push",
      run: [
        'git config user.name "github-actions[bot]"',
        'git config user.email "github-actions[bot]@users.noreply.github.com"',
        "git add .github/workflows/",
        'git diff --cached --quiet || (git commit -m "chore: update generated workflows" && git push)',
      ].join("\n"),
    }),
);

new Workflow({
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
}).jobs((j) => j.add("build-workflows", buildWorkflows)).build("update-workflows");
