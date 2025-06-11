use dhd::{ExecutionEngine, discover_modules, load_modules};
use std::fs;
use tempfile::TempDir;

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
    let loaded_modules: Vec<_> = load_results.into_iter().flatten().collect();

    assert_eq!(loaded_modules.len(), 1);

    // Simulate CLI filtering with module name "atuin"
    let module_filters = ["atuin".to_string()].to_vec();
    let tag_filters: Vec<String> = vec![];

    let filtered_modules = loaded_modules
        .into_iter()
        .filter(|module| {
            // Check module name filter
            let name_match =
                module_filters.is_empty() || module_filters.contains(&module.definition.name);

            // Check tag filter
            let tag_match = tag_filters.is_empty()
                || tag_filters
                    .iter()
                    .any(|tag| module.definition.tags.contains(tag));

            name_match && tag_match
        })
        .collect::<Vec<_>>();

    assert_eq!(
        filtered_modules.len(),
        1,
        "Should have filtered to 1 module"
    );

    let module = &filtered_modules[0];
    assert_eq!(module.definition.name, "atuin");
    assert_eq!(
        module.definition.actions.len(),
        3,
        "Atuin module should have 3 actions"
    );

    // Create execution engine and verify it would execute correctly
    let engine = ExecutionEngine::new(1, true); // concurrency=1, dry_run=true
    let result = engine.execute(filtered_modules);
    assert!(result.is_ok(), "Dry run execution should succeed");
}

#[test]
fn test_empty_module_filters() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple modules
    let module1 = r#"
export default defineModule("module1")
    .tags("dev", "tools")
    .actions([
        packageInstall({ names: ["tool1"] })
    ]);
"#;

    let module2 = r#"
export default defineModule("module2")
    .tags("prod")
    .actions([
        packageInstall({ names: ["tool2"] })
    ]);
"#;

    fs::write(temp_dir.path().join("module1.ts"), module1).unwrap();
    fs::write(temp_dir.path().join("module2.ts"), module2).unwrap();

    let discovered = discover_modules(temp_dir.path()).unwrap();
    let load_results = load_modules(discovered);
    let loaded_modules: Vec<_> = load_results.into_iter().filter_map(|r| r.ok()).collect();

    // With empty filters, all modules should pass
    let module_filters: Vec<String> = vec![];
    let tag_filters: Vec<String> = vec![];

    let filtered_modules = loaded_modules
        .into_iter()
        .filter(|module| {
            let name_match =
                module_filters.is_empty() || module_filters.contains(&module.definition.name);
            let tag_match = tag_filters.is_empty()
                || tag_filters
                    .iter()
                    .any(|tag| module.definition.tags.contains(tag));
            name_match && tag_match
        })
        .collect::<Vec<_>>();

    assert_eq!(filtered_modules.len(), 2, "Should include all modules");
}

#[test]
fn test_tag_filtering() {
    let temp_dir = TempDir::new().unwrap();

    // Create modules with different tags
    let dev_module = r#"
export default defineModule("dev-tools")
    .tags("dev", "tools")
    .actions([
        packageInstall({ names: ["dev-tool"] })
    ]);
"#;

    let prod_module = r#"
export default defineModule("prod-app")
    .tags("prod", "app")
    .actions([
        packageInstall({ names: ["prod-app"] })
    ]);
"#;

    let mixed_module = r#"
export default defineModule("mixed")
    .tags("dev", "prod")
    .actions([
        packageInstall({ names: ["mixed-tool"] })
    ]);
"#;

    fs::write(temp_dir.path().join("dev-tools.ts"), dev_module).unwrap();
    fs::write(temp_dir.path().join("prod-app.ts"), prod_module).unwrap();
    fs::write(temp_dir.path().join("mixed.ts"), mixed_module).unwrap();

    let discovered = discover_modules(temp_dir.path()).unwrap();
    let load_results = load_modules(discovered);
    let loaded_modules: Vec<_> = load_results.into_iter().filter_map(|r| r.ok()).collect();

    // Filter by "dev" tag
    let module_filters: Vec<String> = vec![];
    let tag_filters = ["dev".to_string()].to_vec();

    let filtered_modules = loaded_modules
        .into_iter()
        .filter(|module| {
            let name_match =
                module_filters.is_empty() || module_filters.contains(&module.definition.name);
            let tag_match = tag_filters.is_empty()
                || tag_filters
                    .iter()
                    .any(|tag| module.definition.tags.contains(tag));
            name_match && tag_match
        })
        .collect::<Vec<_>>();

    assert_eq!(
        filtered_modules.len(),
        2,
        "Should include dev and mixed modules"
    );

    let names: Vec<&str> = filtered_modules
        .iter()
        .map(|m| m.definition.name.as_str())
        .collect();
    assert!(names.contains(&"dev-tools"));
    assert!(names.contains(&"mixed"));
}

#[test]
fn test_combined_module_and_tag_filtering() {
    let temp_dir = TempDir::new().unwrap();

    // Create modules
    let module1 = r#"
export default defineModule("app1")
    .tags("web", "frontend")
    .actions([
        packageInstall({ names: ["react"] })
    ]);
"#;

    let module2 = r#"
export default defineModule("app2")
    .tags("backend", "api")
    .actions([
        packageInstall({ names: ["express"] })
    ]);
"#;

    let module3 = r#"
export default defineModule("app3")
    .tags("web", "backend")
    .actions([
        packageInstall({ names: ["nextjs"] })
    ]);
"#;

    fs::write(temp_dir.path().join("app1.ts"), module1).unwrap();
    fs::write(temp_dir.path().join("app2.ts"), module2).unwrap();
    fs::write(temp_dir.path().join("app3.ts"), module3).unwrap();

    let discovered = discover_modules(temp_dir.path()).unwrap();
    let load_results = load_modules(discovered);
    let loaded_modules: Vec<_> = load_results.into_iter().filter_map(|r| r.ok()).collect();

    // Filter by module name "app1" and tag "web" - should only get app1
    let module_filters = ["app1".to_string()].to_vec();
    let tag_filters = ["web".to_string()].to_vec();

    let filtered_modules = loaded_modules
        .into_iter()
        .filter(|module| {
            let name_match =
                module_filters.is_empty() || module_filters.contains(&module.definition.name);
            let tag_match = tag_filters.is_empty()
                || tag_filters
                    .iter()
                    .any(|tag| module.definition.tags.contains(tag));
            name_match && tag_match
        })
        .collect::<Vec<_>>();

    assert_eq!(filtered_modules.len(), 1);
    assert_eq!(filtered_modules[0].definition.name, "app1");
}
