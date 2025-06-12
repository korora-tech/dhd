use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_module_dependencies_are_executed_in_order() {
    let temp_dir = TempDir::new().unwrap();
    let modules_dir = temp_dir.path().join("modules");
    fs::create_dir(&modules_dir).unwrap();
    
    // Create base module
    let base_module = r#"
import { defineModule, executeCommand } from "dhd";

export default defineModule("base-module")
  .description("Base module that should execute first")
  .tags(["test", "base"])
  .actions([
    executeCommand({
      command: "echo",
      arguments: ["[BASE] Base module executed"],
    })
  ]);
"#;
    fs::write(modules_dir.join("base_module.ts"), base_module).unwrap();
    
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
      arguments: ["[DEPENDENT] Dependent module executed"],
    })
  ]);
"#;
    fs::write(modules_dir.join("dependent_module.ts"), dependent_module).unwrap();
    
    // Run dhd apply with only the dependent module selected
    let mut cmd = Command::cargo_bin("dhd").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("apply")
        .arg("--module")
        .arg("dependent-module")
        .arg("--dry-run");
    
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Verify both modules are included
    assert!(stdout.contains("base-module"), "base-module should be included as a dependency");
    assert!(stdout.contains("dependent-module"), "dependent-module should be included");
    
    // Verify the order is correct (base-module should appear before dependent-module in the list)
    let base_pos = stdout.find("base-module").unwrap();
    let dependent_pos = stdout.find("dependent-module").unwrap();
    assert!(base_pos < dependent_pos, "base-module should be listed before dependent-module");
}

#[test]
fn test_missing_dependency_error() {
    let temp_dir = TempDir::new().unwrap();
    let modules_dir = temp_dir.path().join("modules");
    fs::create_dir(&modules_dir).unwrap();
    
    // Create module with missing dependency
    let module_with_missing_dep = r#"
import { defineModule, executeCommand } from "dhd";

export default defineModule("module-with-missing-dep")
  .description("Module with missing dependency")
  .dependsOn(["non-existent-module"])
  .actions([
    executeCommand({
      command: "echo",
      arguments: ["This should not execute"],
    })
  ]);
"#;
    fs::write(modules_dir.join("module_with_missing_dep.ts"), module_with_missing_dep).unwrap();
    
    // Run dhd apply
    let mut cmd = Command::cargo_bin("dhd").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("apply")
        .arg("--module") 
        .arg("module-with-missing-dep");
    
    let output = cmd.output().unwrap();
    assert!(!output.status.success(), "Command should fail due to missing dependency");
    
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("non-existent-module"), "Error message should mention the missing module");
}

#[test]
fn test_circular_dependency_detection() {
    let temp_dir = TempDir::new().unwrap();
    let modules_dir = temp_dir.path().join("modules");
    fs::create_dir(&modules_dir).unwrap();
    
    // Create circular dependency: A -> B -> A
    let module_a = r#"
import { defineModule, executeCommand } from "dhd";

export default defineModule("module-a")
  .dependsOn(["module-b"])
  .actions([
    executeCommand({
      command: "echo",
      arguments: ["Module A"],
    })
  ]);
"#;
    fs::write(modules_dir.join("module_a.ts"), module_a).unwrap();
    
    let module_b = r#"
import { defineModule, executeCommand } from "dhd";

export default defineModule("module-b")
  .dependsOn(["module-a"])
  .actions([
    executeCommand({
      command: "echo",
      arguments: ["Module B"],
    })
  ]);
"#;
    fs::write(modules_dir.join("module_b.ts"), module_b).unwrap();
    
    // Run dhd apply
    let mut cmd = Command::cargo_bin("dhd").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("apply")
        .arg("--module")
        .arg("module-a");
    
    let output = cmd.output().unwrap();
    assert!(!output.status.success(), "Command should fail due to circular dependency");
    
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Cyclic dependency") || stderr.contains("circular"), 
            "Error message should mention circular/cyclic dependency");
}