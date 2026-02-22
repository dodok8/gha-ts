import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const rustToolchain = getAction("dtolnay/rust-toolchain@stable");
const rustCache = getAction("Swatinem/rust-cache@v2");
const miseAction = getAction("jdx/mise-action@v2");
const ghPages = getAction("peaceiris/actions-gh-pages@v4");

const deploy = new Job("ubuntu-latest", {
  permissions: {
    contents: "write",
  },
}).steps((s) =>
  s
    .add(checkout({ with: { "fetch-depth": 0 } }))
    .add(rustToolchain({}))
    .add(rustCache({}))
    .add({ name: "Build gaji", run: "cargo build --release" })
    .add({ name: "Generate types", run: "./target/release/gaji dev" })
    .add(miseAction({}))
    .add({ name: "Install dependencies", run: "pnpm install --frozen-lockfile", "working-directory": "docs" })
    .add({ name: "Build docs", run: "pnpm docs:build", "working-directory": "docs" })
    .add(ghPages({
      with: {
        github_token: "${{ secrets.GITHUB_TOKEN }}",
        publish_dir: "./docs/.vitepress/dist",
        publish_branch: "vitepress",
        cname: "gaji.gaebalgom.work",
      },
    })),
);

new Workflow({
  name: "Deploy Docs",
  on: {
    push: { branches: ["main"], paths: ["docs/**"] },
    workflow_dispatch: {},
  },
}).jobs((j) => j.add("deploy", deploy)).build("vitepress");
