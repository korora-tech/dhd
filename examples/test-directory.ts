export default defineModule("test-directory")
    .description("Test directory creation")
    .actions([
        directory({
            path: "/tmp/test",
            escalate: false,
        }),
    ]);
