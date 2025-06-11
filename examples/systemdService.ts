export default defineModule("systemdService")
    .description("Create systemd service unit files for daemon management")
    .actions([
        systemdService({
            name: "myapp.service",
            description: "MyApp background service",
            execStart: "~/.local/bin/myapp --daemon",
            serviceType: "simple",
            scope: "user",
            restart: "on-failure",
            restartSec: 5,
        }),
        systemdService({
            name: "backup-daemon.service",
            description: "Automated backup daemon",
            execStart: "/usr/local/bin/backup-daemon",
            serviceType: "forking",
            scope: "system",
            restart: "always",
            restartSec: 10,
        }),
        systemdService({
            name: "sync-tool.service",
            description: "File synchronization tool",
            execStart: "~/.local/bin/sync-tool --config ~/.config/sync-tool/config.yaml",
            serviceType: "simple",
            scope: "user",
        }),
    ]);
