import { defineModule, gitConfig } from "../types";

export default defineModule("git-config")
  .description("Configure Git with common settings")
  .tags("git", "development")
  .with((ctx) => [
    // Set global git configuration
    gitConfig({
      scope: "global",
      configs: {
        "user.name": "John Doe",
        "user.email": "john.doe@example.com",
        "init.defaultBranch": "main",
        "pull.rebase": "true",
        "core.editor": "vim",
        "diff.tool": "vimdiff",
        "merge.tool": "vimdiff",
        "color.ui": "auto",
        "push.autoSetupRemote": "true",
        "rerere.enabled": "true",
      },
    }),

    // Example of setting local repository config
    // This would only work when run inside a git repository
    gitConfig({
      scope: "local",
      configs: {
        "core.filemode": "false",
        "core.ignorecase": "true",
      },
    }),

    // Example with escape hatch for arbitrary config
    gitConfig({
      scope: "global",
      configs: {
        // Aliases
        "alias.st": "status",
        "alias.co": "checkout",
        "alias.br": "branch",
        "alias.ci": "commit",
        "alias.unstage": "reset HEAD --",
        "alias.last": "log -1 HEAD",
        "alias.visual": "!gitk",
        
        // Advanced settings
        "core.autocrlf": "input",
        "core.whitespace": "fix,-indent-with-non-tab,trailing-space,cr-at-eol",
        "branch.autosetuprebase": "always",
        "rebase.autoStash": "true",
        
        // Custom settings (escape hatch for any git config)
        "custom.myConfig": "myValue",
        "includeIf.gitdir:~/work/.path": "~/work/.gitconfig",
      },
    }),
  ]);