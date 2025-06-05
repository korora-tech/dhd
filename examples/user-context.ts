import { defineModule, userGroup, linkDotfile, fileWrite, executeCommand } from "@dhd/types";

export default defineModule("user-context")
    .description("Example of using ctx.user and ctx.user.homedir")
    .with((ctx) => [
        // Using ctx.user returns the current username
        userGroup({
            user: ctx.user,
            groups: ["docker", "libvirt", "wheel"],
            append: true
        }),

        // Using ctx.user.homedir returns the home directory path
        linkDotfile({
            source: "config/bashrc",
            target: ctx.user.homedir + "/.bashrc",
            backup: true
        }),

        // You can use it in file paths
        fileWrite({
            destination: ctx.user.homedir + "/.config/app/settings.json",
            content: JSON.stringify({
                username: ctx.user,
                configured: true
            }, null, 2),
            mode: 0o644
        }),

        // Combine with platform selection for user-specific config
        executeCommand({
            command: "chown",
            args: [
                ctx.user + ":" + ctx.user,
                ctx.user.homedir + "/.config"
            ],
            privilegeEscalation: true
        }),

        // Real-world example: Git configuration
        fileWrite({
            destination: ctx.user.homedir + "/.gitconfig",
            content: `[user]
    name = ${ctx.user}
    email = ${ctx.user}@localhost

[core]
    editor = nvim
`,
            backup: true
        })
    ]);