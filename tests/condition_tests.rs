use dhd::actions::{Condition, ComparisonOperator, condition::*};
use serde_json::json;
use std::env;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_file_exists_condition() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    
    // File doesn't exist yet
    let condition = file_exists(file_path.to_string_lossy().to_string());
    assert_eq!(condition.evaluate().unwrap(), false);
    
    // Create file
    fs::write(&file_path, "test content").unwrap();
    
    // Now it should exist
    let condition = file_exists(file_path.to_string_lossy().to_string());
    assert_eq!(condition.evaluate().unwrap(), true);
}

#[test]
fn test_directory_exists_condition() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().join("testdir");
    
    // Directory doesn't exist yet
    let condition = directory_exists(dir_path.to_string_lossy().to_string());
    assert_eq!(condition.evaluate().unwrap(), false);
    
    // Create directory
    fs::create_dir(&dir_path).unwrap();
    
    // Now it should exist
    let condition = directory_exists(dir_path.to_string_lossy().to_string());
    assert_eq!(condition.evaluate().unwrap(), true);
}

#[test]
fn test_command_exists_condition() {
    // Test with a command that should exist on all systems
    let condition = command_exists("sh".to_string());
    assert_eq!(condition.evaluate().unwrap(), true);
    
    // Test with a command that shouldn't exist
    let condition = command_exists("this_command_definitely_does_not_exist_12345".to_string());
    assert_eq!(condition.evaluate().unwrap(), false);
}

#[test]
fn test_command_succeeds_condition() {
    // Test with a command that should succeed
    let condition = command_succeeds("true".to_string(), None);
    assert_eq!(condition.evaluate().unwrap(), true);
    
    // Test with a command that should fail
    let condition = command_succeeds("false".to_string(), None);
    assert_eq!(condition.evaluate().unwrap(), false);
    
    // Test with arguments
    let condition = command_succeeds("echo".to_string(), Some(vec!["hello".to_string()]));
    assert_eq!(condition.evaluate().unwrap(), true);
}

#[test]
fn test_environment_variable_condition() {
    // Set a test environment variable
    unsafe {
        env::set_var("TEST_DHD_VAR", "test_value");
    }
    
    // Test variable exists
    let condition = env_var("TEST_DHD_VAR".to_string(), None);
    assert_eq!(condition.evaluate().unwrap(), true);
    
    // Test variable has specific value
    let condition = env_var("TEST_DHD_VAR".to_string(), Some("test_value".to_string()));
    assert_eq!(condition.evaluate().unwrap(), true);
    
    // Test variable has wrong value
    let condition = env_var("TEST_DHD_VAR".to_string(), Some("wrong_value".to_string()));
    assert_eq!(condition.evaluate().unwrap(), false);
    
    // Test non-existent variable
    let condition = env_var("NON_EXISTENT_VAR_12345".to_string(), None);
    assert_eq!(condition.evaluate().unwrap(), false);
    
    // Clean up
    unsafe {
        env::remove_var("TEST_DHD_VAR");
    }
}

#[test]
fn test_system_property_condition() {
    // Test with actual system properties
    let info = dhd::system_info::get_system_info();
    
    // Test string equality
    let condition = Condition::SystemProperty {
        path: "os.distro".to_string(),
        value: json!(info.os.distro),
        operator: ComparisonOperator::Equals,
    };
    assert_eq!(condition.evaluate().unwrap(), true);
    
    // Test boolean property
    let condition = property("hardware.fingerprint".to_string()).is_true();
    let result = condition.evaluate().unwrap();
    // Result depends on actual hardware, just ensure it evaluates without error
    assert!(result == true || result == false);
}

#[test]
fn test_logical_operators() {
    // Test AND - all must be true
    let condition = all_of(vec![
        command_succeeds("true".to_string(), None),
        command_succeeds("true".to_string(), None),
    ]);
    assert_eq!(condition.evaluate().unwrap(), true);
    
    let condition = all_of(vec![
        command_succeeds("true".to_string(), None),
        command_succeeds("false".to_string(), None),
    ]);
    assert_eq!(condition.evaluate().unwrap(), false);
    
    // Test OR - at least one must be true
    let condition = any_of(vec![
        command_succeeds("false".to_string(), None),
        command_succeeds("true".to_string(), None),
    ]);
    assert_eq!(condition.evaluate().unwrap(), true);
    
    let condition = any_of(vec![
        command_succeeds("false".to_string(), None),
        command_succeeds("false".to_string(), None),
    ]);
    assert_eq!(condition.evaluate().unwrap(), false);
    
    // Test NOT
    let condition = not(command_succeeds("false".to_string(), None));
    assert_eq!(condition.evaluate().unwrap(), true);
    
    let condition = not(command_succeeds("true".to_string(), None));
    assert_eq!(condition.evaluate().unwrap(), false);
}

#[test]
fn test_complex_conditions() {
    // Test nested conditions
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "test").unwrap();
    
    let condition = or(vec![
        and(vec![
            file_exists(file_path.to_string_lossy().to_string()),
            command_exists("sh".to_string()),
        ]),
        env_var("IMPOSSIBLE_VAR_12345".to_string(), None),
    ]);
    assert_eq!(condition.evaluate().unwrap(), true);
}

#[test]
fn test_condition_descriptions() {
    let condition = file_exists("/test/path".to_string());
    assert_eq!(condition.describe(), "file exists: /test/path");
    
    let condition = command_exists("git".to_string());
    assert_eq!(condition.describe(), "command exists: git");
    
    let condition = env_var("HOME".to_string(), Some("/home/user".to_string()));
    assert_eq!(condition.describe(), "env var HOME=/home/user");
    
    let condition = property("os.distro".to_string()).equals(json!("ubuntu"));
    assert_eq!(condition.describe(), "os.distro == \"ubuntu\"");
    
    let condition = not(command_exists("git".to_string()));
    assert_eq!(condition.describe(), "not: command exists: git");
}

#[test]
fn test_property_builder() {
    // Test is_true
    let condition = property("test.bool".to_string()).is_true();
    match &condition {
        Condition::SystemProperty { path, value, operator } => {
            assert_eq!(path, "test.bool");
            assert_eq!(value, &json!(true));
            assert!(matches!(operator, ComparisonOperator::Equals));
        }
        _ => panic!("Wrong condition type"),
    }
    
    // Test is_false
    let condition = property("test.bool".to_string()).is_false();
    match &condition {
        Condition::SystemProperty { path, value, operator } => {
            assert_eq!(path, "test.bool");
            assert_eq!(value, &json!(false));
            assert!(matches!(operator, ComparisonOperator::Equals));
        }
        _ => panic!("Wrong condition type"),
    }
    
    // Test equals
    let condition = property("test.string".to_string()).equals(json!("value"));
    match &condition {
        Condition::SystemProperty { path, value, operator } => {
            assert_eq!(path, "test.string");
            assert_eq!(value, &json!("value"));
            assert!(matches!(operator, ComparisonOperator::Equals));
        }
        _ => panic!("Wrong condition type"),
    }
    
    // Test contains
    let condition = property("test.string".to_string()).contains("substr".to_string());
    match &condition {
        Condition::SystemProperty { path, value, operator } => {
            assert_eq!(path, "test.string");
            assert_eq!(value, &json!("substr"));
            assert!(matches!(operator, ComparisonOperator::Contains));
        }
        _ => panic!("Wrong condition type"),
    }
}

#[test]
fn test_command_builder() {
    // Test exists
    let condition = command("git".to_string()).exists();
    match &condition {
        Condition::CommandExists { command } => {
            assert_eq!(command, "git");
        }
        _ => panic!("Wrong condition type"),
    }
    
    // Test succeeds
    let condition = command("test".to_string()).succeeds();
    match &condition {
        Condition::CommandSucceeds { command, args } => {
            assert_eq!(command, "test");
            assert!(args.is_none());
        }
        _ => panic!("Wrong condition type"),
    }
    
    // Test contains
    let condition = command("lsusb".to_string()).contains("fingerprint".to_string(), true);
    match &condition {
        Condition::CommandSucceeds { command, args } => {
            assert_eq!(command, "lsusb | grep -qi 'fingerprint'");
            assert!(args.is_none());
        }
        _ => panic!("Wrong condition type"),
    }
}

#[test]
fn test_comparison_operators() {
    // Test string contains
    let _condition = Condition::SystemProperty {
        path: "test".to_string(),
        value: json!("world"),
        operator: ComparisonOperator::Contains,
    };
    // This would need actual implementation to test properly
    
    // Just verify the enum values exist
    let _ops = vec![
        ComparisonOperator::Equals,
        ComparisonOperator::NotEquals,
        ComparisonOperator::Contains,
        ComparisonOperator::GreaterThan,
        ComparisonOperator::LessThan,
    ];
}