import { defineModule, systemdService, systemdSocket } from "@korora-tech/dhd";

export default defineModule("systemd")
	.description("Example of systemdService and systemdSocket")
	.with(() => [
		// Create a user systemd service
		systemdService({
			name: "myapp",
			content: `[Unit]
Description=My Application Service
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/myapp
Restart=on-failure
RestartSec=10

[Install]
WantedBy=default.target`,
			user: true,
			enable: true,
			start: true,
			reload: true,
		}),

		// Create a system service for a server application
		systemdService({
			name: "webserver.service",
			content: `[Unit]
Description=Custom Web Server
After=network.target

[Service]
Type=notify
ExecStart=/opt/webserver/bin/server
ExecReload=/bin/kill -HUP $MAINPID
Restart=always
User=webserver
Group=webserver

[Install]
WantedBy=multi-user.target`,
			user: false,
			enable: true,
			start: false, // Don't start immediately
			reload: true,
		}),

		// Create a systemd socket for socket activation
		systemdSocket({
			name: "myapp.socket",
			content: `[Unit]
Description=My App Socket

[Socket]
ListenStream=/run/myapp.sock
SocketMode=0660
SocketUser=myapp
SocketGroup=myapp

[Install]
WantedBy=sockets.target`,
			user: false,
			enable: true,
			start: true,
			reload: true,
		}),

		// Create a user socket for development
		systemdSocket({
			name: "dev-server",
			content: `[Unit]
Description=Development Server Socket

[Socket]
ListenStream=8080
Accept=no

[Install]
WantedBy=sockets.target`,
			user: true,
			enable: true,
			start: true,
			reload: true,
		}),

		// Example: Git credential cache service (from rawkOS)
		systemdService({
			name: "git-credential-cache",
			content: `[Unit]
Description=Git Credential Cache

[Service]
Type=simple
ExecStart=/usr/local/bin/git-credential-cache --socket %t/git-credential-cache.sock
Restart=on-failure

[Install]
WantedBy=default.target`,
			user: true,
			enable: true,
			start: true,
			reload: true,
		}),

		// Corresponding socket for git credential cache
		systemdSocket({
			name: "git-credential-cache",
			content: `[Unit]
Description=Git Credential Cache Socket

[Socket]
ListenStream=%t/git-credential-cache.sock
SocketMode=0600

[Install]
WantedBy=sockets.target`,
			user: true,
			enable: true,
			start: true,
			reload: true,
		}),
	]);