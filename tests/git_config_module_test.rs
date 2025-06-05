use dhd::modules::loader::ModuleLoader;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_parse_git_config_module() {
    let dir = tempdir().expect("Failed to create temp dir");
    let module_path = dir.path().join("test-git.ts");

    let module_content = r#"
import { defineModule, gitConfig } from "../types";

export default defineModule("test-git")
  .description("Test git configuration")
  .tags("git", "test")
  .with((ctx) => [
    gitConfig({
      scope: "global",
      configs: {
        "user.name": "Test User",
        "user.email": "test@example.com",
      },
    }),
    gitConfig({
      scope: "local",
      configs: {
        "core.filemode": "false",
      },
    }),
  ]);
"#;

    fs::write(&module_path, module_content).expect("Failed to write test module");

    let mut loader = ModuleLoader::new();
    let module_data = loader
        .load_module(&module_path)
        .expect("Failed to load module");

    assert_eq!(module_data.id, "test-git");
    assert_eq!(
        module_data.description,
        Some("Test git configuration".to_string())
    );
    assert_eq!(module_data.tags, vec!["git", "test"]);
    assert_eq!(module_data.actions.len(), 2);

    // Check first action
    let first_action = &module_data.actions[0];
    assert_eq!(first_action.action_type, "gitConfig");
    assert!(
        first_action
            .params
            .iter()
            .any(|(k, v)| k == "scope" && v == "global")
    );
    assert!(
        first_action
            .params
            .iter()
            .any(|(k, v)| k == "configs" && v.contains("user.name=Test User"))
    );

    // Check second action
    let second_action = &module_data.actions[1];
    assert_eq!(second_action.action_type, "gitConfig");
    assert!(
        second_action
            .params
            .iter()
            .any(|(k, v)| k == "scope" && v == "local")
    );
    assert!(
        second_action
            .params
            .iter()
            .any(|(k, v)| k == "configs" && v.contains("core.filemode=false"))
    );
}
