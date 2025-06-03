import { defineModule, executeCommand } from "../types";

export default defineModule("withDependency")
  .description("Example of adding a dependency")
  .depends("executeCommand")
  .with(() => [
    executeCommand({
      command: "echo",
      args: ["Running after executeCommand module..."]
    })
  ]);
