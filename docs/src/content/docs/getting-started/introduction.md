---
title: Introduction
description: Learn what DHD is and how it can help manage your system configuration
---

# Introduction to DHD

DHD (Declarative Home Deployment) is a modern configuration management tool that allows you to define and manage your system configuration using TypeScript. It's designed to make setting up new machines, managing dotfiles, and maintaining consistent environments across systems both simple and reliable.

## What Problems Does DHD Solve?

### The Setup Problem
Setting up a new machine is often a tedious, error-prone process:
- Installing dozens of packages manually
- Copying configuration files from various sources
- Remembering all the customizations you've made over time
- Dealing with platform-specific differences

### The Maintenance Problem
Keeping multiple systems in sync is challenging:
- Dotfiles scattered across different repositories
- No clear way to track what's installed where
- Manual updates that often get forgotten
- Configuration drift between machines

### The Automation Problem
Traditional automation tools have their own challenges:
- Complex syntax and steep learning curves
- Limited IDE support and type safety
- Platform-specific scripts that don't transfer
- All-or-nothing execution models

## How DHD Helps

DHD addresses these problems by providing:

### 1. **TypeScript-Based Configuration**
```typescript
export default defineModule("development")
  .description("Development environment setup")
  .actions([
    packageInstall({ names: ["git", "nodejs", "docker"] }),
    gitConfig({ scope: "global", settings: { "user.name": "John Doe" } })
  ]);
```

- Full type safety and autocompletion
- Familiar syntax for developers
- Excellent IDE support
- Easy to version control

### 2. **Declarative Approach**
Define the desired state, not the steps to get there:
- DHD figures out what needs to be changed
- Idempotent operations - run as many times as needed
- Clear, readable configuration files

### 3. **Cross-Platform Support**
Write once, run anywhere:
```typescript
platform.select({
  macos: packageInstall({ names: ["neovim"], manager: "brew" }),
  linux: packageInstall({ names: ["neovim"] }),
  windows: packageInstall({ names: ["neovim"], manager: "scoop" })
})
```

### 4. **Modular Organization**
- Group related configurations into modules
- Use tags to categorize and filter
- Define dependencies between modules
- Share modules across projects

## Who Is DHD For?

DHD is perfect for:

- **Developers** who want to automate their development environment setup
- **DevOps Engineers** managing multiple workstations
- **Teams** needing consistent development environments
- **Anyone** tired of manual system configuration

## Core Philosophy

DHD is built on these principles:

1. **Simplicity First**: Configuration should be easy to write and understand
2. **Type Safety**: Catch errors at compile time, not runtime
3. **Cross-Platform**: Your configuration should work everywhere
4. **Incremental Adoption**: Start small and grow as needed
5. **Developer Experience**: Great tooling and documentation matter

## What's Next?

Ready to get started with DHD?

- [Install DHD](/getting-started/installation/) on your system
- Follow the [Quick Start Guide](/getting-started/quick-start/) to create your first module
- Explore the [Core Concepts](/concepts/modules/) to understand how DHD works
- Browse [Examples](/examples/dotfiles/) for inspiration