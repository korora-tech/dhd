export default defineModule("test-directory")
    .description("Test directory creation")
    .actions([
        directory({
            path: "/tmp/test",
            requires_privilege_escalation: false,
        }),
    ]);
