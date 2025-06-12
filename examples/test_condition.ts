// Test module to verify condition system works
export default defineModule("test-condition")
    .description("Test condition evaluation")
    .tags(["test"])
    .when(
        or([
            commandExists("ls"),  // This should always be true on Linux
            fileExists("/etc/passwd")  // This should also be true
        ])
    )
    .actions([
        executeCommand({
            command: "echo",
            args: ["Condition passed!"],
            escalate: false
        })
    ]);