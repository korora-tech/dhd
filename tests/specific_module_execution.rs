use dhd::{ExecutionEngine, discover_modules, load_modules};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_single_module_execution_flow() {
    let temp_dir = TempDir::new().unwrap();

    // Create a simple module like atuin
    let module_content = r#"
export default defineModule("test-module")
    .description("Test module for execution")
    .actions([
        packageInstall({ names: ["test-tool"] }),
        linkFile({ source: "config.toml", target: "test/config.toml", force: true }),
        executeCommand({ command: "test-tool", args: ["--init"] })
    ]);
"#;

    fs::write(temp_dir.path().join("test-module.ts"), module_content).unwrap();

    // Discovery phase
    let discovered = discover_modules(temp_dir.path()).unwrap();
    println!("Discovered {} modules", discovered.len());

    // Loading phase
    let loaded = load_modules(discovered);
    println!("Loaded {} modules (with results)", loaded.len());

    // Filter successful modules
    let successful_modules: Vec<_> = loaded.into_iter().filter_map(|r| r.ok()).collect();
    println!("Successfully loaded {} modules", successful_modules.len());

    // Print module details
    for module in &successful_modules {
        println!("Module: {}", module.definition.name);
        println!("  Description: {:?}", module.definition.description);
        println!("  Actions: {}", module.definition.actions.len());
        println!("  Tags: {:?}", module.definition.tags);
        println!("  Dependencies: {:?}", module.definition.dependencies);
    }

    // Verify module was loaded correctly
    assert_eq!(successful_modules.len(), 1);
    assert_eq!(successful_modules[0].definition.name, "test-module");
    assert_eq!(successful_modules[0].definition.actions.len(), 3);

    // Create execution engine and run in dry mode
    let engine = ExecutionEngine::new(1, true, false); // concurrency=1, dry_run=true, verbose=false
    let result = engine.execute(successful_modules);

    assert!(result.is_ok(), "Dry run execution should succeed");
}

#[test]
fn test_module_with_specific_name() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple modules
    let module1 = r#"
export default defineModule("target-module")
    .description("Module we want to execute")
    .actions([
        packageInstall({ names: ["target-tool"] }),
        executeCommand({ command: "echo", args: ["Target module"] })
    ]);
"#;

    let module2 = r#"
export default defineModule("other-module")
    .description("Module we don't want to execute")
    .actions([
        packageInstall({ names: ["other-tool"] }),
        executeCommand({ command: "echo", args: ["Other module"] })
    ]);
"#;

    fs::write(temp_dir.path().join("target-module.ts"), module1).unwrap();
    fs::write(temp_dir.path().join("other-module.ts"), module2).unwrap();

    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);
    let all_modules: Vec<_> = loaded.into_iter().filter_map(|r| r.ok()).collect();

    // Simulate module filtering by name (like --module flag)
    let selected_modules: Vec<_> = all_modules
        .into_iter()
        .filter(|m| m.definition.name == "target-module")
        .collect();

    assert_eq!(selected_modules.len(), 1);
    assert_eq!(selected_modules[0].definition.name, "target-module");
    assert_eq!(selected_modules[0].definition.actions.len(), 2);

    // Create execution engine for selected module only
    let engine = ExecutionEngine::new(1, true, false); // concurrency=1, dry_run=true, verbose=false
    let result = engine.execute(selected_modules);
    assert!(result.is_ok(), "Dry run execution should succeed");
}

#[test]
fn test_module_filtering_by_tags() {
    let temp_dir = TempDir::new().unwrap();

    // Create modules with different tags
    let dev_module = r#"
export default defineModule("dev-setup")
    .tags("dev", "tools")
    .actions([
        packageInstall({ names: ["dev-tools"] })
    ]);
"#;

    let prod_module = r#"
export default defineModule("prod-setup")
    .tags("prod", "deployment")
    .actions([
        packageInstall({ names: ["prod-tools"] })
    ]);
"#;

    let mixed_module = r#"
export default defineModule("common-setup")
    .tags("dev", "prod", "common")
    .actions([
        packageInstall({ names: ["common-tools"] })
    ]);
"#;

    fs::write(temp_dir.path().join("dev-setup.ts"), dev_module).unwrap();
    fs::write(temp_dir.path().join("prod-setup.ts"), prod_module).unwrap();
    fs::write(temp_dir.path().join("common-setup.ts"), mixed_module).unwrap();

    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);
    let all_modules: Vec<_> = loaded.into_iter().filter_map(|r| r.ok()).collect();

    // Filter by "dev" tag
    let dev_modules: Vec<_> = all_modules
        .iter()
        .filter(|m| m.definition.tags.contains(&"dev".to_string()))
        .cloned()
        .collect();

    assert_eq!(dev_modules.len(), 2, "Should have 2 modules with 'dev' tag");

    let names: Vec<&str> = dev_modules
        .iter()
        .map(|m| m.definition.name.as_str())
        .collect();
    assert!(names.contains(&"dev-setup"));
    assert!(names.contains(&"common-setup"));

    // Execute filtered modules
    let engine = ExecutionEngine::new(2, true, false); // concurrency=2, dry_run=true, verbose=false
    let result = engine.execute(dev_modules);
    assert!(result.is_ok(), "Dry run execution should succeed");
}

#[test]
fn test_multi_filter_execution() {
    let temp_dir = TempDir::new().unwrap();

    // Create a module that matches multiple criteria
    let module_content = r#"
export default defineModule("special-module")
    .description("A special module for testing")
    .tags("special", "test", "important")
    .actions([
        packageInstall({ names: ["special-tool"] }),
        linkFile({ source: "special.conf", target: "~/.special.conf" }),
        executeCommand({ command: "special-tool", args: ["--setup"] })
    ]);
"#;

    // Create other modules that don't match all criteria
    let other1 = r#"
export default defineModule("regular-module")
    .tags("regular", "test")
    .actions([
        packageInstall({ names: ["regular-tool"] })
    ]);
"#;

    let other2 = r#"
export default defineModule("special-other")
    .tags("special", "other")
    .actions([
        packageInstall({ names: ["other-tool"] })
    ]);
"#;

    fs::write(temp_dir.path().join("special-module.ts"), module_content).unwrap();
    fs::write(temp_dir.path().join("regular-module.ts"), other1).unwrap();
    fs::write(temp_dir.path().join("special-other.ts"), other2).unwrap();

    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);
    let all_modules: Vec<_> = loaded.into_iter().filter_map(|r| r.ok()).collect();

    // Filter by both name pattern and tag
    let selected: Vec<_> = all_modules
        .into_iter()
        .filter(|m| {
            m.definition.name.contains("special") && m.definition.tags.contains(&"test".to_string())
        })
        .collect();

    assert_eq!(
        selected.len(),
        1,
        "Only one module should match both criteria"
    );
    assert_eq!(selected[0].definition.name, "special-module");
    assert_eq!(selected[0].definition.actions.len(), 3);

    let engine = ExecutionEngine::new(1, true, false); // concurrency=1, dry_run=true, verbose=false
    let result = engine.execute(selected);
    assert!(result.is_ok(), "Dry run execution should succeed");
}
