export default defineModule("systemdManage")
    .description("Manage existing systemd services without creating unit files")
    .actions([
        // Enable and start a user service
        systemdManage({
            name: "espanso.service",
            operation: "enable-now",
            scope: "user",
        }),
        // Just enable a service without starting it
        systemdManage({
            name: "backup.timer",
            operation: "enable",
            scope: "user",
        }),
        // Restart a system service
        systemdManage({
            name: "nginx.service",
            operation: "restart",
            scope: "system",
        }),
        // Disable and stop a service
        systemdManage({
            name: "old-daemon.service",
            operation: "disable-now",
            scope: "system",
        }),
        // Start a service that's already enabled
        systemdManage({
            name: "docker.service",
            operation: "start",
            scope: "system",
        }),
    ]);