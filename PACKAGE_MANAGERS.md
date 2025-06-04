# Package Managers Support

DHD supports a wide variety of package managers across different platforms and ecosystems.

## System Package Managers

### Linux
- **APT** (Debian/Ubuntu/Mint/Pop/Raspbian)
- **Pacman** (Arch/Manjaro/EndeavourOS)
- **Paru** (AUR helper for Arch-based systems)
- **Flatpak** (Universal Linux packages)

### macOS
- **Homebrew/Brew** (also supports Linux)

### Windows
- **Winget** (Windows Package Manager)

### BSD
- **pkg** (FreeBSD/DragonFly/NetBSD)
- **ports** (BSD Ports system)

## Language/Runtime Package Managers

- **npm** (Node.js packages)
- **Bun** (Fast JavaScript runtime and package manager)
- **Cargo** (Rust packages)
- **Deno** (Deno modules)

## Platform Support

Package managers automatically check platform compatibility using the `os_info` crate. The system will:

1. Detect your operating system and distribution
2. Only allow package managers that support your platform
3. Automatically select the best available package manager if none is specified

## Privilege Escalation

DHD is agnostic about privilege escalation tools. It will automatically detect and use:
- `run0` (systemd-based, preferred)
- `doas` (OpenBSD-style)
- `sudo` (traditional)

## Usage Examples

```typescript
// Use system package manager (auto-detected)
packageInstall(["git", "vim"])

// Use specific package manager
packageInstall(["neovim"], { using: "flatpak" })

// Install Node.js packages globally
packageInstall(["typescript", "@types/node"], { using: "npm" })

// Install from AUR on Arch
packageInstall(["visual-studio-code-bin"], { using: "paru" })
```

## Bootstrap Support

Some package managers can bootstrap themselves if not installed:
- Homebrew (installs via official script)
- Bun (installs via official script)
- Cargo (installs via rustup)
- Deno (installs via official script)
- Paru (builds from AUR using pacman)
- pkg (bootstraps on fresh BSD installs)
- ports (downloads and extracts ports tree)