#!/usr/bin/env node

// Example demonstrating tag filtering in DHD

// Run this script with:
// dhd generate types && dhd apply --dry-run --tag productivity

// Desktop productivity apps
export const firefox = defineModule("firefox")
    .description("Web browser")
    .tags("desktop", "browser", "productivity")
    .actions([
        packageInstall({ names: ["firefox"] })
    ]);

export const slack = defineModule("slack")
    .description("Team communication")
    .tags("desktop", "communication", "productivity")
    .actions([
        packageInstall({ names: ["slack"] })
    ]);

// CLI productivity tools
export const ripgrep = defineModule("ripgrep")
    .description("Fast search tool")
    .tags("cli", "productivity", "search")
    .actions([
        packageInstall({ names: ["ripgrep"] })
    ]);

// Development tools (not productivity tagged)
export const rust = defineModule("rust")
    .description("Rust programming language")
    .tags("development", "compiler", "language")
    .actions([
        packageInstall({ names: ["rustup"] })
    ]);

// Example commands:
// dhd list                                    # List all modules with tags
// dhd apply --dry-run --tag productivity      # Apply all productivity tools
// dhd apply --dry-run --tag desktop --tag cli --all-tags  # Apply tools that are BOTH desktop AND cli (none match)
// dhd apply --dry-run --tag desktop --tag productivity --all-tags  # Apply desktop productivity tools