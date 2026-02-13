import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");

const audit = new Job("ubuntu-latest")
  .addStep(checkout({}))
  .addStep({
    name: "Install cargo-audit",
    run: "cargo install cargo-audit",
  })
  .addStep({
    name: "Run security audit",
    run: "cargo audit",
  });

const workflow = new Workflow({
  name: "Security Audit",
  on: {
    schedule: [{ cron: "0 3 * * *" }],
  },
}).addJob("audit", audit);

workflow.build("security-audit");
