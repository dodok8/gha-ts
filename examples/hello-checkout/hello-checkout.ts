// Hello Checkout - Test file for gaji parser
// This file is used to test the TypeScript parser and type generation
import { getAction, Job, Workflow } from "../../generated/index.js";

const checkout = getAction("actions/checkout@v4");
const setupNode = getAction("actions/setup-node@v4");

const build = new Job("ubuntu-latest")
    .addStep(checkout({
        name: "Checkout code",
        with: {
            "fetch-depth": 1,
        },
    }))
    .addStep(setupNode({
        name: "Setup Node.js",
        with: {
            "node-version": "20",
        },
    }))
    .addStep({
        name: "Run tests",
        run: "npm test",
    });

const workflow = new Workflow({
    name: "Hello Checkout",
    on: {
        push: {
            branches: ["main"],
        },
    },
}).addJob("build", build);

workflow.build("hello-checkout");
