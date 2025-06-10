export default defineModule("executeCommand")
    .description("Execute a command in the shell")
    .actions([
        executeCommand({
            shell: "bash",
            command: "echo 'Hello, World!'",
        })
    ]);
