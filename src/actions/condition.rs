use crate::execution::VERBOSE_MODE;
use crate::system_info::SystemInfo;
use dhd_macros::{typescript_enum, typescript_fn, typescript_impl, typescript_type};
use std::path::Path;
use std::process::Command;

fn get_property_value(info: &SystemInfo, path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('.').collect();
    
    match parts.as_slice() {
        ["os", "family"] => Some(info.os.family.clone()),
        ["os", "distro"] => Some(info.os.distro.clone()),
        ["os", "version"] => Some(info.os.version.clone()),
        ["os", "codename"] => Some(info.os.codename.clone()),
        ["hardware", "fingerprint"] => Some(info.hardware.fingerprint.to_string()),
        ["hardware", "tpm"] => Some(info.hardware.tpm.to_string()),
        ["hardware", "gpu_vendor"] => Some(info.hardware.gpu_vendor.clone()),
        ["auth", "auth_type"] => Some(info.auth.auth_type.clone()),
        ["auth", "method"] => Some(info.auth.method.clone()),
        ["user", "name"] => Some(info.user.name.clone()),
        ["user", "shell"] => Some(info.user.shell.clone()),
        ["user", "home"] => Some(info.user.home.clone()),
        _ => None,
    }
}

#[typescript_enum]
pub enum ComparisonOperator {
    Equals,
    NotEquals,
    Contains,
    StartsWith,
    EndsWith,
}

#[typescript_enum]
pub enum Condition {
    // Core logical operators
    AllOf { conditions: Vec<Condition> },
    AnyOf { conditions: Vec<Condition> },
    Not { condition: Box<Condition> },
    
    // File system conditions
    FileExists { path: String },
    DirectoryExists { path: String },
    
    // Command conditions
    CommandExists { command: String },
    CommandSucceeds { command: String, args: Option<Vec<String>> },
    
    // Environment conditions
    EnvironmentVariable { name: String, value: Option<String> },
    
    // System property conditions
    SystemProperty { path: String, operator: ComparisonOperator, value: String },
    
    // Secret conditions
    SecretExists { reference: String },
}

impl Condition {
    pub fn evaluate(&self) -> Result<bool, String> {
        let verbose = VERBOSE_MODE.with(|v| *v.borrow());
        
        let result = match self {
            Condition::AllOf { conditions } => {
                for (i, condition) in conditions.iter().enumerate() {
                    match condition.evaluate() {
                        Ok(true) => continue,
                        Ok(false) => {
                            if verbose {
                                println!("        ✗ Condition {} of {} failed: {}", i + 1, conditions.len(), condition.describe());
                            }
                            return Ok(false);
                        }
                        Err(e) => return Err(e),
                    }
                }
                true
            }
            
            Condition::AnyOf { conditions } => {
                for (i, condition) in conditions.iter().enumerate() {
                    match condition.evaluate() {
                        Ok(true) => {
                            if verbose {
                                println!("        ✓ Condition {} of {} succeeded: {}", i + 1, conditions.len(), condition.describe());
                            }
                            return Ok(true);
                        }
                        Ok(false) => continue,
                        Err(e) => return Err(e),
                    }
                }
                false
            }
            
            Condition::Not { condition } => !condition.evaluate()?,
            
            Condition::FileExists { path } => {
                let expanded = shellexpand::tilde(path);
                Path::new(expanded.as_ref()).is_file()
            }
            
            Condition::DirectoryExists { path } => {
                let expanded = shellexpand::tilde(path);
                Path::new(expanded.as_ref()).is_dir()
            }
            
            Condition::CommandExists { command } => {
                which::which(command).is_ok()
            }
            
            Condition::CommandSucceeds { command, args } => {
                let mut cmd = Command::new(command);
                if let Some(args) = args {
                    cmd.args(args);
                }
                
                match cmd.output() {
                    Ok(output) => output.status.success(),
                    Err(_) => false,
                }
            }
            
            Condition::EnvironmentVariable { name, value } => {
                match std::env::var(name) {
                    Ok(env_value) => {
                        if let Some(expected) = value {
                            env_value == *expected
                        } else {
                            true
                        }
                    }
                    Err(_) => false,
                }
            }
            
            Condition::SystemProperty { path, operator, value } => {
                let info = crate::system_info::get_system_info();
                
                match get_property_value(&info, path) {
                    Some(prop_value) => {
                        match operator {
                            ComparisonOperator::Equals => prop_value == *value,
                            ComparisonOperator::NotEquals => prop_value != *value,
                            ComparisonOperator::Contains => prop_value.contains(value),
                            ComparisonOperator::StartsWith => prop_value.starts_with(value),
                            ComparisonOperator::EndsWith => prop_value.ends_with(value),
                        }
                    }
                    None => false,
                }
            }
            
            Condition::SecretExists { reference } => {
                // For now, we just check if it's a valid reference format
                // In the future, we could actually check with the secret provider
                reference.starts_with("op://") || 
                reference.starts_with("env://") || 
                reference.starts_with("literal://")
            }
        };
        
        if verbose {
            let emoji = if result { "✓" } else { "✗" };
            println!("      {} Condition {}: {}", emoji, if result { "passed" } else { "failed" }, self.describe());
        }
        
        Ok(result)
    }
    
    pub fn describe(&self) -> String {
        match self {
            Condition::AllOf { conditions } => {
                format!("all of {} conditions", conditions.len())
            }
            Condition::AnyOf { conditions } => {
                format!("any of {} conditions", conditions.len())
            }
            Condition::Not { condition } => {
                format!("not ({})", condition.describe())
            }
            Condition::FileExists { path } => {
                format!("file exists: {}", path)
            }
            Condition::DirectoryExists { path } => {
                format!("directory exists: {}", path)
            }
            Condition::CommandExists { command } => {
                format!("command exists: {}", command)
            }
            Condition::CommandSucceeds { command, args } => {
                if let Some(args) = args {
                    format!("command succeeds: {} {}", command, args.join(" "))
                } else {
                    format!("command succeeds: {}", command)
                }
            }
            Condition::EnvironmentVariable { name, value } => {
                if let Some(value) = value {
                    format!("environment variable {} = {}", name, value)
                } else {
                    format!("environment variable {} is set", name)
                }
            }
            Condition::SystemProperty { path, operator, value } => {
                let op_str = match operator {
                    ComparisonOperator::Equals => "==",
                    ComparisonOperator::NotEquals => "!=",
                    ComparisonOperator::Contains => "contains",
                    ComparisonOperator::StartsWith => "starts with",
                    ComparisonOperator::EndsWith => "ends with",
                };
                format!("system property {} {} {}", path, op_str, value)
            }
            Condition::SecretExists { reference } => {
                format!("secret exists: {}", reference)
            }
        }
    }
}

// Builder pattern for conditions
#[typescript_type]
pub struct ConditionBuilder {
    condition: Condition,
}

#[typescript_impl]
impl ConditionBuilder {
    pub fn new(condition: Condition) -> Self {
        ConditionBuilder { condition }
    }
    
    pub fn and(self, other: Condition) -> Self {
        ConditionBuilder {
            condition: Condition::AllOf {
                conditions: vec![self.condition, other],
            },
        }
    }
    
    pub fn or(self, other: Condition) -> Self {
        ConditionBuilder {
            condition: Condition::AnyOf {
                conditions: vec![self.condition, other],
            },
        }
    }
    
    pub fn not(self) -> Self {
        ConditionBuilder {
            condition: Condition::Not {
                condition: Box::new(self.condition),
            },
        }
    }
    
    pub fn build(self) -> Condition {
        self.condition
    }
}

// Property path builder for system properties
#[typescript_type]
pub struct PropertyBuilder {
    path: String,
}

#[typescript_impl]
impl PropertyBuilder {
    pub fn new(path: String) -> Self {
        PropertyBuilder { path }
    }
    
    pub fn equals(self, value: String) -> Condition {
        Condition::SystemProperty {
            path: self.path,
            operator: ComparisonOperator::Equals,
            value,
        }
    }
    
    pub fn not_equals(self, value: String) -> Condition {
        Condition::SystemProperty {
            path: self.path,
            operator: ComparisonOperator::NotEquals,
            value,
        }
    }
    
    pub fn contains(self, value: String) -> Condition {
        Condition::SystemProperty {
            path: self.path,
            operator: ComparisonOperator::Contains,
            value,
        }
    }
    
    pub fn starts_with(self, value: String) -> Condition {
        Condition::SystemProperty {
            path: self.path,
            operator: ComparisonOperator::StartsWith,
            value,
        }
    }
    
    pub fn ends_with(self, value: String) -> Condition {
        Condition::SystemProperty {
            path: self.path,
            operator: ComparisonOperator::EndsWith,
            value,
        }
    }
}

// Helper functions for creating conditions
#[typescript_fn]
pub fn all_of(conditions: Vec<Condition>) -> Condition {
    Condition::AllOf { conditions }
}

#[typescript_fn]
pub fn any_of(conditions: Vec<Condition>) -> Condition {
    Condition::AnyOf { conditions }
}

#[typescript_fn]
pub fn not(condition: Condition) -> Condition {
    Condition::Not {
        condition: Box::new(condition),
    }
}

#[typescript_fn]
pub fn file_exists(path: String) -> Condition {
    Condition::FileExists { path }
}

#[typescript_fn]
pub fn directory_exists(path: String) -> Condition {
    Condition::DirectoryExists { path }
}

#[typescript_fn]
pub fn command_exists(command: String) -> Condition {
    Condition::CommandExists { command }
}

#[typescript_fn]
pub fn command_succeeds(command: String, args: Option<Vec<String>>) -> Condition {
    Condition::CommandSucceeds { command, args }
}

#[typescript_fn]
pub fn env_var(name: String, value: Option<String>) -> Condition {
    Condition::EnvironmentVariable { name, value }
}

#[typescript_fn]
pub fn property(path: String) -> PropertyBuilder {
    PropertyBuilder::new(path)
}

// Convenience functions for boolean operations
#[typescript_fn]
pub fn and(a: Condition, b: Condition) -> Condition {
    Condition::AllOf {
        conditions: vec![a, b],
    }
}

#[typescript_fn]
pub fn or(a: Condition, b: Condition) -> Condition {
    Condition::AnyOf {
        conditions: vec![a, b],
    }
}

// Alias for command_succeeds with no args
#[typescript_fn]
pub fn command(command: String) -> Condition {
    Condition::CommandSucceeds {
        command,
        args: None,
    }
}

// New secret-related condition helper
#[typescript_fn]
pub fn secret_exists(reference: String) -> Condition {
    Condition::SecretExists { reference }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_exists() {
        let condition = file_exists("/etc/passwd".to_string());
        assert!(condition.evaluate().unwrap_or(false));

        let condition = file_exists("/nonexistent".to_string());
        assert!(!condition.evaluate().unwrap_or(true));
    }

    #[test]
    fn test_directory_exists() {
        let condition = directory_exists("/tmp".to_string());
        assert!(condition.evaluate().unwrap_or(false));

        let condition = directory_exists("/nonexistent".to_string());
        assert!(!condition.evaluate().unwrap_or(true));
    }

    #[test]
    fn test_command_exists() {
        let condition = command_exists("sh".to_string());
        assert!(condition.evaluate().unwrap_or(false));

        let condition = command_exists("nonexistent_command".to_string());
        assert!(!condition.evaluate().unwrap_or(true));
    }

    #[test]
    fn test_all_of() {
        let condition = all_of(vec![
            file_exists("/etc/passwd".to_string()),
            directory_exists("/tmp".to_string()),
        ]);
        assert!(condition.evaluate().unwrap_or(false));

        let condition = all_of(vec![
            file_exists("/etc/passwd".to_string()),
            file_exists("/nonexistent".to_string()),
        ]);
        assert!(!condition.evaluate().unwrap_or(true));
    }

    #[test]
    fn test_any_of() {
        let condition = any_of(vec![
            file_exists("/nonexistent".to_string()),
            directory_exists("/tmp".to_string()),
        ]);
        assert!(condition.evaluate().unwrap_or(false));

        let condition = any_of(vec![
            file_exists("/nonexistent1".to_string()),
            file_exists("/nonexistent2".to_string()),
        ]);
        assert!(!condition.evaluate().unwrap_or(true));
    }

    #[test]
    fn test_not() {
        let condition = not(file_exists("/nonexistent".to_string()));
        assert!(condition.evaluate().unwrap_or(false));

        let condition = not(file_exists("/etc/passwd".to_string()));
        assert!(!condition.evaluate().unwrap_or(true));
    }

    #[test]
    fn test_environment_variable() {
        std::env::set_var("TEST_VAR", "test_value");
        
        let condition = env_var("TEST_VAR".to_string(), Some("test_value".to_string()));
        assert!(condition.evaluate().unwrap_or(false));

        let condition = env_var("TEST_VAR".to_string(), Some("wrong_value".to_string()));
        assert!(!condition.evaluate().unwrap_or(true));

        let condition = env_var("TEST_VAR".to_string(), None);
        assert!(condition.evaluate().unwrap_or(false));

        let condition = env_var("NONEXISTENT_VAR".to_string(), None);
        assert!(!condition.evaluate().unwrap_or(true));
    }

    #[test]
    fn test_secret_exists() {
        let condition = secret_exists("op://Personal/GitHub/token".to_string());
        assert!(condition.evaluate().unwrap_or(false));

        let condition = secret_exists("env://MY_SECRET".to_string());
        assert!(condition.evaluate().unwrap_or(false));

        let condition = secret_exists("literal://my-value".to_string());
        assert!(condition.evaluate().unwrap_or(false));

        let condition = secret_exists("invalid-reference".to_string());
        assert!(!condition.evaluate().unwrap_or(true));
    }

    #[test]
    fn test_builder_pattern() {
        let builder = ConditionBuilder::new(file_exists("/etc/passwd".to_string()));
        let condition = builder.and(directory_exists("/tmp".to_string())).build();
        
        match condition {
            Condition::AllOf { conditions } => assert_eq!(conditions.len(), 2),
            _ => panic!("Expected AllOf condition"),
        }
    }

    #[test]
    fn test_describe() {
        let condition = file_exists("/etc/passwd".to_string());
        assert_eq!(condition.describe(), "file exists: /etc/passwd");

        let condition = all_of(vec![
            file_exists("/etc/passwd".to_string()),
            directory_exists("/tmp".to_string()),
        ]);
        assert_eq!(condition.describe(), "all of 2 conditions");
    }
}