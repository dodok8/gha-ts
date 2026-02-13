import { JavaScriptAction } from "../generated/index.js";

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
