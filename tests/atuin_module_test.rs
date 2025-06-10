use dhd::{discover_modules, load_modules, ExecutionEngine, ExecutionMode};
use tempfile::TempDir;
use std::fs;

#[test]
fn test_atuin_module_loads_correctly() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create the exact atuin module content
    let module_content = r#"
export default defineModule("atuin")
    .description("Sync, search and backup shell history with Atuin")
    .actions([
        packageInstall({
            names: ["atuin"],
        }),
        linkDotfile({
            from: "config.toml",
            to: "atuin/config.toml",
            force: true,
        }),
        executeCommand({
            shell: "nu",
            command: 'atuin init nu | save -f ($nu.user-autoload-dirs | path join "atuin.nu")',
        }),
    ]);
"#;
    
    fs::write(temp_dir.path().join("atuin.ts"), module_content).unwrap();
    
    let discovered = discover_modules(temp_dir.path()).unwrap();
    assert_eq!(discovered.len(), 1, "Should discover 1 module");
    
    let loaded = load_modules(discovered);
    assert_eq!(loaded.len(), 1, "Should load 1 module");
    
    let module = &loaded[0].as_ref().unwrap().definition;
    assert_eq!(module.name, "atuin");
    assert_eq!(module.description, Some("Sync, search and backup shell history with Atuin".to_string()));
    assert_eq!(module.actions.len(), 3, "Atuin module should have 3 actions");
}

#[test]
fn test_module_atom_flattening() {
    let temp_dir = TempDir::new().unwrap();
    
    let module_content = r#"
export default defineModule("test-flatten")
    .description("Test atom flattening")
    .actions([
        packageInstall({ names: ["pkg1", "pkg2", "pkg3"] }),
        linkDotfile({ from: "config1", to: "~/.config1" }),
        linkDotfile({ from: "config2", to: "~/.config2" }),
    ]);
"#;
    
    fs::write(temp_dir.path().join("test-flatten.ts"), module_content).unwrap();
    
    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);
    let successful_modules: Vec<_> = loaded.into_iter()
        .filter_map(|r| r.ok())
        .collect();
    
    // Create execution plan to count atoms
    let engine = ExecutionEngine::new(ExecutionMode::DryRun);
    let plan = engine.plan(successful_modules);
    assert!(plan.atoms.len() >= 3, "Should have at least 3 atoms after flattening");
}

#[test]
fn test_tag_filtering_with_no_tags() {
    let temp_dir = TempDir::new().unwrap();
    
    // Module without tags
    let module_content = r#"
export default defineModule("no-tags")
    .description("Module without tags")
    .actions([
        packageInstall({ names: ["tool"] })
    ]);
"#;
    
    fs::write(temp_dir.path().join("no-tags.ts"), module_content).unwrap();
    
    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);
    let successful_modules: Vec<_> = loaded.iter()
        .filter_map(|r| r.as_ref().ok())
        .collect();
    
    // When no tag filter is provided, all modules should be included
    assert_eq!(successful_modules.len(), 1, "Module with no tags should be included when no filter is applied");
}

#[test]
fn test_tag_filtering_with_specific_tag() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create multiple modules with different tags
    let cli_module = r#"
export default defineModule("cli-tool")
    .tags("cli", "development")
    .actions([packageInstall({ names: ["cli-tool"] })]);
"#;
    
    let desktop_module = r#"
export default defineModule("desktop-app")
    .tags("desktop", "gui")
    .actions([packageInstall({ names: ["desktop-app"] })]);
"#;
    
    let dev_module = r#"
export default defineModule("dev-tool")
    .tags("development", "tools")
    .actions([packageInstall({ names: ["dev-tool"] })]);
"#;
    
    fs::write(temp_dir.path().join("cli-tool.ts"), cli_module).unwrap();
    fs::write(temp_dir.path().join("desktop-app.ts"), desktop_module).unwrap();
    fs::write(temp_dir.path().join("dev-tool.ts"), dev_module).unwrap();
    
    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);
    let successful_modules: Vec<_> = loaded.iter()
        .filter_map(|r| r.as_ref().ok())
        .collect();
    
    // Filter by "development" tag
    let dev_filtered: Vec<_> = successful_modules.iter()
        .filter(|m| m.definition.tags.contains(&"development".to_string()))
        .collect();
    assert_eq!(dev_filtered.len(), 2, "Should have 2 modules with 'development' tag");
    
    // Filter by "desktop" tag
    let desktop_filtered: Vec<_> = successful_modules.iter()
        .filter(|m| m.definition.tags.contains(&"desktop".to_string()))
        .collect();
    assert_eq!(desktop_filtered.len(), 1, "Should have 1 module with 'desktop' tag");
    
    // Filter by non-existent tag
    let none_filtered: Vec<_> = successful_modules.iter()
        .filter(|m| m.definition.tags.contains(&"nonexistent".to_string()))
        .collect();
    assert_eq!(none_filtered.len(), 0, "Should have 0 modules with non-existent tag");
}

#[test]
fn test_module_selection_with_explicit_module() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create multiple modules
    let module1 = r#"
export default defineModule("module1")
    .actions([packageInstall({ names: ["pkg1"] })]);
"#;
    
    let module2 = r#"
export default defineModule("module2")
    .actions([packageInstall({ names: ["pkg2"] })]);
"#;
    
    fs::write(temp_dir.path().join("module1.ts"), module1).unwrap();
    fs::write(temp_dir.path().join("module2.ts"), module2).unwrap();
    
    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);
    let successful_modules: Vec<_> = loaded.iter()
        .filter_map(|r| r.as_ref().ok())
        .collect();
    
    // Simulate selecting only module1
    let selected: Vec<_> = successful_modules.iter()
        .filter(|m| m.definition.name == "module1")
        .collect();
    
    assert_eq!(selected.len(), 1, "Should select only the specified module");
    assert_eq!(selected[0].definition.name, "module1");
}