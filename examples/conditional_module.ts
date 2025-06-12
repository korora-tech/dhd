// Example of using the typed condition API in DHD modules

// Simple property check
export default defineModule("nvidia-driver")
    .description("NVIDIA proprietary drivers")
    .tags(["drivers", "graphics"])
    .when(property("hardware.gpu_vendor").equals("nvidia"))
    .actions([
        packageInstall({ names: ["nvidia-driver", "nvidia-utils"] })
    ]);

// Command check with contains
export const fingerprintModule = defineModule("fingerprint-auth")
    .description("Fingerprint authentication support")
    .tags(["security", "biometrics"])
    .when(command("lsusb").contains("fingerprint", true))
    .actions([
        packageInstall({ names: ["fprintd", "libpam-fprintd"] }),
        executeCommand({ 
            command: "pam-auth-update",
            args: ["--enable", "fprintd"],
            escalate: true
        })
    ]);

// Complex conditions with or/and
export const desktopModule = defineModule("desktop-environment")
    .description("Desktop environment configuration")
    .when(
        or([
            property("os.distro").equals("ubuntu"),
            property("os.distro").equals("fedora"),
            and([
                property("os.family").equals("debian"),
                commandExists("apt")
            ])
        ])
    )
    .actions([
        packageInstall({ names: ["gnome-shell", "gnome-terminal"] })
    ]);

// File and directory existence checks
export const dotfilesModule = defineModule("dotfiles")
    .description("User dotfiles configuration")
    .when(directoryExists("/home/user/.config"))
    .actions([
        linkFile({ 
            source: "~/.config/nvim",
            target: "dotfiles/nvim",
            force: true
        }),
        onlyIf(
            linkFile({ 
                source: "~/.config/wezterm",
                target: "dotfiles/wezterm",
                force: true
            }),
            [fileExists("dotfiles/wezterm/wezterm.lua")]
        )
    ]);

// Environment variable check
export const developmentModule = defineModule("dev-tools")
    .description("Development tools")
    .when(envVar("DEVELOPMENT_MACHINE", "true"))
    .actions([
        packageInstall({ names: ["git", "vim", "tmux"] })
    ]);

// Negation example
export const nonSystemdModule = defineModule("non-systemd-init")
    .description("Configuration for non-systemd systems")
    .when(not(commandExists("systemctl")))
    .actions([
        executeCommand({ 
            command: "echo",
            args: ["Running on non-systemd system"],
            escalate: false
        })
    ]);