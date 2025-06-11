---
title: Installation
description: How to install DHD on your system
---

# Installation

DHD can be installed in several ways depending on your preferences and system setup.

## Prerequisites

- **Rust** (for building from source)
- **Git** (for cloning the repository)
- **Bun** (optional, for TypeScript type checking in your modules) - install from https://bun.sh

## Installation Methods

### From Source (Recommended)

The most up-to-date way to install DHD is by building from source:

```bash
# Clone the repository
git clone https://github.com/korora-tech/dhd
cd dhd

# Build and install
cargo install --path .
```

This will install the `dhd` binary to your Cargo bin directory (usually `~/.cargo/bin/`).

### Using Cargo

Once DHD is published to crates.io:

```bash
cargo install dhd
```

### Pre-built Binaries

Pre-built binaries for common platforms will be available from the [GitHub Releases](https://github.com/korora-tech/dhd/releases) page.

#### Linux/macOS

```bash
# Download the latest release (example for Linux x64)
curl -LO https://github.com/korora-tech/dhd/releases/latest/download/dhd-linux-x64.tar.gz

# Extract
tar -xzf dhd-linux-x64.tar.gz

# Move to PATH
sudo mv dhd /usr/local/bin/

# Make executable
sudo chmod +x /usr/local/bin/dhd
```

#### Windows

Download the appropriate `.exe` file from the releases page and add it to your PATH.

### Package Managers

Support for popular package managers is planned:

#### Homebrew (macOS/Linux)
```bash
# Coming soon
brew install korora-tech/tap/dhd
```

#### AUR (Arch Linux)
```bash
# Coming soon
yay -S dhd
```

## Verify Installation

After installation, verify DHD is working:

```bash
# Check version
dhd --version

# Show help
dhd --help
```

## Shell Completion

DHD can generate shell completions for better command-line experience:

### Bash
```bash
dhd completion bash > ~/.local/share/bash-completion/completions/dhd
```

### Zsh
```bash
dhd completion zsh > ~/.zfunc/_dhd
```

### Fish
```bash
dhd completion fish > ~/.config/fish/completions/dhd.fish
```

### PowerShell
```powershell
dhd completion powershell | Out-String | Invoke-Expression
```

## TypeScript Support (Optional)

For the best development experience when writing DHD modules, install the TypeScript definitions:

```bash
# In your DHD modules directory
bun init -y
bun add --dev @korora-tech/dhd-types

# Generate type definitions
dhd codegen
```

This enables:
- Autocompletion in your editor
- Type checking for your modules
- IntelliSense documentation

## Updating DHD

To update DHD to the latest version:

### From Source
```bash
cd /path/to/dhd
git pull
cargo install --path . --force
```

### Using Cargo
```bash
cargo install dhd --force
```

## Uninstalling

To remove DHD from your system:

### Cargo Installation
```bash
cargo uninstall dhd
```

### Manual Installation
```bash
sudo rm /usr/local/bin/dhd
```

## Troubleshooting

### Command Not Found

If `dhd` is not found after installation, ensure the installation directory is in your PATH:

```bash
# For Cargo installations
export PATH="$HOME/.cargo/bin:$PATH"

# Add to your shell config file (.bashrc, .zshrc, etc.)
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
```

### Permission Errors

If you encounter permission errors during installation:

1. Avoid using `sudo` with cargo
2. Ensure you have write permissions to the installation directory
3. Consider using `--root` flag to install to a different location

### Build Errors

If building from source fails:

1. Ensure you have the latest stable Rust toolchain:
   ```bash
   rustup update stable
   ```

2. Check for missing system dependencies:
   ```bash
   # Ubuntu/Debian
   sudo apt-get install build-essential pkg-config libssl-dev

   # Fedora
   sudo dnf install gcc pkg-config openssl-devel

   # macOS
   xcode-select --install
   ```

## Next Steps

Now that you have DHD installed:

- Follow the [Quick Start Guide](/getting-started/quick-start/) to create your first module
- Learn about [Core Concepts](/concepts/modules/)
- Explore [Examples](/examples/dotfiles/) for inspiration