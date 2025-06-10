export default defineModule("directory")
    .description("Create directories with optional privilege escalation")
    .actions([
        directory({
            path: "~/.config/myapp",
            requires_privilege_escalation: false,
        }),
        directory({
            path: "/etc/myapp",
            requires_privilege_escalation: true, // this field should default to false and be optional
        }),
        directory({
            path: "~/.cache/myapp/logs",
            requires_privilege_escalation: false,
        }),
    ]);
