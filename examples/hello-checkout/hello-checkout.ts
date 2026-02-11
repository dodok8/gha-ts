// Hello Checkout - Test file for gha-ts parser
// This file is used to test the TypeScript parser and type generation
import { getAction } from "../../generated";

const checkout = getAction("actions/checkout@v4");
const setupNode = getAction("actions/setup-node@v4");

// Test nested expressions
const workflow = {
    name: "Hello Checkout",
    jobs: {
        build: {
            steps: [
                checkout({ name: "Checkout code" }),
                setupNode({
                    name: "Setup Node.js",
                    with: {
                        "node-version": "20",
                    },
                }),
                {
                    name: "Run tests",
                    run: "npm test",
                },
            ],
        },
    },
};

console.log(JSON.stringify(workflow, null, 2));
