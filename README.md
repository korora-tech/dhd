# DHD: Declarative Home Deployment

> Manage your home directory, dotfiles, and system configuration using TypeScript

DHD is a modern configuration management tool that lets you define your system setup declaratively using TypeScript. Whether you're setting up a new machine, managing dotfiles, or ensuring consistent development environments across systems, DHD makes it simple, type-safe, and reproducible.

## Features

- **TypeScript-Powered**: Write your configuration in TypeScript with full type safety and IDE support
- **Cross-Platform**: Works on Linux (Ubuntu, Debian, Fedora, Arch, NixOS), macOS, and Windows
- **Declarative**: Define what you want, not how to get there
- **Idempotent**: Run multiple times safely - DHD only makes necessary changes
- **Modular**: Organize configurations into reusable modules with tags and dependencies
- **Parallel Execution**: Fast execution with intelligent dependency resolution
- **Dry Run Mode**: Preview changes before applying them

## Quick Start

### Installation

```bash
# Install from source (Rust required)
cargo install --path .

# Or download pre-built binaries (coming soon)
# curl -sSL https://install.dhd.korora.tech | sh
```

### Your First Module

Create a file called `essentials.ts`:

```typescript
import { defineModule, packageInstall, gitConfig, directory } from "dhd";

export default defineModule("essentials")
  .description("My essential tools and configs")
  .tags(["core", "development"])
  .actions([
    // Install essential packages
    packageInstall({
      names: ["git", "neovim", "tmux", "ripgrep", "fzf"]
    }),
    
    // Configure git
    gitConfig({
      scope: "global",
      settings: {
        "user.name": "Your Name",
        "user.email": "your.email@example.com",
        "init.defaultBranch": "main"
      }
    }),
    
    // Create config directories
    directory({ path: "~/.config/nvim" }),
    directory({ path: "~/.config/tmux" })
  ]);
```

### Apply Your Configuration

```bash
# See what modules are available
dhd list

# Preview what would change
dhd apply --dry-run

# Apply all modules
dhd apply

# Apply specific tags
dhd apply --tags core

# Apply specific modules
dhd apply --modules essentials
```

## Core Concepts

### Modules

Modules are the top-level organizational unit in DHD. Each TypeScript file can export one or more modules that group related configuration tasks.

```typescript
export default defineModule("my-module")
  .description("What this module does")
  .tags(["category", "type"])
  .dependsOn(["other-module"])
  .actions([
    // Your configuration actions here
  ]);
```

### Actions

Actions are high-level operations that DHD can perform:

- **Package Management**: Install/remove packages across different package managers
- **File Operations**: Create directories, copy files, manage symlinks
- **System Services**: Manage systemd services and sockets
- **Command Execution**: Run arbitrary commands with privilege escalation
- **Downloads**: Fetch files from HTTP/HTTPS URLs
- **Git Configuration**: Manage git settings at system/global/local scope
- **Desktop Environment**: Configure GNOME extensions, import dconf settings

### Platform-Specific Configuration

Use platform detection for conditional actions:

```typescript
import { defineModule, packageInstall, platform } from "dhd";

export default defineModule("editor")
  .description("Install preferred editor")
  .actions([
    platform.select({
      macos: packageInstall({ names: ["neovim"], manager: "brew" }),
      linux: packageInstall({ names: ["neovim"] }),
      windows: packageInstall({ names: ["neovim"], manager: "scoop" })
    })
  ]);
```

## Examples

### Development Environment

```typescript
export default defineModule("dev-environment")
  .description("Complete development setup")
  .tags(["development"])
  .actions([
    // Language runtimes
    packageInstall({ 
      names: ["nodejs", "python3", "rustup", "golang"],
      manager: platform.isLinux() ? "native" : "brew"
    }),
    
    // Development tools
    packageInstall({
      names: ["docker", "docker-compose", "kubectl", "terraform"]
    }),
    
    // Shell configuration
    copyFile({
      source: "./configs/zshrc",
      destination: "~/.zshrc"
    }),
    
    // VS Code settings
    linkFile({
      source: "./configs/vscode-settings.json",
      destination: "~/.config/Code/User/settings.json"
    })
  ]);
```

### Dotfiles Management

```typescript
export default defineModule("dotfiles")
  .description("Symlink all dotfiles")
  .tags(["dotfiles"])
  .actions([
    linkDirectory({
      source: "./dotfiles",
      destination: "~/.config",
      merge: true
    }),
    
    // Run post-install scripts
    executeCommand({
      command: "~/.config/install-scripts/post-install.sh",
      workingDirectory: "~"
    })
  ]);
```

### System Services

```typescript
export default defineModule("backup-automation")
  .description("Automated backup service")
  .tags(["backup", "automation"])
  .actions([
    // Create backup script
    copyFile({
      source: "./scripts/backup.sh",
      destination: "/usr/local/bin/backup.sh",
      mode: "755",
      escalate: true
    }),
    
    // Install systemd service
    systemdService({
      name: "backup.service",
      description: "Automated backup service",
      execStart: "/usr/local/bin/backup.sh",
      serviceType: "oneshot",
      scope: "user"
    }),
    
    // Install systemd timer
    systemdSocket({
      name: "backup.timer",
      description: "Run backup daily",
      timerOnCalendar: "daily",
      timerPersistent: true,
      scope: "user"
    }),
    
    // Enable the timer
    systemdManage({
      unit: "backup.timer",
      operation: "enable"
    })
  ]);
```

## Command Reference

```bash
# List all discovered modules
dhd list

# Show module details
dhd list --verbose

# Apply configurations
dhd apply [OPTIONS]
  --dry-run              Preview changes without applying
  --modules <MODULES>    Apply specific modules (comma-separated)
  --tags <TAGS>         Apply modules with specific tags
  --exclude-tags <TAGS>  Exclude modules with specific tags
  --concurrency <N>      Number of parallel operations (default: 10)

# Generate TypeScript definitions
dhd codegen
```

## Configuration

DHD looks for modules in:
1. Current directory
2. `dhd/` subdirectory
3. `modules/` subdirectory
4. `.dhd/` subdirectory

Files matching these patterns are ignored:
- `node_modules/**`
- `dist/**`
- `build/**`
- `*.test.ts`
- `*.spec.ts`

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## License

DHD is licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Acknowledgments

DHD is inspired by configuration management tools like Ansible, Puppet, and Chef, but designed specifically for personal system management with a focus on developer experience and type safety.