import { getAction, Job, Workflow } from "../generated/index.js";

const rustCache = getAction("Swatinem/rust-cache@v2");
const checkout = getAction("actions/checkout@v5");
const rustToolchain = getAction("dtolnay/rust-toolchain@stable");

const fmt = new Job("ubuntu-latest")
  .steps((s) =>
    s
      .add(
        checkout({
          with: {
            "fetch-depth": 1,
          },
        }),
      )
      .add(rustToolchain({
        with: {
          components: "rustfmt",
        },
      }))
      .add(rustCache({}))
      .add(
        {
          run: "cargo fmt --all --check",
        },
      )
  );

const clippy = new Job("ubuntu-latest").steps(
  (s) =>
    s
      .add(
        checkout({
          with: {
            "fetch-depth": 1,
          },
        }),
      )
      .add(rustToolchain({
        with: {
          components: "clippy",
        },
      }))
      .add(rustCache({}))
      .add(
        {
          run: "cargo clippy --all-targets --all-features -- -D warnings",
        },
      ),
);

const test = new Job("ubuntu-latest").steps(
  (s) =>
    s
      .add(
        checkout({
          with: {
            "fetch-depth": 1,
          },
        }),
      )
      .add(
        rustToolchain({}),
      )
      .add(rustCache({}))
      .add(
        { run: "cargo test --all-features" },
      ),
);

new Workflow(
  {
    name: "PR",
    on: {
      pull_request: {
        branches: ["main"],
      },
    },
  },
).jobs(
  (j) =>
    j
      .add(
        "fmt",
        fmt,
      )
      .add(
        "test",
        test,
      )
      .add(
        "clippy",
        clippy,
      ),
).build("pr");
