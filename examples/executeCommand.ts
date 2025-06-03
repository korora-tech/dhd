import { defineModule, executeCommand } from "../types";

export default defineModule("executeCommand")
	.description("Example of using executeCommand")
	.with((_ctx) => [
		executeCommand({
			command: "echo",
			args: ["Hello, world!"]
		})
	]);
