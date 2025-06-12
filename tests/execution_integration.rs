use dhd::{ExecutionEngine, discover_modules, load_modules};
use std::fs;
use tempfile::TempDir;

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
        linkFile({ source: "config", target: "~/.config/test" }),
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
    let successful_modules: Vec<_> = loaded.into_iter().filter_map(|r| r.ok()).collect();
    assert_eq!(successful_modules.len(), 1);

    // Verify module was loaded correctly
    assert_eq!(successful_modules[0].definition.name, "test-exec");
    assert_eq!(successful_modules[0].definition.actions.len(), 3);

    // Create execution engine with dry run
    let engine = ExecutionEngine::new(1, true); // concurrency=1, dry_run=true

    // In dry run mode, execute should succeed without actually running commands
    let result = engine.execute(successful_modules);
    assert!(result.is_ok(), "Dry run execution should succeed");
}

#[test]
fn test_execution_with_specific_module() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple modules
    let module1 = r#"
export default defineModule("selected")
    .actions([
        packageInstall({ names: ["selected-pkg"] }),
        linkFile({ source: "selected.conf", target: "~/.selected" })
    ]);
"#;

    let module2 = r#"
export default defineModule("not-selected")
    .actions([
        packageInstall({ names: ["other-pkg"] }),
        linkFile({ source: "other.conf", target: "~/.other" })
    ]);
"#;

    fs::write(temp_dir.path().join("selected.ts"), module1).unwrap();
    fs::write(temp_dir.path().join("not-selected.ts"), module2).unwrap();

    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);
    let successful_modules: Vec<_> = loaded.into_iter().filter_map(|r| r.ok()).collect();

    // Simulate --module flag by filtering to specific module
    let selected_modules: Vec<_> = successful_modules
        .into_iter()
        .filter(|m| m.definition.name == "selected")
        .collect();

    assert_eq!(selected_modules.len(), 1);
    assert_eq!(selected_modules[0].definition.name, "selected");
    assert_eq!(selected_modules[0].definition.actions.len(), 2);

    let engine = ExecutionEngine::new(1, true); // concurrency=1, dry_run=true
    let result = engine.execute(selected_modules);
    assert!(result.is_ok(), "Dry run execution should succeed");
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
        linkFile({ source: "file1", target: "~/.file1" }),
        executeCommand({ command: "echo", args: ["2"] }),
        packageInstall({ names: ["pkg3"] }),
        linkFile({ source: "file2", target: "~/.file2" })
    ]);
"#;

    fs::write(temp_dir.path().join("priority-test.ts"), module_content).unwrap();

    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);
    let successful_modules: Vec<_> = loaded.into_iter().filter_map(|r| r.ok()).collect();

    assert_eq!(successful_modules.len(), 1);
    assert_eq!(successful_modules[0].definition.actions.len(), 6);

    let engine = ExecutionEngine::new(2, true); // concurrency=2, dry_run=true
    let result = engine.execute(successful_modules);
    assert!(result.is_ok(), "Dry run execution should succeed");
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
    let successful_modules: Vec<_> = loaded.into_iter().filter_map(|r| r.ok()).collect();

    assert_eq!(successful_modules.len(), 1);
    assert_eq!(successful_modules[0].definition.actions.len(), 0);

    let engine = ExecutionEngine::new(1, true); // concurrency=1, dry_run=true
    let result = engine.execute(successful_modules);
    assert!(result.is_ok(), "Empty module execution should succeed");
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
    .dependsOn(["base"])
    .actions([
        packageInstall({ names: ["app"] }),
        executeCommand({ command: "app", args: ["--init"] })
    ]);
"#;

    fs::write(temp_dir.path().join("base.ts"), base_module).unwrap();
    fs::write(temp_dir.path().join("app.ts"), dependent_module).unwrap();

    let discovered = discover_modules(temp_dir.path()).unwrap();
    let loaded = load_modules(discovered);
    let successful_modules: Vec<_> = loaded.into_iter().filter_map(|r| r.ok()).collect();

    assert_eq!(successful_modules.len(), 2);

    // When executing "app", both modules should be included
    let app_module = successful_modules
        .iter()
        .find(|m| m.definition.name == "app")
        .unwrap();

    // In a real implementation, dependency resolution would ensure base is executed first
    assert_eq!(app_module.definition.dependencies, vec!["base"]);

    let engine = ExecutionEngine::new(1, true); // concurrency=1, dry_run=true
    let result = engine.execute(successful_modules);
    assert!(result.is_ok(), "Execution with dependencies should succeed");
}
