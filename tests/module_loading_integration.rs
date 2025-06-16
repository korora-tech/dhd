use dhd::{discover_modules, load_modules};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_load_module_with_tags() {
    let temp_dir = TempDir::new().unwrap();

    let module_content = r#"
export default defineModule("vscode")
    .description("Visual Studio Code - Modern code editor")
    .tags("development", "editor", "ide")
    .actions([
        packageInstall({ names: ["code"] })
    ]);
"#;

    fs::write(temp_dir.path().join("test-app.ts"), module_content).unwrap();

    let discovered = discover_modules(temp_dir.path()).unwrap();
    assert_eq!(discovered.len(), 1);

    let loaded = load_modules(discovered);
    assert_eq!(loaded.len(), 1);

    let module = &loaded[0].as_ref().unwrap().definition;
    assert_eq!(module.name, "vscode");
    assert_eq!(module.description, Some("Visual Studio Code - Modern code editor".to_string()));
    assert_eq!(module.tags, vec!["development", "editor", "ide"]);
}

#[test]
fn test_load_module_with_dependencies() {
    let temp_dir = TempDir::new().unwrap();

    let module_content = r#"
export default defineModule("kubernetes-tools")
    .description("Kubernetes development tools")
    .dependsOn(["docker", "helm", "kubectl"])
    .actions([
        packageInstall({ names: ["kubectx", "kubens", "k9s"] })
    ]);
"#;

    fs::write(temp_dir.path().join("niri.ts"), module_content).unwrap();

    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);

    let module = &loaded[0].as_ref().unwrap().definition;
    assert_eq!(module.name, "kubernetes-tools");
    assert_eq!(module.dependencies, vec!["docker", "helm", "kubectl"]);
}

#[test]
fn test_load_module_with_all_features() {
    let temp_dir = TempDir::new().unwrap();

    let module_content = r#"
export default defineModule("postgresql-dev")
    .description("PostgreSQL development environment with tools")
    .tags("database", "development", "sql")
    .dependsOn(["docker"])
    .actions([
        packageInstall({ 
            names: ["postgresql-client", "pgcli", "pg-top"],
            manager: "apt"
        }),
        executeCommand({
            command: "docker run -d --name postgres-dev -e POSTGRES_PASSWORD=devpass -p 5432:5432 postgres:16",
            args: [],
            escalate: false
        }),
        linkFile({
            source: "pgpass",
            target: "~/.pgpass",
            force: true
        })
    ]);
"#;

    fs::write(temp_dir.path().join("complex-app.ts"), module_content).unwrap();

    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);

    let module = &loaded[0].as_ref().unwrap().definition;
    assert_eq!(module.name, "postgresql-dev");
    assert_eq!(module.tags.len(), 3);
    assert_eq!(module.dependencies, vec!["docker"]);
    assert_eq!(module.actions.len(), 3);
}

#[test]
fn test_module_dependency_ordering() {
    use dhd::{DiscoveredModule, LoadedModule, ModuleDefinition};
    use std::path::PathBuf;

    // Create modules with dependencies
    let modules = vec![
        LoadedModule {
            source: DiscoveredModule {
                path: PathBuf::from("app.ts"),
                name: "app".to_string(),
            },
            definition: ModuleDefinition {
                name: "app".to_string(),
                description: None,
                tags: vec![],
                dependencies: vec!["lib1".to_string(), "lib2".to_string()],
                actions: vec![],
                when: None,
            },
        },
        LoadedModule {
            source: DiscoveredModule {
                path: PathBuf::from("lib1.ts"),
                name: "lib1".to_string(),
            },
            definition: ModuleDefinition {
                name: "lib1".to_string(),
                description: None,
                tags: vec![],
                dependencies: vec!["base".to_string()],
                actions: vec![],
                when: None,
            },
        },
        LoadedModule {
            source: DiscoveredModule {
                path: PathBuf::from("lib2.ts"),
                name: "lib2".to_string(),
            },
            definition: ModuleDefinition {
                name: "lib2".to_string(),
                description: None,
                tags: vec![],
                dependencies: vec!["base".to_string()],
                actions: vec![],
                when: None,
            },
        },
        LoadedModule {
            source: DiscoveredModule {
                path: PathBuf::from("base.ts"),
                name: "base".to_string(),
            },
            definition: ModuleDefinition {
                name: "base".to_string(),
                description: None,
                tags: vec![],
                dependencies: vec![],
                actions: vec![],
                when: None,
            },
        },
    ];

    // TODO: When dependency resolution is implemented, verify the order
    // For now, just verify we can create modules with dependencies
    assert_eq!(modules[0].definition.dependencies.len(), 2);
    assert_eq!(modules[1].definition.dependencies.len(), 1);
    assert_eq!(modules[3].definition.dependencies.len(), 0);
}

#[test]
fn test_load_module_with_method_chaining() {
    let temp_dir = TempDir::new().unwrap();

    // Test the fluent API with method chaining
    let module_content = r#"
import { defineModule, packageInstall } from "dhd";

export default defineModule("rust-dev")
    .description("Rust development environment")
    .tags("rust", "development", "programming")
    .dependsOn(["git", "curl"])
    .tags("systems")
    .actions([
        packageInstall({ names: ["rustup", "cargo-watch", "cargo-edit"] })
    ]);
"#;

    fs::write(temp_dir.path().join("chained.ts"), module_content).unwrap();

    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);

    let module = &loaded[0].as_ref().unwrap().definition;
    assert_eq!(module.name, "rust-dev");
    // Tags should accumulate
    assert_eq!(module.tags.len(), 4);
    assert!(module.tags.contains(&"rust".to_string()));
    assert!(module.tags.contains(&"development".to_string()));
    assert!(module.tags.contains(&"programming".to_string()));
    assert!(module.tags.contains(&"systems".to_string()));
    // Dependencies should accumulate
    assert_eq!(module.dependencies.len(), 2);
    assert!(module.dependencies.contains(&"git".to_string()));
    assert!(module.dependencies.contains(&"curl".to_string()));
}

#[test]
fn test_filter_modules_by_tags() {
    use dhd::{DiscoveredModule, LoadedModule, ModuleDefinition};
    use std::path::PathBuf;

    let modules = vec![
        LoadedModule {
            source: DiscoveredModule {
                path: PathBuf::from("desktop-app.ts"),
                name: "desktop-app".to_string(),
            },
            definition: ModuleDefinition {
                name: "desktop-app".to_string(),
                description: None,
                tags: vec!["desktop".to_string(), "gui".to_string()],
                dependencies: vec![],
                actions: vec![],
                when: None,
            },
        },
        LoadedModule {
            source: DiscoveredModule {
                path: PathBuf::from("cli-tool.ts"),
                name: "cli-tool".to_string(),
            },
            definition: ModuleDefinition {
                name: "cli-tool".to_string(),
                description: None,
                tags: vec!["cli".to_string(), "development".to_string()],
                dependencies: vec![],
                actions: vec![],
                when: None,
            },
        },
        LoadedModule {
            source: DiscoveredModule {
                path: PathBuf::from("dev-tool.ts"),
                name: "dev-tool".to_string(),
            },
            definition: ModuleDefinition {
                name: "dev-tool".to_string(),
                description: None,
                tags: vec!["development".to_string(), "tools".to_string()],
                dependencies: vec![],
                actions: vec![],
                when: None,
            },
        },
    ];

    // Filter modules by tag
    let desktop_modules: Vec<_> = modules
        .iter()
        .filter(|m| m.definition.tags.contains(&"desktop".to_string()))
        .collect();
    assert_eq!(desktop_modules.len(), 1);
    assert_eq!(desktop_modules[0].definition.name, "desktop-app");

    let dev_modules: Vec<_> = modules
        .iter()
        .filter(|m| m.definition.tags.contains(&"development".to_string()))
        .collect();
    assert_eq!(dev_modules.len(), 2);
}

#[test]
fn test_module_action_counting() {
    let temp_dir = TempDir::new().unwrap();

    let module_content = r#"
export default defineModule("nginx-setup")
    .description("Complete NGINX web server setup")
    .actions([
        packageInstall({ names: ["nginx", "certbot", "python3-certbot-nginx"] }),
        linkFile({ source: "nginx.conf", target: "/etc/nginx/nginx.conf" }),
        linkFile({ source: "sites-available/default", target: "/etc/nginx/sites-available/default" }),
        executeCommand({ command: "systemctl", args: ["enable", "nginx"] }),
        executeCommand({ command: "certbot", args: ["--nginx", "-d", "example.com", "--non-interactive", "--agree-tos", "-m", "admin@example.com"] })
    ]);
"#;

    fs::write(temp_dir.path().join("multi-action.ts"), module_content).unwrap();

    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);

    assert_eq!(loaded.len(), 1);
    let module = &loaded[0].as_ref().unwrap().definition;
    assert_eq!(module.actions.len(), 5, "Module should have 5 actions");
}

#[test]
fn test_empty_module_actions() {
    let temp_dir = TempDir::new().unwrap();

    let module_content = r#"
export default defineModule("documentation-only")
    .description("Module for documentation purposes only")
    .tags("docs", "meta")
    .actions([]);
"#;

    fs::write(temp_dir.path().join("empty.ts"), module_content).unwrap();

    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);

    assert_eq!(loaded.len(), 1);
    let module = &loaded[0].as_ref().unwrap().definition;
    assert_eq!(module.actions.len(), 0, "Module should have 0 actions");
}

#[test]
fn test_module_with_all_action_types() {
    let temp_dir = TempDir::new().unwrap();

    let module_content = r#"
export default defineModule("development-environment")
    .description("Complete development environment setup")
    .actions([
        packageInstall({ names: ["git", "vim", "tmux", "zsh"] }),
        linkFile({ source: "gitconfig", target: "~/.gitconfig" }),
        executeCommand({ command: "chsh", args: ["-s", "/usr/bin/zsh"] }),
        directory({ path: "~/.config/nvim" })
    ]);
"#;

    fs::write(temp_dir.path().join("all-actions.ts"), module_content).unwrap();

    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);

    assert_eq!(loaded.len(), 1);
    let module = &loaded[0].as_ref().unwrap().definition;
    assert_eq!(
        module.actions.len(),
        4,
        "Module should have 4 different action types"
    );
}
