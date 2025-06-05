# DHD Justfile for common development tasks

# Default recipe - show help
default:
    @just --list

# Run tests
test:
    cargo test

# Format code
fmt:
    cargo fmt

# Run linter
lint:
    cargo clippy --all-targets -- -D warnings

# Build the project
build:
    cargo build

# Build release version
build-release:
    cargo build --release

# Run the CLI
run *ARGS:
    cargo run -- {{ARGS}}

# Run the TUI
tui:
    cargo run -- --tui

# Build full app with Tauri
tauri-build:
    bun run tauri build

# Run development server for web interface
dev:
    bun run dev

# Tag a new version (updates all version files)
tag VERSION:
    #!/usr/bin/env bash
    set -euo pipefail
    
    echo "Updating version to {{VERSION}} in all files..."
    
    # Update Cargo.toml (only the package version, not dependencies)
    sed -i '0,/^version = ".*"/s//version = "{{VERSION}}"/' Cargo.toml
    
    # Update package.json
    sed -i 's/"version": ".*"/"version": "{{VERSION}}"/' package.json
    
    # Update types/package.json
    sed -i 's/"version": ".*"/"version": "{{VERSION}}"/' types/package.json
    
    # Update tauri.conf.json
    sed -i 's/"version": ".*"/"version": "{{VERSION}}"/' tauri.conf.json
    
    echo "Version updated to {{VERSION}} in all files"
    echo "Run 'just tag-commit {{VERSION}}' to commit and tag"

# Commit version changes and create tag
tag-commit VERSION:
    git add Cargo.toml package.json types/package.json tauri.conf.json
    git commit -m "chore: bump version to {{VERSION}}"
    git tag {{VERSION}}
    echo "Version {{VERSION}} committed and tagged"
    echo "Run 'git push && git push --tags' to push changes"

# Update version, commit, and push with tag
release VERSION:
    just tag {{VERSION}}
    just tag-commit {{VERSION}}
    git push && git push --tags
    echo "Version {{VERSION}} released!"

# Check all version files
check-versions:
    @echo "Checking versions across all files:"
    @echo -n "Cargo.toml: "
    @grep '^version = ' Cargo.toml | cut -d'"' -f2
    @echo -n "package.json: "
    @grep '"version":' package.json | head -1 | cut -d'"' -f4
    @echo -n "types/package.json: "
    @grep '"version":' types/package.json | head -1 | cut -d'"' -f4
    @echo -n "tauri.conf.json: "
    @grep '"version":' tauri.conf.json | cut -d'"' -f4

# Clean build artifacts
clean:
    cargo clean
    rm -rf dist
    rm -rf target

# Run pre-commit checks (format, lint, test)
pre-commit:
    just fmt
    just lint
    just test
    echo "All pre-commit checks passed!"