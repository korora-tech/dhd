import { packageInstall, platform } from '@dhd';

// Install the latest release of a GitHub project
packageInstall({
  names: platform.select({
    default: ["sigstore/gitsign"]
  }),
  manager: "github"
});

// Install a specific version of a GitHub project
packageInstall({
  names: platform.select({
    default: ["BurntSushi/ripgrep@14.1.0"]
  }),
  manager: "github"
});

// Install multiple GitHub releases
packageInstall({
  names: platform.select({
    default: [
      "junegunn/fzf@0.46.0",
      "sharkdp/bat@v0.24.0",
      "ogham/exa"
    ]
  }),
  manager: "github"
});

// Platform-specific installations
packageInstall({
  names: platform.select({
    linux: ["docker/compose@v2.24.0"],
    macos: ["homebrew/brew@4.2.0"],
    default: ["cli/cli@v2.42.0"]  // GitHub CLI
  }),
  manager: "github"
});

// Alternative: Simple array syntax when platform selection is not needed
packageInstall({
  names: ["owner/repo@version"],
  manager: "github"
});

// Specify custom binary name when it differs from repo name
packageInstall({
  names: ["carapace-sh/carapace-bin:carapace"],  // Install as 'carapace' not 'carapace-bin'
  manager: "github"
});