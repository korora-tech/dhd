import { defineModule, packageInstall } from "../types";

// Example module demonstrating platform-specific package installation
export default defineModule("platform-example")
  .description("Example of platform-specific package installation")
  .tags("example", "packages")
  .actions([
    // Install development tools with platform-specific package names
    packageInstall({
      names: {
        linux: {
          ubuntu: ["build-essential", "git", "curl"],
          debian: ["build-essential", "git", "curl"],
          fedora: ["@development-tools", "git", "curl"],
          arch: ["base-devel", "git", "curl"],
        },
        mac: ["git", "curl"],
        windows: ["Git.Git", "curl"],
      },
    }),
    
    // Install text editor with auto-detected package manager
    packageInstall({
      names: {
        all: ["neovim"],
      },
    }),
    
    // Install Node.js packages globally
    packageInstall({
      names: {
        all: ["typescript", "@biomejs/biome", "tsx"],
      },
      manager: "npm",
    }),
  ]);