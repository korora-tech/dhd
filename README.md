<!-- markdownlint-disable MD041 MD033 -->
# DHD (Declarative Home Deployments)

[![CI](https://github.com/korora-tech/dhd/actions/workflows/ci.yml/badge.svg)](https://github.com/korora-tech/dhd/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/korora-tech/dhd/branch/main/graph/badge.svg)](https://codecov.io/gh/korora-tech/dhd)

DHD is a cross-platform dotfile management system. It lets you define your home configurations declaratively and ensures your development environments are reproducible.

## Prerequisites

- **Rust & Cargo** (>= 1.85)
- **Node.js** & **Bun** (or **npm**/**yarn**)
- **Dagger CLI** (optional, for CI)
- Platform-specific requirements for Tauri desktop apps:
  - **Linux**: GTK3 (e.g., `libgtk-3-dev`), WebKit2 (e.g., `libwebkit2gtk-4.0-dev`)
  - **macOS**: Xcode Command Line Tools (for code signing and bundling)
  - **Windows**: Windows SDK (for MSI packaging)

## Installation

### Building from Source

DHD includes a CLI, TUI, and GUI in a single binary. Always use the Tauri build process to ensure web assets are properly embedded:

```bash
# Install dependencies
bun install

# Build everything (CLI + TUI + GUI)
bun run tauri build
```

The built binary will be at `target/release/dhd` and includes all interfaces.

### TypeScript Module Development

DHD modules are written in TypeScript. The type definitions are published to JSR for easy consumption:

```bash
# Install DHD types from JSR
bunx jsr add @dhd/types

# Or with npm
npx jsr add @dhd/types
```

Then create your module:
```typescript
import { defineModule, packageInstall, executeCommand } from "@dhd/types";

export default defineModule("my-setup")
  .description("My development setup")
  .with((ctx) => [
    packageInstall({ names: ["git", "neovim"] }),
    executeCommand({ command: "git", args: ["config", "--global", "init.defaultBranch", "main"] })
  ]);
```

## Development

### Running the CLI

```bash
cargo run -- [OPTIONS]
```

### Running the TUI

```bash
cargo run -- --tui
# or
cargo run -- tui
```

### Running the GUI

For development, first build the assets:
```bash
bun run tauri build
./target/release/dhd --gui
```

### Web Frontend (Hot Reload)

```bash
bun run dev
```

### Graphical Desktop App (Tauri)

In one terminal, start the web dev server:
```bash
bun run dev
```
In another terminal, launch the Tauri app in dev mode:
```bash
bun run tauri dev
```
This will open the desktop app pointing at `http://localhost:3000`.

## Building for Production

### Build Web Assets

Compile the frontend into static assets in the `dist` directory:
```bash
bun run build
```

### Build for Release

Always use Tauri build for production releases to ensure proper asset embedding:
```bash
bun run tauri build
```

This will:
1. Build the web frontend
2. Embed the assets into the Rust binary
3. Build the CLI with TUI and GUI support
4. Create platform-specific bundles in `target/release/bundle/`
For example:
- macOS: `.app` and `.dmg`
- Linux: `.AppImage`, `.deb`, `.rpm`
- Windows: `.msi`, `.exe`

## Testing

### Rust Unit/Integration Tests

```bash
cargo test
```

## Continuous Integration

To run the full pipeline (Rust tests, web build, Tauri packaging, examples) via Dagger:
```bash
dagger call <task>
```
Alternatively, configure GitHub Actions/GitLab CI to invoke the Dagger pipeline or the individual steps above.


## License

MIT © Korora Tech
