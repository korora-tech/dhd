use dhd::{ExecutionEngine, ExecutionMode, discover_modules, load_modules};
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

    // Create execution plan
    let engine = ExecutionEngine::new(ExecutionMode::DryRun);
    let plan = engine.plan(successful_modules);

    println!("\nExecution plan:");
    println!("  Module count: {}", plan.module_count);
    println!("  Action count: {}", plan.action_count);
    println!("  Atom count: {}", plan.atoms.len());

    // Verify the plan
    assert_eq!(plan.module_count, 1);
    assert_eq!(plan.action_count, 3);
    assert!(plan.atoms.len() >= 3, "Should have at least 3 atoms");
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

    // Create execution plan for selected module only
    let engine = ExecutionEngine::new(ExecutionMode::DryRun);
    let plan = engine.plan(selected_modules);

    assert_eq!(plan.module_count, 1);
    assert_eq!(plan.action_count, 2);
}

#[test]
fn test_module_filtering_edge_cases() {
    let temp_dir = TempDir::new().unwrap();

    // Module with no actions
    let empty_module = r#"
export default defineModule("empty")
    .description("Empty module")
    .actions([]);
"#;

    // Module with tags but no actions
    let tagged_empty = r#"
export default defineModule("tagged-empty")
    .description("Tagged but empty")
    .tags("test", "empty")
    .actions([]);
"#;

    // Module with dependencies
    let dependent_module = r#"
export default defineModule("dependent")
    .description("Module with dependencies")
    .depends("base")
    .actions([
        packageInstall({ names: ["dependent-tool"] })
    ]);
"#;

    fs::write(temp_dir.path().join("empty.ts"), empty_module).unwrap();
    fs::write(temp_dir.path().join("tagged-empty.ts"), tagged_empty).unwrap();
    fs::write(temp_dir.path().join("dependent.ts"), dependent_module).unwrap();

    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);
    let successful_modules: Vec<_> = loaded.into_iter().filter_map(|r| r.ok()).collect();

    assert_eq!(successful_modules.len(), 3);

    // Test filtering each module
    for module in &successful_modules {
        println!("Testing module: {}", module.definition.name);
        let selected = vec![module.clone()];
        let engine = ExecutionEngine::new(ExecutionMode::DryRun);
        let plan = engine.plan(selected);

        match module.definition.name.as_str() {
            "empty" | "tagged-empty" => {
                assert_eq!(plan.atoms.len(), 0, "Empty modules should have no atoms");
                assert_eq!(plan.action_count, 0, "Empty modules should have no actions");
            }
            "dependent" => {
                assert_eq!(
                    plan.action_count, 1,
                    "Dependent module should have 1 action"
                );
                assert!(
                    plan.atoms.len() >= 1,
                    "Dependent module should have at least 1 atom"
                );
            }
            _ => panic!("Unexpected module name"),
        }
    }
}
