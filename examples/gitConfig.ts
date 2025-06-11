export default defineModule("gitConfig")
    .description("Configure git settings programmatically")
    .actions([
        // Set basic user configuration globally
        gitConfig({
            entries: [
                { key: "user.name", value: "John Doe" },
                { key: "user.email", value: "john.doe@example.com" },
                { key: "init.defaultBranch", value: "main" },
                { key: "core.editor", value: "vim" },
            ],
            global: true,
        }),
        // Add multiple credential helpers (multi-valued config)
        gitConfig({
            entries: [
                { key: "credential.helper", value: "", add: true }, // Clear existing
                { key: "credential.helper", value: "store", add: true },
                { key: "credential.helper", value: "cache --timeout=3600", add: true },
            ],
            global: true,
        }),
        // Configure aliases
        gitConfig({
            entries: [
                { key: "alias.co", value: "checkout" },
                { key: "alias.br", value: "branch" },
                { key: "alias.ci", value: "commit" },
                { key: "alias.st", value: "status" },
                { key: "alias.unstage", value: "reset HEAD --" },
                { key: "alias.last", value: "log -1 HEAD" },
            ],
            global: true,
        }),
        // Set local repository-specific configuration
        gitConfig({
            entries: [
                { key: "remote.origin.url", value: "https://github.com/example/repo.git" },
                { key: "branch.main.remote", value: "origin" },
                { key: "branch.main.merge", value: "refs/heads/main" },
            ],
            global: false,
        }),
        // Configure Git LFS
        gitConfig({
            entries: [
                { key: "filter.lfs.clean", value: "git-lfs clean -- %f" },
                { key: "filter.lfs.smudge", value: "git-lfs smudge -- %f" },
                { key: "filter.lfs.process", value: "git-lfs filter-process" },
                { key: "filter.lfs.required", value: "true" },
            ],
            global: true,
        }),
        // Unset a configuration value
        gitConfig({
            entries: [
                { key: "user.signingkey", value: "" },
            ],
            global: true,
            unset: true,
        }),
    ]);