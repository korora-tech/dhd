use dhd::{discover_modules, load_modules};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_load_module_with_tags() {
    let temp_dir = TempDir::new().unwrap();

    let module_content = r#"
export default defineModule("test-app")
    .description("A test application")
    .tags("desktop", "productivity", "test")
    .actions([
        packageInstall({ names: ["test-app"] })
    ]);
"#;

    fs::write(temp_dir.path().join("test-app.ts"), module_content).unwrap();

    let discovered = discover_modules(temp_dir.path()).unwrap();
    assert_eq!(discovered.len(), 1);

    let loaded = load_modules(discovered);
    assert_eq!(loaded.len(), 1);

    let module = &loaded[0].as_ref().unwrap().definition;
    assert_eq!(module.name, "test-app");
    assert_eq!(module.description, Some("A test application".to_string()));
    assert_eq!(module.tags, vec!["desktop", "productivity", "test"]);
}

#[test]
fn test_load_module_with_dependencies() {
    let temp_dir = TempDir::new().unwrap();

    let module_content = r#"
export default defineModule("niri")
    .description("Window manager")
    .dependsOn(["waybar", "swaync", "fuzzel"])
    .actions([
        packageInstall({ names: ["niri"] })
    ]);
"#;

    fs::write(temp_dir.path().join("niri.ts"), module_content).unwrap();

    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);

    let module = &loaded[0].as_ref().unwrap().definition;
    assert_eq!(module.name, "niri");
    assert_eq!(module.dependencies, vec!["waybar", "swaync", "fuzzel"]);
}

#[test]
fn test_load_module_with_all_features() {
    let temp_dir = TempDir::new().unwrap();

    let module_content = r#"
export default defineModule("complex-app")
    .description("A complex application with all features")
    .tags("development", "tools")
    .dependsOn(["base-lib"])
    .actions([
        packageInstall({ 
            names: ["complex-app", "complex-app-plugins"],
            manager: "bun"
        }),
        executeCommand({
            command: "complex-app",
            args: ["--init", "--config", "/etc/complex-app.conf"],
            escalate: true
        }),
        linkFile({
            source: "complex-app.conf",
            target: "~/.config/complex-app/config.conf",
            force: true
        })
    ]);
"#;

    fs::write(temp_dir.path().join("complex-app.ts"), module_content).unwrap();

    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);

    let module = &loaded[0].as_ref().unwrap().definition;
    assert_eq!(module.name, "complex-app");
    assert_eq!(module.tags.len(), 2);
    assert_eq!(module.dependencies, vec!["base-lib"]);
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

export default defineModule("chained")
    .description("Test method chaining")
    .tags("test", "example")
    .dependsOn(["dep1", "dep2"])
    .tags("additional")
    .actions([
        packageInstall({ names: ["pkg1"] })
    ]);
"#;

    fs::write(temp_dir.path().join("chained.ts"), module_content).unwrap();

    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);

    let module = &loaded[0].as_ref().unwrap().definition;
    assert_eq!(module.name, "chained");
    // Tags should accumulate
    assert_eq!(module.tags.len(), 3);
    assert!(module.tags.contains(&"test".to_string()));
    assert!(module.tags.contains(&"example".to_string()));
    assert!(module.tags.contains(&"additional".to_string()));
    // Dependencies should accumulate
    assert_eq!(module.dependencies.len(), 2);
    assert!(module.dependencies.contains(&"dep1".to_string()));
    assert!(module.dependencies.contains(&"dep2".to_string()));
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
export default defineModule("multi-action")
    .description("Module with multiple actions")
    .actions([
        packageInstall({ names: ["tool1", "tool2"] }),
        linkFile({ source: "config1.toml", target: "~/.config/tool1/config.toml" }),
        linkFile({ source: "config2.toml", target: "~/.config/tool2/config.toml" }),
        executeCommand({ command: "tool1", args: ["--init"] }),
        executeCommand({ command: "tool2", args: ["--setup"] })
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
export default defineModule("empty")
    .description("Module with no actions")
    .tags("test")
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
export default defineModule("all-actions")
    .description("Module demonstrating all action types")
    .actions([
        packageInstall({ names: ["package1"] }),
        linkFile({ source: "dotfile", target: "~/.dotfile" }),
        executeCommand({ command: "echo", args: ["test"] }),
        directory({ path: "/test/dir" })
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
