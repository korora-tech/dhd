use dhd::{discover_modules, load_modules, ExecutionEngine, ExecutionMode};
use tempfile::TempDir;
use std::fs;

#[test]
fn test_cli_module_filtering_logic() {
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
    
    // Simulate CLI discovery
    let discovered = discover_modules(temp_dir.path()).unwrap();
    assert_eq!(discovered.len(), 1);
    
    // Simulate CLI loading
    let load_results = load_modules(discovered.clone());
    let mut loaded_modules = Vec::new();
    
    for result in load_results {
        if let Ok(loaded) = result {
            loaded_modules.push(loaded);
        }
    }
    
    assert_eq!(loaded_modules.len(), 1);
    
    // Simulate CLI filtering with module name "atuin"
    let module_filters = vec!["atuin".to_string()];
    let tag_filters: Vec<String> = vec![];
    
    let filtered_modules = loaded_modules.into_iter()
        .filter(|module| {
            // Check module name filter
            let name_match = module_filters.is_empty() || 
                module_filters.contains(&module.definition.name);
            
            // Check tag filter
            let tag_match = tag_filters.is_empty() || 
                tag_filters.iter().any(|tag| module.definition.tags.contains(tag));
            
            name_match && tag_match
        })
        .collect::<Vec<_>>();
    
    assert_eq!(filtered_modules.len(), 1, "Should have filtered to 1 module");
    
    let module = &filtered_modules[0];
    assert_eq!(module.definition.name, "atuin");
    assert_eq!(module.definition.actions.len(), 3, "Atuin module should have 3 actions");
    
    // Create execution plan
    let engine = ExecutionEngine::new(ExecutionMode::DryRun);
    let plan = engine.plan(filtered_modules);
    
    println!("Execution plan for atuin module:");
    println!("  Module count: {}", plan.module_count);
    println!("  Action count: {}", plan.action_count);
    println!("  Atom count: {}", plan.atoms.len());
    
    assert_eq!(plan.module_count, 1);
    assert_eq!(plan.action_count, 3);
    assert!(plan.atoms.len() >= 3, "Should have at least 3 atoms");
}

#[test]
fn test_empty_module_filters() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create multiple modules
    let module1 = r#"
export default defineModule("module1")
    .actions([packageInstall({ names: ["tool1"] })]);
"#;
    
    let module2 = r#"
export default defineModule("module2")
    .actions([packageInstall({ names: ["tool2"] })]);
"#;
    
    fs::write(temp_dir.path().join("module1.ts"), module1).unwrap();
    fs::write(temp_dir.path().join("module2.ts"), module2).unwrap();
    
    let discovered = discover_modules(temp_dir.path()).unwrap();
    let load_results = load_modules(discovered);
    let loaded_modules: Vec<_> = load_results.into_iter()
        .filter_map(|r| r.ok())
        .collect();
    
    // Test with empty filters (should select all)
    let module_filters: Vec<String> = vec![];
    let tag_filters: Vec<String> = vec![];
    
    let filtered_modules = loaded_modules.iter()
        .filter(|module| {
            let name_match = module_filters.is_empty() || 
                module_filters.contains(&module.definition.name);
            let tag_match = tag_filters.is_empty() || 
                tag_filters.iter().any(|tag| module.definition.tags.contains(tag));
            name_match && tag_match
        })
        .collect::<Vec<_>>();
    
    assert_eq!(filtered_modules.len(), 2, "Empty filters should select all modules");
}