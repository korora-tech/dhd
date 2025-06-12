use dhd::loader::load_module;
use dhd::discovery::DiscoveredModule;
use dhd::actions::{Condition, ComparisonOperator};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use serde_json::json;

fn create_test_module(dir: &std::path::Path, name: &str, content: &str) -> DiscoveredModule {
    let path = dir.join(format!("{}.ts", name));
    fs::write(&path, content).unwrap();
    DiscoveredModule {
        path,
        name: name.to_string(),
    }
}

#[test]
fn test_parse_property_conditions() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
export default defineModule("test")
    .when(property("hardware.fingerprint").isTrue())
    .actions([]);
"#;
    
    let module = create_test_module(temp_dir.path(), "test", content);
    let loaded = load_module(&module).unwrap();
    
    assert!(loaded.definition.when.is_some());
    match loaded.definition.when.as_ref().unwrap() {
        Condition::SystemProperty { path, value, operator } => {
            assert_eq!(path, "hardware.fingerprint");
            assert_eq!(value, &json!(true));
            assert!(matches!(operator, ComparisonOperator::Equals));
        }
        _ => panic!("Wrong condition type"),
    }
}

#[test]
fn test_parse_property_equals() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
export default defineModule("test")
    .when(property("os.distro").equals("ubuntu"))
    .actions([]);
"#;
    
    let module = create_test_module(temp_dir.path(), "test", content);
    let loaded = load_module(&module).unwrap();
    
    assert!(loaded.definition.when.is_some());
    match loaded.definition.when.as_ref().unwrap() {
        Condition::SystemProperty { path, value, operator } => {
            assert_eq!(path, "os.distro");
            assert_eq!(value, &json!("ubuntu"));
            assert!(matches!(operator, ComparisonOperator::Equals));
        }
        _ => panic!("Wrong condition type"),
    }
}

#[test]
fn test_parse_property_contains() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
export default defineModule("test")
    .when(property("gpu.vendor").contains("nvidia"))
    .actions([]);
"#;
    
    let module = create_test_module(temp_dir.path(), "test", content);
    let loaded = load_module(&module).unwrap();
    
    assert!(loaded.definition.when.is_some());
    match loaded.definition.when.as_ref().unwrap() {
        Condition::SystemProperty { path, value, operator } => {
            assert_eq!(path, "gpu.vendor");
            assert_eq!(value, &json!("nvidia"));
            assert!(matches!(operator, ComparisonOperator::Contains));
        }
        _ => panic!("Wrong condition type"),
    }
}

#[test]
fn test_parse_command_conditions() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
export default defineModule("test")
    .when(command("git").exists())
    .actions([]);
"#;
    
    let module = create_test_module(temp_dir.path(), "test", content);
    let loaded = load_module(&module).unwrap();
    
    assert!(loaded.definition.when.is_some());
    match loaded.definition.when.as_ref().unwrap() {
        Condition::CommandExists { command } => {
            assert_eq!(command, "git");
        }
        _ => panic!("Wrong condition type"),
    }
}

#[test]
fn test_parse_command_contains() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
export default defineModule("test")
    .when(command("lsusb").contains("fingerprint", true))
    .actions([]);
"#;
    
    let module = create_test_module(temp_dir.path(), "test", content);
    let loaded = load_module(&module).unwrap();
    
    assert!(loaded.definition.when.is_some());
    match loaded.definition.when.as_ref().unwrap() {
        Condition::CommandSucceeds { command, args } => {
            assert_eq!(command, "lsusb | grep -qi 'fingerprint'");
            assert!(args.is_none());
        }
        _ => panic!("Wrong condition type"),
    }
}

#[test]
fn test_parse_file_exists() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
export default defineModule("test")
    .when(fileExists("/etc/passwd"))
    .actions([]);
"#;
    
    let module = create_test_module(temp_dir.path(), "test", content);
    let loaded = load_module(&module).unwrap();
    
    assert!(loaded.definition.when.is_some());
    match loaded.definition.when.as_ref().unwrap() {
        Condition::FileExists { path } => {
            assert_eq!(path, "/etc/passwd");
        }
        _ => panic!("Wrong condition type"),
    }
}

#[test]
fn test_parse_directory_exists() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
export default defineModule("test")
    .when(directoryExists("/home/user/.config"))
    .actions([]);
"#;
    
    let module = create_test_module(temp_dir.path(), "test", content);
    let loaded = load_module(&module).unwrap();
    
    assert!(loaded.definition.when.is_some());
    match loaded.definition.when.as_ref().unwrap() {
        Condition::DirectoryExists { path } => {
            assert_eq!(path, "/home/user/.config");
        }
        _ => panic!("Wrong condition type"),
    }
}

#[test]
fn test_parse_env_var() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
export default defineModule("test")
    .when(envVar("NODE_ENV", "production"))
    .actions([]);
"#;
    
    let module = create_test_module(temp_dir.path(), "test", content);
    let loaded = load_module(&module).unwrap();
    
    assert!(loaded.definition.when.is_some());
    match loaded.definition.when.as_ref().unwrap() {
        Condition::EnvironmentVariable { name, value } => {
            assert_eq!(name, "NODE_ENV");
            assert_eq!(value.as_ref().unwrap(), "production");
        }
        _ => panic!("Wrong condition type"),
    }
}

#[test]
fn test_parse_logical_operators() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
export default defineModule("test")
    .when(
        or([
            property("os.distro").equals("ubuntu"),
            property("os.distro").equals("fedora")
        ])
    )
    .actions([]);
"#;
    
    let module = create_test_module(temp_dir.path(), "test", content);
    let loaded = load_module(&module).unwrap();
    
    assert!(loaded.definition.when.is_some());
    match loaded.definition.when.as_ref().unwrap() {
        Condition::AnyOf { conditions } => {
            assert_eq!(conditions.len(), 2);
            // Verify both are SystemProperty conditions
            for condition in conditions {
                assert!(matches!(condition, Condition::SystemProperty { .. }));
            }
        }
        _ => panic!("Wrong condition type"),
    }
}

#[test]
fn test_parse_and_operator() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
export default defineModule("test")
    .when(
        and([
            commandExists("apt"),
            property("os.family").equals("debian")
        ])
    )
    .actions([]);
"#;
    
    let module = create_test_module(temp_dir.path(), "test", content);
    let loaded = load_module(&module).unwrap();
    
    assert!(loaded.definition.when.is_some());
    match loaded.definition.when.as_ref().unwrap() {
        Condition::AllOf { conditions } => {
            assert_eq!(conditions.len(), 2);
            // Verify first is CommandExists
            assert!(matches!(&conditions[0], Condition::CommandExists { .. }));
            // Verify second is SystemProperty
            assert!(matches!(&conditions[1], Condition::SystemProperty { .. }));
        }
        _ => panic!("Wrong condition type"),
    }
}

#[test]
fn test_parse_not_operator() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
export default defineModule("test")
    .when(not(commandExists("systemctl")))
    .actions([]);
"#;
    
    let module = create_test_module(temp_dir.path(), "test", content);
    let loaded = load_module(&module).unwrap();
    
    assert!(loaded.definition.when.is_some());
    match loaded.definition.when.as_ref().unwrap() {
        Condition::Not { condition } => {
            assert!(matches!(condition.as_ref(), Condition::CommandExists { .. }));
        }
        _ => panic!("Wrong condition type"),
    }
}

#[test]
fn test_parse_complex_nested_conditions() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
export default defineModule("test")
    .when(
        or([
            property("hardware.fingerprint").isTrue(),
            and([
                command("lsusb").contains("fingerprint", true),
                fileExists("/usr/lib/fprintd")
            ])
        ])
    )
    .actions([]);
"#;
    
    let module = create_test_module(temp_dir.path(), "test", content);
    let loaded = load_module(&module).unwrap();
    
    assert!(loaded.definition.when.is_some());
    match loaded.definition.when.as_ref().unwrap() {
        Condition::AnyOf { conditions } => {
            assert_eq!(conditions.len(), 2);
            // First should be SystemProperty
            assert!(matches!(&conditions[0], Condition::SystemProperty { .. }));
            // Second should be AllOf
            match &conditions[1] {
                Condition::AllOf { conditions: inner } => {
                    assert_eq!(inner.len(), 2);
                    assert!(matches!(&inner[0], Condition::CommandSucceeds { .. }));
                    assert!(matches!(&inner[1], Condition::FileExists { .. }));
                }
                _ => panic!("Expected AllOf condition"),
            }
        }
        _ => panic!("Wrong condition type"),
    }
}

#[test]
fn test_module_without_conditions() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
export default defineModule("test")
    .description("No conditions")
    .actions([]);
"#;
    
    let module = create_test_module(temp_dir.path(), "test", content);
    let loaded = load_module(&module).unwrap();
    
    assert!(loaded.definition.when.is_none());
}