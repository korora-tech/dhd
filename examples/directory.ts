export default defineModule("directory")
    .description("Create directories with optional privilege escalation")
    .actions([
        directory({
            path: "~/.config/myapp",
            escalate: false,
        }),
        directory({
            path: "/etc/myapp",
            escalate: true, // this field should default to false and be optional
        }),
        directory({
            path: "~/.cache/myapp/logs",
            escalate: false,
        }),
    ]);
