// Migrated from YAML by gaji init --migrate
// NOTE: This is a basic conversion. Please review and adjust as needed.
import { getAction, Job, Workflow } from "../generated/index.js";

const rustCache = getAction("Swatinem/rust-cache@v2");
const checkout = getAction("actions/checkout@v5");
const rustToolchain = getAction("dtolnay/rust-toolchain@stable");

const fmt = new Job("ubuntu-latest")
  .addStep(checkout({}))
  .addStep(rustToolchain({
    with: {
      components: "rustfmt",
    },
  }))
  .addStep({
    run: "cargo fmt --all --check",
  });

const clippy = new Job("ubuntu-latest")
  .addStep(checkout({}))
  .addStep(rustToolchain({
    with: {
      components: "clippy",
    },
  }))
  .addStep(rustCache({}))
  .addStep({
    run: "cargo clippy --all-targets --all-features -- -D warnings",
  });

const test = new Job("ubuntu-latest")
  .addStep(checkout({
    with: {
      "fetch-depth": 1,
    },
  }))
  .addStep(rustToolchain({}))
  .addStep(rustCache({}))
  .addStep({
    run: "cargo test --all-features",
  });

const workflow = new Workflow({
  name: "PR",
  on: {
    pull_request: {
      branches: ["main"],
    },
  },
}).addJob("fmt", fmt).addJob("clippy", clippy).addJob("test", test);

workflow.build("pr");
