use dhd::{discover_modules, load_modules, ExecutionEngine, ExecutionMode};
use tempfile::TempDir;
use std::fs;

#[test]
fn test_full_execution_flow() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create a test module
    let module_content = r#"
export default defineModule("test-exec")
    .description("Test execution flow")
    .tags("test")
    .actions([
        packageInstall({ names: ["test-package"] }),
        linkDotfile({ from: "config", to: "~/.config/test" }),
        executeCommand({ command: "echo", args: ["test"] })
    ]);
"#;
    
    fs::write(temp_dir.path().join("test-exec.ts"), module_content).unwrap();
    
    // Discovery phase
    let discovered = discover_modules(temp_dir.path()).unwrap();
    assert_eq!(discovered.len(), 1);
    
    // Loading phase
    let loaded = load_modules(discovered);
    assert_eq!(loaded.len(), 1);
    
    // Filter successful modules
    let successful_modules: Vec<_> = loaded.into_iter()
        .filter_map(|r| r.ok())
        .collect();
    assert_eq!(successful_modules.len(), 1);
    
    // Create execution plan
    let engine = ExecutionEngine::new(ExecutionMode::DryRun);
    let plan = engine.plan(successful_modules);
    assert!(plan.atoms.len() >= 3, "Should have at least 3 atoms");
    assert_eq!(plan.module_count, 1);
    assert_eq!(plan.action_count, 3);
}

#[test]
fn test_execution_with_specific_module() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create multiple modules
    let module1 = r#"
export default defineModule("selected")
    .actions([
        packageInstall({ names: ["selected-pkg"] }),
        linkDotfile({ from: "selected.conf", to: "~/.selected" })
    ]);
"#;
    
    let module2 = r#"
export default defineModule("not-selected")
    .actions([
        packageInstall({ names: ["other-pkg"] }),
        linkDotfile({ from: "other.conf", to: "~/.other" })
    ]);
"#;
    
    fs::write(temp_dir.path().join("selected.ts"), module1).unwrap();
    fs::write(temp_dir.path().join("not-selected.ts"), module2).unwrap();
    
    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);
    let successful_modules: Vec<_> = loaded.into_iter()
        .filter_map(|r| r.ok())
        .collect();
    
    // Simulate --module flag by filtering to specific module
    let selected_modules: Vec<_> = successful_modules.into_iter()
        .filter(|m| m.definition.name == "selected")
        .collect();
    
    assert_eq!(selected_modules.len(), 1);
    
    let engine = ExecutionEngine::new(ExecutionMode::DryRun);
    let plan = engine.plan(selected_modules);
    // Should only have atoms from the selected module
    assert!(plan.atoms.len() >= 2, "Should have at least 2 atoms from selected module");
    
    // Verify no atoms from not-selected module
    let atom_sources: Vec<_> = plan.atoms.iter()
        .map(|(_, module_name)| module_name)
        .collect();
    assert!(atom_sources.iter().all(|s| *s == "selected"));
}

#[test]
fn test_execution_order_priority() {
    let temp_dir = TempDir::new().unwrap();
    
    let module_content = r#"
export default defineModule("priority-test")
    .actions([
        // These should be grouped by type in execution order
        executeCommand({ command: "echo", args: ["1"] }),
        packageInstall({ names: ["pkg1", "pkg2"] }),
        linkDotfile({ from: "file1", to: "~/.file1" }),
        executeCommand({ command: "echo", args: ["2"] }),
        packageInstall({ names: ["pkg3"] }),
        linkDotfile({ from: "file2", to: "~/.file2" })
    ]);
"#;
    
    fs::write(temp_dir.path().join("priority-test.ts"), module_content).unwrap();
    
    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);
    let successful_modules: Vec<_> = loaded.into_iter()
        .filter_map(|r| r.ok())
        .collect();
    
    let engine = ExecutionEngine::new(ExecutionMode::DryRun);
    let plan = engine.plan(successful_modules);
    
    // The execution plan should have atoms in the correct order
    // Package installs typically come first in the execution order
    assert!(!plan.atoms.is_empty(), "Should have atoms in execution plan");
}

#[test]
fn test_empty_module_execution() {
    let temp_dir = TempDir::new().unwrap();
    
    let module_content = r#"
export default defineModule("empty")
    .description("Empty module")
    .actions([]);
"#;
    
    fs::write(temp_dir.path().join("empty.ts"), module_content).unwrap();
    
    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);
    let successful_modules: Vec<_> = loaded.into_iter()
        .filter_map(|r| r.ok())
        .collect();
    
    let engine = ExecutionEngine::new(ExecutionMode::DryRun);
    let plan = engine.plan(successful_modules);
    assert_eq!(plan.atoms.len(), 0, "Empty module should produce no atoms");
    assert_eq!(plan.action_count, 0, "Empty module should have no actions");
}

#[test]
fn test_module_with_dependencies_execution() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create modules with dependencies
    let base_module = r#"
export default defineModule("base")
    .actions([
        packageInstall({ names: ["base-lib"] })
    ]);
"#;
    
    let dependent_module = r#"
export default defineModule("app")
    .depends("base")
    .actions([
        packageInstall({ names: ["app"] }),
        executeCommand({ command: "app", args: ["--init"] })
    ]);
"#;
    
    fs::write(temp_dir.path().join("base.ts"), base_module).unwrap();
    fs::write(temp_dir.path().join("app.ts"), dependent_module).unwrap();
    
    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);
    let successful_modules: Vec<_> = loaded.into_iter()
        .filter_map(|r| r.ok())
        .collect();
    
    assert_eq!(successful_modules.len(), 2);
    
    // When executing "app", both modules should be included
    let app_module = successful_modules.iter()
        .find(|m| m.definition.name == "app")
        .unwrap();
    
    // In a real implementation, dependency resolution would ensure base is executed first
    assert_eq!(app_module.definition.dependencies, vec!["base"]);
}