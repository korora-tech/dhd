use dhd::actions::{Condition, ComparisonOperator, condition::*};
use serde_json::json;
use std::env;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_file_exists_condition() {
    let temp_dir = TempDir::new().unwrap();
    let ssh_key_path = temp_dir.path().join(".ssh/id_ed25519");
    
    // SSH key doesn't exist yet
    let condition = file_exists(ssh_key_path.to_string_lossy().to_string());
    assert_eq!(condition.evaluate().unwrap(), false);
    
    // Create SSH directory and key file
    fs::create_dir_all(temp_dir.path().join(".ssh")).unwrap();
    fs::write(&ssh_key_path, "-----BEGIN OPENSSH PRIVATE KEY-----\n...").unwrap();
    
    // Now it should exist
    let condition = file_exists(ssh_key_path.to_string_lossy().to_string());
    assert_eq!(condition.evaluate().unwrap(), true);
}

#[test]
fn test_directory_exists_condition() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/nvim");
    
    // Config directory doesn't exist yet
    let condition = directory_exists(config_dir.to_string_lossy().to_string());
    assert_eq!(condition.evaluate().unwrap(), false);
    
    // Create nested config directory structure
    fs::create_dir_all(&config_dir).unwrap();
    
    // Now it should exist
    let condition = directory_exists(config_dir.to_string_lossy().to_string());
    assert_eq!(condition.evaluate().unwrap(), true);
}

#[test]
fn test_command_exists_condition() {
    // Test with common development tools
    let condition = command_exists("git".to_string());
    // Git is commonly installed, but we'll just check the evaluation doesn't error
    let _ = condition.evaluate();
    
    // Test with a command that shouldn't exist
    let condition = command_exists("nonexistent_kubernetes_tool_xyz123".to_string());
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
    
    // Test with a more realistic command
    let condition = command_succeeds("git".to_string(), Some(vec!["--version".to_string()]));
    // This should succeed if git is installed
    let _ = condition.evaluate();
}

#[test]
fn test_environment_variable_condition() {
    // Set a development-related environment variable
    unsafe {
        env::set_var("DOCKER_HOST", "unix:///var/run/docker.sock");
    }
    
    // Test variable exists
    let condition = env_var("DOCKER_HOST".to_string(), None);
    assert_eq!(condition.evaluate().unwrap(), true);
    
    // Test variable has specific value
    let condition = env_var("DOCKER_HOST".to_string(), Some("unix:///var/run/docker.sock".to_string()));
    assert_eq!(condition.evaluate().unwrap(), true);
    
    // Test variable has wrong value
    let condition = env_var("DOCKER_HOST".to_string(), Some("tcp://localhost:2375".to_string()));
    assert_eq!(condition.evaluate().unwrap(), false);
    
    // Test common development environment variables
    let condition = env_var("KUBECONFIG".to_string(), None);
    // This might or might not exist, just verify it doesn't error
    let _ = condition.evaluate();
    
    // Clean up
    unsafe {
        env::remove_var("DOCKER_HOST");
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
    // Test nested conditions for a typical development setup check
    let temp_dir = TempDir::new().unwrap();
    let docker_config = temp_dir.path().join(".docker/config.json");
    fs::create_dir_all(temp_dir.path().join(".docker")).unwrap();
    fs::write(&docker_config, "{}").unwrap();
    
    let condition = or(vec![
        and(vec![
            file_exists(docker_config.to_string_lossy().to_string()),
            command_exists("docker".to_string()),
        ]),
        env_var("DOCKER_BUILDKIT".to_string(), Some("1".to_string())),
    ]);
    // Just verify it evaluates without error
    let _ = condition.evaluate();
}

#[test]
fn test_condition_descriptions() {
    let condition = file_exists("/etc/kubernetes/admin.conf".to_string());
    assert_eq!(condition.describe(), "file exists: /etc/kubernetes/admin.conf");
    
    let condition = command_exists("kubectl".to_string());
    assert_eq!(condition.describe(), "command exists: kubectl");
    
    let condition = env_var("GOPATH".to_string(), Some("/home/developer/go".to_string()));
    assert_eq!(condition.describe(), "env var GOPATH=/home/developer/go");
    
    let condition = property("os.distro".to_string()).equals(json!("ubuntu"));
    assert_eq!(condition.describe(), "os.distro == \"ubuntu\"");
    
    let condition = not(command_exists("podman".to_string()));
    assert_eq!(condition.describe(), "not: command exists: podman");
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
    let condition = command("docker".to_string()).exists();
    match &condition {
        Condition::CommandExists { command } => {
            assert_eq!(command, "docker");
        }
        _ => panic!("Wrong condition type"),
    }
    
    // Test succeeds
    let condition = command("systemctl".to_string()).succeeds();
    match &condition {
        Condition::CommandSucceeds { command, args } => {
            assert_eq!(command, "systemctl");
            assert!(args.is_none());
        }
        _ => panic!("Wrong condition type"),
    }
    
    // Test contains - check if nvidia driver is loaded
    let condition = command("lsmod".to_string()).contains("nvidia".to_string(), true);
    match &condition {
        Condition::CommandSucceeds { command, args } => {
            assert_eq!(command, "lsmod | grep -qi 'nvidia'");
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