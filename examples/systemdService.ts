export default defineModule("systemdService")
    .description("Create systemd service unit files for daemon management")
    .actions([
        systemdservice({
            name: "myapp.service",
            description: "MyApp background service",
            exec_start: "~/.local/bin/myapp --daemon",
            service_type: "simple",
            scope: "user",
            restart: "on-failure",
            restart_sec: 5,
        }),
        systemdservice({
            name: "backup-daemon.service",
            description: "Automated backup daemon",
            exec_start: "/usr/local/bin/backup-daemon",
            service_type: "forking",
            scope: "system",
            restart: "always",
            restart_sec: 10,
        }),
        systemdservice({
            name: "sync-tool.service",
            description: "File synchronization tool",
            exec_start: "~/.local/bin/sync-tool --config ~/.config/sync-tool/config.yaml",
            service_type: "simple",
            scope: "user",
        }),
    ]);
