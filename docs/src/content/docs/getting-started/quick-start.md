---
title: Quick Start
description: Create your first DHD module in 5 minutes
---

# Quick Start Guide

This guide will walk you through creating your first DHD module and applying it to your system.

## Step 1: Create Your First Module

Create a new file called `essentials.ts`:

```typescript
import { defineModule, packageInstall, gitConfig, directory, copyFile } from "dhd";

export default defineModule("essentials")
  .description("My essential system setup")
  .tags(["core", "setup"])
  .actions([
    // Install essential packages
    packageInstall({
      names: ["git", "curl", "wget", "vim", "tmux"]
    }),
    
    // Configure git globally
    gitConfig({
      scope: "global",
      settings: {
        "user.name": "Your Name",
        "user.email": "your.email@example.com",
        "init.defaultBranch": "main",
        "pull.rebase": "true"
      }
    }),
    
    // Create common directories
    directory({ path: "~/projects" }),
    directory({ path: "~/.config" }),
    directory({ path: "~/.local/bin" })
  ]);
```

## Step 2: List Available Modules

DHD automatically discovers TypeScript files in your current directory:

```bash
# List all modules
dhd list

# Output:
# Available modules:
#   essentials - My essential system setup [core, setup]
```

## Step 3: Preview Changes (Dry Run)

Before applying changes, preview what DHD will do:

```bash
dhd apply --dry-run

# Output shows planned actions without making changes:
# [DRY RUN] Would install packages: git, curl, wget, vim, tmux
# [DRY RUN] Would set git config user.name = "Your Name"
# [DRY RUN] Would create directory: ~/projects
# ...
```

## Step 4: Apply Your Configuration

When you're ready, apply the configuration:

```bash
dhd apply

# DHD will:
# 1. Check what's already configured
# 2. Apply only necessary changes
# 3. Show progress for each action
# 4. Report success/failure
```

## Step 5: Create a More Complex Module

Let's create a development environment module:

```typescript
// development.ts
import { defineModule, packageInstall, executeCommand, linkFile, platform } from "dhd";

export default defineModule("development")
  .description("Development environment setup")
  .tags(["dev", "tools"])
  .dependsOn(["essentials"])  // Requires essentials module first
  .actions([
    // Language runtimes
    platform.select({
      linux: packageInstall({ 
        names: ["nodejs", "npm", "python3", "python3-pip"] 
      }),
      macos: packageInstall({ 
        names: ["node", "python3"],
        manager: "brew"
      })
    }),
    
    // Development tools
    packageInstall({
      names: ["docker", "docker-compose"],
      manager: platform.isLinux() ? "native" : "brew"
    }),
    
    // Configure npm
    executeCommand({
      command: "npm config set init-author-name 'Your Name'"
    }),
    
    // Link dotfiles
    linkFile({
      source: "./dotfiles/vimrc",
      destination: "~/.vimrc"
    })
  ]);
```

## Step 6: Use Tags for Selective Application

Apply only specific tagged modules:

```bash
# Apply only core modules
dhd apply --tags core

# Apply dev tools but exclude docker
dhd apply --tags dev --exclude-tags docker

# Apply specific modules
dhd apply --modules essentials,development
```

## Common Patterns

### Platform-Specific Actions

```typescript
import { platform, packageInstall } from "dhd";

// Different actions per platform
platform.select({
  ubuntu: packageInstall({ names: ["neovim"] }),
  macos: packageInstall({ names: ["neovim"], manager: "brew" }),
  fedora: packageInstall({ names: ["neovim"], manager: "dnf" })
});

// Conditional execution
if (platform.isLinux()) {
  // Linux-specific actions
}
```

### Working with Dotfiles

```typescript
export default defineModule("dotfiles")
  .description("Symlink all dotfiles")
  .actions([
    // Link individual files
    linkFile({
      source: "./dotfiles/zshrc",
      destination: "~/.zshrc",
      force: true  // Overwrite if exists
    }),
    
    // Link entire directory
    linkDirectory({
      source: "./dotfiles/config",
      destination: "~/.config",
      merge: true  // Merge with existing directory
    })
  ]);
```

### System Services

```typescript
export default defineModule("services")
  .description("Configure system services")
  .actions([
    // Create a systemd service
    systemdService({
      name: "my-backup.service",
      description: "Daily backup service",
      execStart: "/home/user/scripts/backup.sh",
      serviceType: "oneshot"
    }),
    
    // Enable and start the service
    systemdManage({
      unit: "my-backup.service",
      operation: "enable"
    })
  ]);
```

## Best Practices

1. **Start Small**: Begin with essential tools and gradually add more modules
2. **Use Tags**: Organize modules with meaningful tags for easy filtering
3. **Test with Dry Run**: Always preview changes before applying
4. **Version Control**: Keep your DHD modules in a git repository
5. **Document Actions**: Use clear descriptions for modules and actions

## Project Structure

A typical DHD project might look like:

```
my-system-config/
├── essentials.ts       # Core system setup
├── development.ts      # Dev tools and languages
├── dotfiles/          # Configuration files
│   ├── vimrc
│   ├── zshrc
│   └── gitconfig
├── work/              # Work-specific modules
│   ├── vpn.ts
│   └── corporate.ts
└── personal/          # Personal modules
    ├── gaming.ts
    └── media.ts
```

## Next Steps

Now that you've created your first DHD modules:

- Explore [Actions Reference](/reference/actions/) to see all available actions
- Learn about [Modules](/concepts/modules/) in depth
- Check out [Examples](/examples/dotfiles/) for real-world configurations
- Read about [Platform Detection](/concepts/platform-detection/) for cross-platform modules

## Getting Help

- Run `dhd --help` for CLI documentation
- Check the [CLI Reference](/cli/commands/) for all commands
- Visit our [GitHub repository](https://github.com/korora-tech/dhd) for issues and discussions