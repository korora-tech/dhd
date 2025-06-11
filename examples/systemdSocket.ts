export default defineModule("systemdSocket")
    .description("Create systemd socket unit files for service activation")
    .actions([
        systemdSocket({
            name: "myapp.socket",
            description: "Socket for MyApp service",
            listenStream: "~/.local/share/myapp/myapp.sock",
            scope: "user",
        }),
        systemdSocket({
            name: "webapp.socket",
            description: "Web application socket",
            listenStream: "/run/webapp/webapp.sock",
            scope: "system",
        }),
        systemdSocket({
            name: "api-server.socket",
            description: "API server socket for inter-process communication",
            listenStream: "~/.cache/api-server/api.sock",
            scope: "user",
        }),
    ]);
