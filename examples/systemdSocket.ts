export default defineModule("systemdSocket")
    .description("Create systemd socket unit files for service activation")
    .actions([
        systemdsocket({
            name: "myapp.socket",
            description: "Socket for MyApp service",
            listen_stream: "~/.local/share/myapp/myapp.sock",
            scope: "user",
        }),
        systemdsocket({
            name: "webapp.socket",
            description: "Web application socket",
            listen_stream: "/run/webapp/webapp.sock",
            scope: "system",
        }),
        systemdsocket({
            name: "api-server.socket",
            description: "API server socket for inter-process communication",
            listen_stream: "~/.cache/api-server/api.sock",
            scope: "user",
        }),
    ]);
