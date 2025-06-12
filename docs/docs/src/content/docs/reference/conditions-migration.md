---
title: Migrating from CEL to Typed Conditions
description: Guide for migrating from the CEL-based condition system to the new typed condition API
---

# Migrating from CEL to Typed Conditions

This guide helps you migrate from the previous CEL (Common Expression Language) based condition system to the new typed condition API.

## Why the Change?

The CEL-based system used string expressions that broke TypeScript's LSP support:
```typescript
// Old CEL approach - no TypeScript support
.when("hardware.fingerprint || command('lsusb | grep -qi fingerprint')")
```

The new typed API maintains full TypeScript support with autocompletion and type checking:
```typescript
// New typed approach - full TypeScript support
.when(
    or([
        property("hardware.fingerprint").isTrue(),
        command("lsusb").contains("fingerprint", true)
    ])
)
```

## Migration Examples

### Simple Property Checks

**Before (CEL):**
```typescript
.when("hardware.fingerprint")
.when("os.distro == 'ubuntu'")
```

**After (Typed):**
```typescript
.when(property("hardware.fingerprint").isTrue())
.when(property("os.distro").equals("ubuntu"))
```

### Command Checks

**Before (CEL):**
```typescript
.when("command('git')")
.when("command('lsusb | grep -qi fingerprint')")
```

**After (Typed):**
```typescript
.when(commandExists("git"))
.when(command("lsusb").contains("fingerprint", true))
```

### Logical Operators

**Before (CEL):**
```typescript
.when("os.distro == 'ubuntu' || os.distro == 'fedora'")
.when("os.family == 'debian' && command('apt')")
.when("!command('systemctl')")
```

**After (Typed):**
```typescript
.when(
    or([
        property("os.distro").equals("ubuntu"),
        property("os.distro").equals("fedora")
    ])
)
.when(
    and([
        property("os.family").equals("debian"),
        commandExists("apt")
    ])
)
.when(not(commandExists("systemctl")))
```

### Complex Conditions

**Before (CEL):**
```typescript
.when("hardware.fingerprint || (command('lsusb | grep -qi fingerprint') && file('/usr/lib/fprintd'))")
```

**After (Typed):**
```typescript
.when(
    or([
        property("hardware.fingerprint").isTrue(),
        and([
            command("lsusb").contains("fingerprint", true),
            fileExists("/usr/lib/fprintd")
        ])
    ])
)
```

## Benefits of the New System

1. **Full TypeScript Support**
   - Autocompletion for all condition functions
   - Type checking at compile time
   - IntelliSense documentation
   - Refactoring support

2. **Better Error Messages**
   - Compile-time errors for invalid conditions
   - Clear runtime error messages
   - No string parsing errors

3. **Improved Readability**
   - Clear, structured conditions
   - Self-documenting code
   - Easier to understand complex logic

4. **Extensibility**
   - Easy to add new condition types
   - Builder pattern for custom conditions
   - Type-safe extensions

## Quick Reference

| CEL Expression | Typed API |
|----------------|-----------|
| `property` | `property("path").equals(value)` |
| `property == true` | `property("path").isTrue()` |
| `property == false` | `property("path").isFalse()` |
| `property contains 'text'` | `property("path").contains("text")` |
| `command('cmd')` | `commandExists("cmd")` |
| `command('cmd \| grep text')` | `command("cmd").contains("text", false)` |
| `file('/path')` | `fileExists("/path")` |
| `dir('/path')` | `directoryExists("/path")` |
| `env.VAR` | `envVar("VAR")` |
| `env.VAR == 'value'` | `envVar("VAR", "value")` |
| `expr1 \|\| expr2` | `or([expr1, expr2])` |
| `expr1 && expr2` | `and([expr1, expr2])` |
| `!expr` | `not(expr)` |

## Future Enhancements

- Action-level conditions with `onlyIf` and `skipIf` (coming soon)
- Custom condition builders
- More system properties
- Enhanced comparison operators