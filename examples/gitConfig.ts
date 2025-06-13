export default defineModule("gitConfig")
    .description("Configure git settings using nested objects")
    .actions([
        // Set all global git configuration in one clean object
        gitConfig({
            global: {
                user: {
                    name: "John Doe",
                    email: "john.doe@example.com"
                },
                init: {
                    defaultBranch: "main"
                },
                core: {
                    editor: "vim"
                },
                alias: {
                    co: "checkout",
                    br: "branch",
                    ci: "commit",
                    st: "status",
                    unstage: "reset HEAD --",
                    last: "log -1 HEAD"
                },
                // Configure credential helpers (arrays for multi-valued configs)
                credential: {
                    helper: ["", "store", "cache --timeout=3600"]
                },
                // Git LFS configuration
                "filter.lfs": {
                    clean: "git-lfs clean -- %f",
                    smudge: "git-lfs smudge -- %f",
                    process: "git-lfs filter-process",
                    required: "true"
                }
            }
        }),

        // Configure local repository settings (when inside a git repo)
        gitConfig({
            local: {
                remote: {
                    origin: {
                        url: "https://github.com/example/repo.git"
                    }
                },
                branch: {
                    main: {
                        remote: "origin",
                        merge: "refs/heads/main"
                    }
                }
            }
        }),
    ]);