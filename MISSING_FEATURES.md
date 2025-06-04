# Missing Features in DHD

Based on analysis of rawkOS modules, here are the features that are currently missing in DHD:

## 1. Action Types

### Custom Actions
- **customAction()** - Generic custom action support for implementing specific behaviors
- Used in GNOME module for:
  - DconfImportAction
  - InstallExtensionsAction
  - RemoveUnwantedAppsAction

### System Integration Actions
- **httpDownload()** - Download files from URLs with checksum verification
  - Parameters: url, destination, checksum (algorithm, value), mode
  - Used in git module for downloading gitsign-credential-cache

- **systemdService()** - Create and manage systemd services
  - Parameters: name, description, execStart, type, scope, restart, restartSec
  - Used in git module for gitsign-credential-cache service

- **systemdSocket()** - Create systemd socket units
  - Parameters: name, description, listenStream, scope
  - Used in git module for gitsign-credential-cache socket

## 2. Conditional Logic

### Platform/Environment Detection
- **conditions** object - Currently imported but not functional
  - conditions.isGnome - Check if GNOME is the desktop environment
  - conditions.isKDE - Check if KDE is the desktop environment
  - conditions.isWayland - Check if running under Wayland
  - conditions.isX11 - Check if running under X11
  - conditions.isLinux - Check if running on Linux
  - conditions.isMacOS - Check if running on macOS
  - conditions.isWindows - Check if running on Windows

### Conditional Execution
- **when()** - Conditional execution of actions based on conditions
  - Example: `when(conditions.isGnome)` to only run actions on GNOME

## 3. Module Features

### Module Dependencies
- **dependsOn()** - Module dependency management
  - Already used in niri module: `.dependsOn("waybar", "swaync")`
  - Needs proper implementation to ensure modules are applied in correct order

### Module Tags
- **tags()** - Module categorization and filtering
  - Already used in several modules for organization
  - Needs implementation for filtering/searching modules

## 4. File Operations

### Advanced File Operations
- **copyFile()** - Currently available but needs:
  - Better permission handling
  - Backup functionality before overwriting
  - Template processing support

- **fileWrite()** - Currently available but needs:
  - Template processing
  - Variable substitution
  - Line-based editing capabilities

## 5. Package Management

### Package Manager Detection
- Automatic detection of available package managers
- Fallback mechanisms when preferred manager not available
- Support for multiple package managers in single action

### Package Groups
- Support for virtual packages/groups
- Better handling of package name variations across distributions

## 6. Environment Variables

### Access to Environment
- **import.meta.env.HOME** - Used in git module
- Need proper environment variable access and validation
- Support for default values and required variables

## 7. Error Handling

### Action Validation
- Pre-flight checks before executing actions
- Rollback capabilities on failure
- Better error messages with context

### Dry Run Mode
- Ability to preview what actions would be taken
- Show diffs for file modifications
- Validate configurations without applying

## 8. User Interaction

### Prompts and Confirmations
- Interactive prompts for user input
- Confirmation dialogs for destructive actions
- Progress indicators for long-running operations

## 9. State Management

### Action State Tracking
- Track which actions have been applied
- Idempotency guarantees
- Undo/rollback functionality

### Module State
- Track module installation status
- Version management for modules
- Update notifications

## 10. Integration Features

### Shell Integration
- Better shell detection and configuration
- Support for multiple shells per user
- Shell-specific configuration handling

### Desktop Environment Integration
- Automatic desktop environment detection
- DE-specific configuration paths
- Theme and appearance management

## Priority Implementation Order

1. **High Priority** (Core functionality):
   - customAction() support
   - conditions and when() conditional execution
   - httpDownload() action
   - Environment variable access

2. **Medium Priority** (System integration):
   - systemdService() and systemdSocket()
   - Module dependencies (dependsOn)
   - Better error handling and validation

3. **Low Priority** (Nice to have):
   - Interactive prompts
   - State management
   - Advanced template processing