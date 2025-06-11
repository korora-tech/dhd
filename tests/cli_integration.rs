use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Module Discovery and Execution"));
    assert!(stdout.contains("Usage: dhd"));
    assert!(stdout.contains("Commands:"));
    assert!(stdout.contains("generate"));
    assert!(stdout.contains("list"));
}

#[test]
fn test_generate_types_command() {
    // Clean up any existing types.d.ts
    let types_file = "types.d.ts";
    if Path::new(types_file).exists() {
        fs::remove_file(types_file).ok();
    }

    // Run the generate types command
    let output = Command::new("cargo")
        .args(&["run", "--", "generate", "types"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Check that types.d.ts was created
    assert!(Path::new(types_file).exists());

    // Read and verify the content
    let content = fs::read_to_string(types_file).expect("Failed to read generated types.d.ts");

    // Verify basic structure
    assert!(content.contains("// Generated TypeScript definitions for DHD"));
    assert!(content.contains("declare global"));
    assert!(content.contains("interface ModuleBuilder"));
    assert!(content.contains("function defineModule"));
    assert!(content.contains("type ActionType"));

    // Verify our key types
    assert!(content.contains("ModuleBuilder"));
    assert!(content.contains("ModuleDefinition"));
    assert!(content.contains("PackageInstall"));

    // Verify the file ends with export {} to make it a module
    assert!(content.contains("export {};"));
    assert!(content.contains("LinkFile"));
    assert!(content.contains("ExecuteCommand"));

    // Verify that tsconfig.json was also created
    let tsconfig_file = "tsconfig.json";
    assert!(Path::new(tsconfig_file).exists());

    let tsconfig_content =
        fs::read_to_string(tsconfig_file).expect("Failed to read generated tsconfig.json");
    assert!(tsconfig_content.contains("types.d.ts"));
    assert!(tsconfig_content.contains("**/*.ts"));

    // Verify methods are included
    assert!(content.contains("description(desc: string): this"));
    assert!(content.contains("actions(actions: ActionType[]): ModuleDefinition"));

    // Clean up
    fs::remove_file(types_file).ok();
    fs::remove_file(tsconfig_file).ok();
}

#[test]
fn test_unknown_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "unknown-command"])
        .output()
        .expect("Failed to execute command");

    // Should fail for unknown command
    assert!(!output.status.success());
}

#[test]
fn test_generate_command_without_subcommand() {
    let output = Command::new("cargo")
        .args(&["run", "--", "generate"])
        .output()
        .expect("Failed to execute command");

    // Should show help or error for missing subcommand
    assert!(!output.status.success());
}

#[test]
fn test_list_command_empty_directory() {
    let temp_dir = TempDir::new().unwrap();

    // First build the binary
    Command::new("cargo")
        .args(&["build", "--quiet"])
        .output()
        .expect("Failed to build");

    let binary_path = env!("CARGO_BIN_EXE_dhd");
    let output = Command::new(binary_path)
        .args(&["list"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No TypeScript modules found"));
}

#[test]
fn test_list_command_with_modules() {
    let temp_dir = TempDir::new().unwrap();

    // Create some test TypeScript files with valid module definitions
    std::fs::write(
        temp_dir.path().join("app.ts"),
        r#"export default { name: "app-module", description: "Application module", actions: [] };"#,
    )
    .unwrap();

    std::fs::write(
        temp_dir.path().join("config.ts"),
        r#"export default { name: "config-module", description: "Configuration module", actions: [] };"#
    ).unwrap();

    // First build the binary
    Command::new("cargo")
        .args(&["build", "--quiet"])
        .output()
        .expect("Failed to build");

    let binary_path = env!("CARGO_BIN_EXE_dhd");
    let output = Command::new(binary_path)
        .args(&["list"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Found 2 TypeScript module(s):"));
    assert!(stdout.contains("app-module (app.ts) - Application module"));
    assert!(stdout.contains("config-module (config.ts) - Configuration module"));
}
