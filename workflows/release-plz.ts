// release-plz: automated versioning, changelog, git tag, and GitHub Release
import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const rustToolchain = getAction("dtolnay/rust-toolchain@stable");
const releasePlz = getAction("release-plz/action@v0.5");

// Job 1: Release (create git tag + GitHub Release, no crates.io publish)
const release = new Job("ubuntu-latest", {
  permissions: {
    contents: "write",
  },
}).steps((s) =>
  s
    .add(
      checkout({
        with: {
          "fetch-depth": 0,
          "persist-credentials": false,
        },
      }),
    )
    .add(rustToolchain({}))
    .add(
      releasePlz({
        with: {
          command: "release",
        },
        env: {
          GITHUB_TOKEN: "${{ secrets.PAT }}",
        },
      }),
    ),
);

// Job 2: Release PR (create/update release PR with version bump + changelog)
const releasePr = new Job("ubuntu-latest", {
  permissions: {
    contents: "write",
    "pull-requests": "write",
  },
}).steps((s) =>
  s
    .add(
      checkout({
        with: {
          "fetch-depth": 0,
          "persist-credentials": false,
        },
      }),
    )
    .add(rustToolchain({}))
    .add(
      releasePlz({
        with: {
          command: "release-pr",
        },
        env: {
          GITHUB_TOKEN: "${{ secrets.PAT }}",
        },
      }),
    ),
);

new Workflow({
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
}).jobs((j) =>
  j
    .add("release-plz-release", release)
    .add("release-plz-pr", releasePr),
).build("release-plz");
