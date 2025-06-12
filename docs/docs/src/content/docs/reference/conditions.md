---
title: Conditions
description: Learn how to use conditions to control when modules and actions are executed
---

# Conditions

DHD provides a powerful, type-safe condition system that allows you to control when modules and actions are executed based on system properties, file existence, command output, and more.

## Module-Level Conditions

You can add conditions to entire modules using the `.when()` method:

```typescript
export default defineModule("nvidia-driver")
    .description("NVIDIA proprietary drivers")
    .when(property("hardware.gpu_vendor").equals("nvidia"))
    .actions([
        packageInstall({ names: ["nvidia-driver", "nvidia-utils"] })
    ]);
```

When a module's condition evaluates to false, all of its actions are skipped.

## Action-Level Conditions

*Note: Action-level conditions are planned for a future release. Currently, only module-level conditions are supported.*

Individual actions will be able to have conditions using `onlyIf` and `skipIf`:

```typescript
// Coming soon!
export default defineModule("dotfiles")
    .actions([
        // Only create link if source file exists
        onlyIf(
            linkFile({ 
                source: "~/.config/wezterm",
                target: "dotfiles/wezterm",
                force: true
            }),
            [fileExists("dotfiles/wezterm/wezterm.lua")]
        ),
        
        // Skip action if condition is true
        skipIf(
            executeCommand({ 
                command: "systemctl",
                args: ["enable", "firewall"],
                escalate: true
            }),
            [commandExists("ufw")]
        )
    ]);
```

## Condition Types

### File and Directory Checks

```typescript
// Check if a file exists
fileExists("/etc/passwd")

// Check if a directory exists
directoryExists("/home/user/.config")
```

### Command Checks

```typescript
// Check if a command exists in PATH
commandExists("git")

// Check if a command succeeds (exit code 0)
commandSucceeds("ping", ["-c", "1", "google.com"])

// Using the command builder for more complex checks
command("lsusb").exists()
command("lsusb").succeeds()
command("lsusb").contains("fingerprint", true)  // case-insensitive
```

### System Properties

System properties allow you to check various system attributes:

```typescript
// Check boolean properties
property("hardware.fingerprint").isTrue()
property("hardware.tpm").isFalse()

// Check string properties
property("os.distro").equals("ubuntu")
property("os.family").equals("debian")
property("gpu.vendor").contains("nvidia")

// Check numeric properties (future)
property("os.version").greaterThan(22)
```

Available system properties:
- `os.family` - OS family (debian, fedora, arch, etc.)
- `os.distro` - Distribution name (ubuntu, fedora, etc.)
- `os.version` - OS version (22.04, 39, etc.)
- `os.codename` - Version codename (jammy, etc.)
- `hardware.fingerprint` - Has fingerprint reader (boolean)
- `hardware.tpm` - Has TPM chip (boolean)
- `hardware.gpu_vendor` - GPU vendor (nvidia, amd, intel)
- `auth.type` - Authentication type (local, central, ldap)
- `auth.method` - Authentication method (password, biometric, smartcard)
- `user.name` - Current username
- `user.shell` - User's shell
- `user.home` - User's home directory

### Environment Variables

```typescript
// Check if environment variable exists
envVar("DEVELOPMENT_MACHINE")

// Check if environment variable has specific value
envVar("NODE_ENV", "production")
```

### Logical Operators

Combine conditions using logical operators:

```typescript
// OR - at least one condition must be true
or([
    property("os.distro").equals("ubuntu"),
    property("os.distro").equals("fedora")
])

// AND - all conditions must be true
and([
    property("os.family").equals("debian"),
    commandExists("apt")
])

// NOT - negate a condition
not(commandExists("systemctl"))

// Complex combinations
or([
    property("hardware.fingerprint").isTrue(),
    and([
        command("lsusb").contains("fingerprint", true),
        fileExists("/usr/lib/fprintd")
    ])
])
```

## Verbose Mode

Use the `--verbose` flag to see condition evaluation details:

```bash
dhd apply --verbose --module fprintd
```

Output:
```
● Planning module: fprintd
  ⏭️  Module skipped due to condition: any of: [hardware.fingerprint == true, command succeeds: lsusb | grep -qi 'fingerprint']
```

## Examples

### Desktop Environment Based on Distro

```typescript
export default defineModule("desktop-environment")
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
```

### Hardware-Specific Drivers

```typescript
export default defineModule("fingerprint-auth")
    .description("Fingerprint authentication support")
    .when(
        or([
            property("hardware.fingerprint").isTrue(),
            command("lsusb").contains("fingerprint", true)
        ])
    )
    .actions([
        packageInstall({ names: ["fprintd", "libpam-fprintd"] })
    ]);
```

### Development Environment

```typescript
export default defineModule("dev-tools")
    .description("Development tools")
    .when(envVar("DEVELOPMENT_MACHINE", "true"))
    .actions([
        packageInstall({ names: ["git", "vim", "tmux"] }),
        onlyIf(
            packageInstall({ names: ["docker", "docker-compose"] }),
            [not(commandExists("podman"))]
        )
    ]);
```

## Best Practices

1. **Use specific conditions**: Prefer `property("os.distro").equals("ubuntu")` over generic command checks when possible
2. **Combine conditions logically**: Use `and`, `or`, and `not` to create precise conditions
3. **Test conditions**: Use `--verbose --dry-run` to verify conditions evaluate as expected
4. **Document complex conditions**: Add descriptions to modules with complex conditions
5. **Fail gracefully**: Consider what happens when conditions can't be evaluated (e.g., missing commands)

## TypeScript Support

The condition system is fully typed, providing excellent IDE support:

- Autocompletion for all condition functions
- Type checking for condition parameters
- IntelliSense documentation
- Compile-time validation

This ensures you maintain DHD's core tenet of excellence in authoring while writing powerful conditional logic.