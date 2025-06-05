import { systemdService, systemdSocket } from "@dhd/types";

// Example 1: Simple typed service
systemdService({
    name: "my-backup",
    content: {
        unit: {
            description: "My Backup Service",
            after: ["network.target"],
        },
        service: {
            type: "oneshot",
            execStart: ["/home/user/scripts/backup.sh"],
            user: "user",
            group: "user",
        },
        install: {
            wantedBy: ["default.target"],
        },
    },
    user: true,
    enable: true,
});

// Example 2: Complex service with security settings
systemdService({
    name: "secure-api",
    content: {
        unit: {
            description: "Secure API Service",
            documentation: ["https://example.com/docs"],
            after: ["network.target", "postgresql.service"],
            wants: ["postgresql.service"],
        },
        service: {
            type: "notify",
            execStart: ["/usr/local/bin/api-server --config=/etc/api/config.yaml"],
            execStartPre: [
                "/usr/local/bin/api-server --check-config=/etc/api/config.yaml",
            ],
            restart: "on-failure",
            restartSec: 5,
            user: "api-user",
            group: "api-group",
            workingDirectory: "/var/lib/api",
            environment: {
                NODE_ENV: "production",
                API_PORT: "8080",
            },
            environmentFile: ["/etc/api/environment"],
            
            // Security settings
            privateTmp: true,
            privateDevices: true,
            protectSystem: "strict",
            protectHome: true,
            noNewPrivileges: true,
            readWritePaths: ["/var/lib/api", "/var/log/api"],
            readOnlyPaths: ["/etc/api"],
            
            // Resource limits
            limitNOFILE: "65536",
            limitNPROC: "512",
            
            // Logging
            standardOutput: "journal",
            standardError: "journal",
            syslogIdentifier: "secure-api",
        },
        install: {
            wantedBy: ["multi-user.target"],
        },
    },
    system: true,
    enable: true,
    start: true,
    reload: true,
});

// Example 3: Timer-activated service
systemdService({
    name: "daily-cleanup",
    content: {
        unit: {
            description: "Daily cleanup tasks",
            condition: {
                pathExists: "/var/lib/cleanup/enabled",
            },
        },
        service: {
            type: "oneshot",
            execStart: [
                "/usr/local/bin/cleanup.sh --remove-old-logs",
                "/usr/local/bin/cleanup.sh --compress-archives",
            ],
            nice: 10,
            ioSchedulingClass: "idle",
        },
    },
    user: false,
    enable: false, // Will be activated by timer
});

// Example 4: Socket-activated service
systemdSocket({
    name: "api",
    content: {
        unit: {
            description: "API Socket",
            before: ["sockets.target"],
        },
        socket: {
            listenStream: ["0.0.0.0:8080"],
            bindIPv6Only: "both",
            accept: false,
            service: "api.service",
            reusePort: true,
            keepAlive: true,
            keepAliveTimeSec: 60,
            noDelay: true,
        },
        install: {
            wantedBy: ["sockets.target"],
        },
    },
    system: true,
    enable: true,
    start: true,
});

// Example 5: Unix socket with permissions
systemdSocket({
    name: "app-control",
    content: {
        unit: {
            description: "Application Control Socket",
        },
        socket: {
            listenStream: ["/run/app/control.sock"],
            socketUser: "app",
            socketGroup: "app",
            socketMode: "0660",
            accept: false,
            removeOnStop: true,
        },
        install: {
            wantedBy: ["sockets.target"],
        },
    },
    system: true,
    enable: true,
});

// Example 6: Backward compatibility - raw unit file content still works
systemdService({
    name: "legacy-service",
    content: `[Unit]
Description=Legacy Service
After=network.target

[Service]
Type=simple
ExecStart=/usr/bin/legacy-app
Restart=always

[Install]
WantedBy=multi-user.target`,
    system: true,
    enable: true,
});

// Example 7: Docker service management
systemdService({
    name: "docker",
    enable: false,  // Disable Docker service
    system: true,
});