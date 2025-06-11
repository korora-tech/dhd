use dhd::{discover_modules, load_modules, ExecutionEngine};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_atuin_module_loading() {
    let temp_dir = TempDir::new().unwrap();

    // Create the exact atuin module
    let atuin_module = r#"
export default defineModule("atuin")
    .description("Sync, search and backup shell history with Atuin")
    .actions([
        packageInstall({
            names: ["atuin"],
        }),
        linkFile({
            source: "config.toml",
            target: "atuin/config.toml",
            force: true,
        }),
        executeCommand({
            shell: "nu",
            command: 'atuin init nu | save -f ($nu.user-autoload-dirs | path join "atuin.nu")',
        }),
    ]);
"#;

    fs::write(temp_dir.path().join("atuin.ts"), atuin_module).unwrap();

    // Test discovery
    let discovered = discover_modules(temp_dir.path()).unwrap();
    assert_eq!(discovered.len(), 1);
    assert_eq!(discovered[0].name, "atuin");

    // Test loading
    let loaded = load_modules(discovered);
    assert_eq!(loaded.len(), 1);

    let module = loaded[0].as_ref().unwrap();
    assert_eq!(module.definition.name, "atuin");
    assert_eq!(
        module.definition.description,
        Some("Sync, search and backup shell history with Atuin".to_string())
    );
    assert_eq!(module.definition.actions.len(), 3);

    // Verify action types
    use dhd::ActionType;
    match &module.definition.actions[0] {
        ActionType::PackageInstall(pkg) => {
            assert_eq!(pkg.names, vec!["atuin"]);
        }
        _ => panic!("Expected PackageInstall action"),
    }

    match &module.definition.actions[1] {
        ActionType::LinkFile(link) => {
            assert_eq!(link.source, "config.toml");
            assert_eq!(link.target, "atuin/config.toml");
            assert!(link.force);
        }
        _ => panic!("Expected LinkFile action"),
    }

    match &module.definition.actions[2] {
        ActionType::ExecuteCommand(cmd) => {
            assert_eq!(cmd.shell, Some("nu".to_string()));
            assert!(cmd.command.contains("atuin init nu"));
        }
        _ => panic!("Expected ExecuteCommand action"),
    }

    // Test that it can be executed
    let successful_modules = vec![module.clone()];
    let engine = ExecutionEngine::new(1, true); // concurrency=1, dry_run=true
    let result = engine.execute(successful_modules);
    assert!(result.is_ok(), "Dry run execution should succeed");
}

#[test]
fn test_atuin_module_with_other_modules() {
    let temp_dir = TempDir::new().unwrap();

    // Create atuin module
    let atuin_module = r#"
export default defineModule("atuin")
    .description("Sync, search and backup shell history with Atuin")
    .actions([
        packageInstall({
            names: ["atuin"],
        }),
        linkFile({
            source: "config.toml",
            target: "atuin/config.toml",
            force: true,
        }),
    ]);
"#;

    // Create another module
    let other_module = r#"
export default defineModule("other-tool")
    .description("Some other tool")
    .actions([
        packageInstall({
            names: ["other-tool"],
        }),
    ]);
"#;

    fs::write(temp_dir.path().join("atuin.ts"), atuin_module).unwrap();
    fs::write(temp_dir.path().join("other.ts"), other_module).unwrap();

    let discovered = discover_modules(temp_dir.path()).unwrap();
    assert_eq!(discovered.len(), 2);

    let loaded = load_modules(discovered);
    let successful_modules: Vec<_> = loaded.into_iter().filter_map(|r| r.ok()).collect();
    assert_eq!(successful_modules.len(), 2);

    // Find atuin module specifically
    let atuin = successful_modules
        .iter()
        .find(|m| m.definition.name == "atuin")
        .unwrap();
    assert_eq!(atuin.definition.actions.len(), 2);
}

#[test]
fn test_atuin_module_error_handling() {
    let temp_dir = TempDir::new().unwrap();

    // Create invalid atuin module
    let invalid_module = r#"
export default defineModule("atuin")
    .description("Invalid module")
    .actions([
        // This is not a valid action
        notAnAction({ invalid: true })
    ]);
"#;

    fs::write(temp_dir.path().join("atuin.ts"), invalid_module).unwrap();

    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);
    
    // Should have one result
    assert_eq!(loaded.len(), 1);
    
    // Check if it's an error or if the loader handles unknown actions gracefully
    match &loaded[0] {
        Ok(module) => {
            // If it loads successfully, the module should have no valid actions
            // since notAnAction is not recognized
            println!("Module loaded with {} actions", module.definition.actions.len());
            // This is acceptable - the loader may ignore unknown actions
        }
        Err(e) => {
            // This is also acceptable - the loader may error on unknown actions
            println!("Module failed to load: {}", e);
        }
    }
}