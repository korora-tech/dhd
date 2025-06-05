# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Build & Test
- **Build**: `cargo build` (CLI only) or `bun run tauri build` (full CLI+TUI+GUI)
- **Test**: `cargo test` or `cargo test <test_name>` for specific tests
- **Format**: `cargo fmt` (always run before committing)
- **Lint**: `cargo clippy --all-targets -- -D warnings` (always run before committing)
- **Coverage**: `cargo tarpaulin --out xml --all-features --workspace`

### Running DHD
- **CLI**: `cargo run -- [OPTIONS]`
- **TUI**: `cargo run -- --tui`
- **GUI**: First build with `bun run tauri build`, then `./target/release/dhd --gui`
- **Web Dev**: `bun run dev` (hot reload at localhost:3000)

## Architecture Overview

DHD uses a three-layer architecture:

1. **Modules** (TypeScript): User-facing configuration files that define system state declaratively
2. **Actions** (Rust): High-level operations that translate module definitions into executable plans
3. **Atoms** (Rust): Low-level, idempotent operations that perform actual system changes

Key architectural principles:
- Actions plan into one or more Atoms
- Atoms are executed in dependency order using a DAG
- All operations are idempotent (check before execute)
- Platform-specific implementations are isolated in dedicated modules

## Code Organization

- `/src/modules/`: Module loading, parsing (OXC), and registry
- `/src/actions/`: Action trait and implementations (e.g., PackageInstall, ExecuteCommand)
- `/src/atoms/`: Atomic operations (e.g., run_command, symlink, file_write)
- `/src/actions/package/managers/`: Platform-specific package managers
- `/types/`: TypeScript type definitions published to JSR as @dhd/types

## Key Conventions

- Always run `cargo fmt` and `cargo clippy` before committing
- Use `fd` over `find`, `rg` over `grep`
- Prefer editing existing files over creating new ones
- Never create documentation files unless explicitly requested
- Follow existing code patterns in neighboring files
- Use the existing error handling pattern with `DhdError` and `Result<T>`
- Never tag versions with a v prefix

## Testing Approach

- Unit tests live alongside code in `#[cfg(test)]` modules
- Integration tests go in `/tests/` directory
- Always add tests for new Atoms and Actions
- Test both success and failure cases
- Use `tempfile` crate for filesystem tests

## Release Process

To release a new version:
1. Run `just release X.Y.Z` (e.g., `just release 0.2.5`)
2. This will automatically:
   - Update version in `Cargo.toml`, `package.json`, `types/package.json`, and `tauri.conf.json`
   - Commit with message: `chore: bump version to X.Y.Z`
   - Create tag without v prefix: `git tag X.Y.Z`
   - Push commits and tags: `git push && git push --tags`
3. CI will automatically publish to crates.io and JSR

Alternative manual steps:
- `just tag X.Y.Z` - Update versions only
- `just tag-commit X.Y.Z` - Commit and tag only
- `just check-versions` - Check current versions across all files