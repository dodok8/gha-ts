// release-plz: automated versioning, changelog, and crates.io publishing
import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const rustToolchain = getAction("dtolnay/rust-toolchain@stable");
const releasePlz = getAction("release-plz/action@v0.5");

// Job 1: Release (publish to crates.io + create git tag + GitHub Release)
const release = new Job("ubuntu-latest", {
  permissions: {
    contents: "write",
    "id-token": "write",
  },
})
  .addStep(
    checkout({
      with: {
        "fetch-depth": 0,
        "persist-credentials": false,
      },
    })
  )
  .addStep(rustToolchain({}))
  .addStep(
    releasePlz({
      with: {
        command: "release",
      },
      env: {
        GITHUB_TOKEN: "${{ secrets.GITHUB_TOKEN }}",
      },
    })
  );

// Job 2: Release PR (create/update release PR with version bump + changelog)
const releasePr = new Job("ubuntu-latest", {
  permissions: {
    contents: "write",
    "pull-requests": "write",
  },
})
  .addStep(
    checkout({
      with: {
        "fetch-depth": 0,
        "persist-credentials": false,
      },
    })
  )
  .addStep(rustToolchain({}))
  .addStep(
    releasePlz({
      with: {
        command: "release-pr",
      },
      env: {
        GITHUB_TOKEN: "${{ secrets.GITHUB_TOKEN }}",
      },
    })
  );

const workflow = new Workflow({
  name: "Release-plz",
  on: {
    push: {
      branches: ["main"],
    },
  },
  concurrency: {
    group: "release-plz-${{ github.ref }}",
    "cancel-in-progress": false,
  },
})
  .addJob("release-plz-release", release)
  .addJob("release-plz-pr", releasePr);

workflow.build("release-plz");
