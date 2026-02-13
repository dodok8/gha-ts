import {
  CallAction,
  JavaScriptAction,
  Job,
  Workflow,
} from "../generated/index.js";

// Define the JavaScript action
const action = new JavaScriptAction(
  {
    name: "Hello World",
    description: "Greet someone and record the time",
    inputs: {
      "who-to-greet": {
        description: "Who to greet",
        required: true,
        default: "World",
      },
    },
    outputs: {
      time: {
        description: "The time we greeted you",
      },
    },
  },
  {
    using: "node20",
    main: "dist/index.js",
  },
);

action.build("hello-world");

// Use the action in a workflow
const helloWorldJob = new Job("ubuntu-latest")
  .addStep({
    name: "Hello world action step",
    id: "hello",
    ...CallAction.from(action).toJSON(),
    with: {
      "who-to-greet": "Mona the Octocat",
    },
  })
  .addStep({
    name: "Get the output time",
    run: 'echo "The time was ${{ steps.hello.outputs.time }}"',
  });

const workflow = new Workflow({
  name: "Use JavaScript Action",
  on: {
    push: {
      paths: [
        "dist",
      ],
    },
  },
}).addJob("hello_world_job", helloWorldJob);

workflow.build("use-js-action");
