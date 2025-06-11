# Contributing to DHD

Thank you for your interest in contributing to DHD! This guide will help you get started.

## Code of Conduct

By participating in this project, you agree to abide by our Code of Conduct (to be added).

## How to Contribute

### Reporting Issues

- Check if the issue already exists
- Provide a clear description of the problem
- Include steps to reproduce
- Share your environment details (OS, DHD version)

### Suggesting Features

- Open a discussion first for major features
- Explain the use case and benefits
- Consider implementation complexity

### Submitting Code

1. **Fork and Clone**
   ```bash
   git clone https://github.com/your-username/dhd.git
   cd dhd
   ```

2. **Create a Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make Changes**
   - Follow the coding standards
   - Add tests for new functionality
   - Update documentation as needed

4. **Test Your Changes**
   ```bash
   # Run tests
   cargo test --all-features

   # Check formatting
   cargo fmt --all -- --check

   # Run clippy
   cargo clippy --all-targets --all-features -- -D warnings

   # Build
   cargo build --release
   ```

5. **Commit Your Changes**
   ```bash
   git add .
   git commit -m "feat: add new feature"
   ```

   Follow conventional commit format:
   - `feat:` for new features
   - `fix:` for bug fixes
   - `docs:` for documentation
   - `test:` for tests
   - `chore:` for maintenance
   - `refactor:` for refactoring

6. **Push and Create PR**
   ```bash
   git push origin feature/your-feature-name
   ```

## Development Setup

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Git
- Bun (for documentation) - install from https://bun.sh

### Building

```bash
# Build debug version
cargo build

# Build release version
cargo build --release

# Run tests
cargo test

# Run with example modules
cargo run -- list
```

### Documentation

```bash
cd docs
bun install
bun run dev
```

## Project Structure

```
dhd/
├── src/              # Main source code
│   ├── actions/      # High-level actions
│   ├── atoms/        # Low-level operations
│   └── main.rs       # CLI entry point
├── dhd-macros/       # Procedural macros
├── tests/            # Integration tests
├── examples/         # Example modules
└── docs/             # Documentation site
```

## Testing

### Unit Tests

- Place unit tests in the same file as the code
- Use `#[cfg(test)]` module

### Integration Tests

- Add integration tests to `tests/` directory
- Test complete workflows

### Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Your test here
    }
}
```

## Adding New Actions

1. Create action module in `src/actions/`
2. Define the action struct with `#[derive(TypeScriptEnum)]`
3. Implement action logic
4. Add corresponding atom in `src/atoms/`
5. Update `src/actions/mod.rs`
6. Add tests
7. Document the action

Example:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, TypeScriptEnum)]
#[serde(rename_all = "camelCase")]
pub struct MyNewAction {
    pub name: String,
    #[serde(default)]
    pub optional: bool,
}
```

## Documentation Guidelines

- Document all public APIs
- Include examples in doc comments
- Update the documentation site for user-facing changes
- Keep README.md in sync

## Release Process

Releases are automated via GitHub Actions when a tag is pushed:

```bash
# Update version in Cargo.toml files
# Commit changes
git commit -m "chore: bump version to 0.1.0"

# Create and push tag
git tag 0.1.0
git push origin 0.1.0
```

## Getting Help

- Open an issue for bugs
- Start a discussion for questions
- Join our community (Discord/Matrix link to be added)

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT).