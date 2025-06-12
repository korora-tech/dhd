use dhd::dependency_resolver::resolve_dependencies;
use dhd::discovery::discover_modules;
use dhd::loader::load_modules;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_module_dependencies_are_resolved() {
    // Set up test environment
    let temp_dir = TempDir::new().unwrap();
    
    // Create base module
    let base_module = r#"
import { defineModule, executeCommand } from "dhd";

export default defineModule("base-module")
  .description("Base module that should execute first")
  .tags(["test", "base"])
  .actions([
    executeCommand({
      command: "echo",
      arguments: ["Base module executed"],
    })
  ]);
"#;
    fs::write(temp_dir.path().join("base_module.ts"), base_module).unwrap();
    
    // Create dependent module
    let dependent_module = r#"
import { defineModule, executeCommand } from "dhd";

export default defineModule("dependent-module")
  .description("Module that depends on base-module")
  .tags(["test", "dependent"])
  .dependsOn(["base-module"])
  .actions([
    executeCommand({
      command: "echo",
      arguments: ["Dependent module executed"],
    })
  ]);
"#;
    fs::write(temp_dir.path().join("dependent_module.ts"), dependent_module).unwrap();
    
    // Discover modules in test directory
    let discovered = discover_modules(temp_dir.path()).expect("Failed to discover modules");
    assert_eq!(discovered.len(), 2, "Should discover 2 test modules");
    
    // Load modules
    let load_results = load_modules(discovered);
    let loaded: Vec<_> = load_results.into_iter()
        .filter_map(|r| r.ok())
        .collect();
    assert_eq!(loaded.len(), 2, "Should load 2 test modules");
    
    // Find our test modules
    let base_module = loaded.iter().find(|m| m.definition.name == "base-module").expect("base-module not found");
    let dependent_module = loaded.iter().find(|m| m.definition.name == "dependent-module").expect("dependent-module not found");
    
    // Verify dependencies are parsed correctly
    assert_eq!(base_module.definition.dependencies.len(), 0, "base-module should have no dependencies");
    assert_eq!(dependent_module.definition.dependencies.len(), 1, "dependent-module should have 1 dependency");
    assert_eq!(dependent_module.definition.dependencies[0], "base-module", "dependent-module should depend on base-module");
    
    // Test dependency resolution
    let resolved = resolve_dependencies(loaded.clone()).expect("Failed to resolve dependencies");
    assert_eq!(resolved.len(), 2, "Should resolve both modules");
    
    // Verify correct order: base-module should come before dependent-module
    let base_index = resolved.iter().position(|m| m.definition.name == "base-module").expect("base-module not in resolved list");
    let dependent_index = resolved.iter().position(|m| m.definition.name == "dependent-module").expect("dependent-module not in resolved list");
    assert!(base_index < dependent_index, "base-module should be executed before dependent-module");
}

#[test]
fn test_selecting_dependent_module_includes_dependencies() {
    // This test simulates what should happen when a user runs:
    // dhd apply --modules dependent-module
    
    let temp_dir = TempDir::new().unwrap();
    
    // Create the same test modules
    let base_module = r#"
import { defineModule, executeCommand } from "dhd";

export default defineModule("base-module")
  .description("Base module that should execute first")
  .tags(["test", "base"])
  .actions([
    executeCommand({
      command: "echo",
      arguments: ["Base module executed"],
    })
  ]);
"#;
    fs::write(temp_dir.path().join("base_module.ts"), base_module).unwrap();
    
    let dependent_module = r#"
import { defineModule, executeCommand } from "dhd";

export default defineModule("dependent-module")
  .description("Module that depends on base-module")
  .tags(["test", "dependent"])
  .dependsOn(["base-module"])
  .actions([
    executeCommand({
      command: "echo",
      arguments: ["Dependent module executed"],
    })
  ]);
"#;
    fs::write(temp_dir.path().join("dependent_module.ts"), dependent_module).unwrap();
    
    let discovered = discover_modules(temp_dir.path()).expect("Failed to discover modules");
    let load_results = load_modules(discovered);
    let loaded: Vec<_> = load_results.into_iter()
        .filter_map(|r| r.ok())
        .collect();
    
    // Simulate user selecting only "dependent-module"
    let selected_modules = vec!["dependent-module".to_string()];
    
    // What currently happens (incorrect):
    let currently_filtered: Vec<_> = loaded.iter()
        .filter(|m| selected_modules.contains(&m.definition.name))
        .cloned()
        .collect();
    assert_eq!(currently_filtered.len(), 1, "Current implementation only includes the selected module");
    
    // What should happen (correct):
    // When selecting a module with dependencies, all dependencies should be included
    let mut modules_to_execute = currently_filtered.clone();
    let mut added_deps = true;
    
    while added_deps {
        added_deps = false;
        let current_names: Vec<String> = modules_to_execute.iter()
            .map(|m| m.definition.name.clone())
            .collect();
        
        for module in &modules_to_execute.clone() {
            for dep in &module.definition.dependencies {
                if !current_names.contains(dep) {
                    if let Some(dep_module) = loaded.iter().find(|m| &m.definition.name == dep) {
                        modules_to_execute.push(dep_module.clone());
                        added_deps = true;
                    }
                }
            }
        }
    }
    
    assert_eq!(modules_to_execute.len(), 2, "Should include both the selected module and its dependencies");
    
    // Resolve dependencies for correct execution order
    let resolved = resolve_dependencies(modules_to_execute).expect("Failed to resolve dependencies");
    assert_eq!(resolved[0].definition.name, "base-module", "base-module should execute first");
    assert_eq!(resolved[1].definition.name, "dependent-module", "dependent-module should execute second");
}

#[test]
fn test_missing_dependency_error() {
    use dhd::loader::LoadedModule;
    use dhd::discovery::DiscoveredModule;
    use dhd::module::ModuleDefinition;
    use dhd::dependency_resolver::DependencyError;
    use std::path::PathBuf;
    
    let modules = vec![
        LoadedModule {
            source: DiscoveredModule {
                path: PathBuf::from("module_with_missing_dep.ts"),
                name: "module-with-missing-dep".to_string(),
            },
            definition: ModuleDefinition {
                name: "module-with-missing-dep".to_string(),
                description: None,
                tags: vec![],
                dependencies: vec!["non-existent-module".to_string()],
                when: None,
                actions: vec![],
            },
        }
    ];
    
    let result = resolve_dependencies(modules);
    assert!(result.is_err(), "Should fail with missing dependency");
    
    match result.unwrap_err() {
        DependencyError::MissingDependency { module, dependency } => {
            assert_eq!(module, "module-with-missing-dep");
            assert_eq!(dependency, "non-existent-module");
        }
        _ => panic!("Expected MissingDependency error"),
    }
}