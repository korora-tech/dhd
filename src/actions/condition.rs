use dhd_macros::{typescript_enum, typescript_fn, typescript_type, typescript_impl};
use serde::{Deserialize, Serialize};

/// Represents different types of conditions that can be evaluated
#[derive(Serialize, Deserialize)]
#[typescript_enum]
pub enum Condition {
    /// Check if a file exists
    FileExists { path: String },
    /// Check if a directory exists
    DirectoryExists { path: String },
    /// Check if a command succeeds
    CommandSucceeds {
        command: String,
        args: Option<Vec<String>>,
    },
    /// Check environment variable
    EnvironmentVariable { name: String, value: Option<String> },
    /// Check system property
    SystemProperty { 
        path: String,  // e.g., "hardware.fingerprint", "os.family"
        value: serde_json::Value,  // Expected value
        operator: ComparisonOperator,
    },
    /// Check if a command exists in PATH
    CommandExists { command: String },
    /// All conditions must pass
    AllOf { conditions: Vec<Condition> },
    /// At least one condition must pass
    AnyOf { conditions: Vec<Condition> },
    /// Negate a condition
    Not { condition: Box<Condition> },
}

#[derive(Serialize, Deserialize)]
#[typescript_enum]
pub enum ComparisonOperator {
    Equals,
    NotEquals,
    Contains,
    GreaterThan,
    LessThan,
}

impl Condition {
    /// Evaluate the condition
    pub fn evaluate(&self) -> Result<bool, String> {
        match self {
            Condition::FileExists { path } => {
                Ok(std::path::Path::new(path).exists() && std::path::Path::new(path).is_file())
            }
            Condition::DirectoryExists { path } => {
                Ok(std::path::Path::new(path).exists() && std::path::Path::new(path).is_dir())
            }
            Condition::CommandSucceeds { command, args } => {
                use std::process::Command;

                let mut cmd = Command::new("sh");
                cmd.arg("-c");

                if let Some(args) = args {
                    cmd.arg(format!("{} {}", command, args.join(" ")));
                } else {
                    cmd.arg(command);
                }

                match cmd.output() {
                    Ok(output) => Ok(output.status.success()),
                    Err(e) => Err(format!("Failed to execute command: {}", e)),
                }
            }
            Condition::EnvironmentVariable { name, value } => {
                match std::env::var(name) {
                    Ok(actual_value) => {
                        if let Some(expected_value) = value {
                            Ok(actual_value == *expected_value)
                        } else {
                            Ok(true) // Variable exists
                        }
                    }
                    Err(_) => Ok(false), // Variable doesn't exist
                }
            }
            Condition::SystemProperty { path, value, operator } => {
                // Get system info
                let info = crate::system_info::get_system_info();
                
                // Parse the property path and get the value
                let actual_value = match path.as_str() {
                    "os.family" => serde_json::Value::String(info.os.family),
                    "os.distro" => serde_json::Value::String(info.os.distro),
                    "os.version" => serde_json::Value::String(info.os.version),
                    "os.codename" => serde_json::Value::String(info.os.codename),
                    "hardware.fingerprint" => serde_json::Value::Bool(info.hardware.fingerprint),
                    "hardware.tpm" => serde_json::Value::Bool(info.hardware.tpm),
                    "hardware.gpu_vendor" => serde_json::Value::String(info.hardware.gpu_vendor),
                    "auth.type" => serde_json::Value::String(info.auth.auth_type),
                    "auth.method" => serde_json::Value::String(info.auth.method),
                    "user.name" => serde_json::Value::String(info.user.name),
                    "user.shell" => serde_json::Value::String(info.user.shell),
                    "user.home" => serde_json::Value::String(info.user.home),
                    _ => return Err(format!("Unknown system property: {}", path)),
                };
                
                // Compare values based on operator
                match operator {
                    ComparisonOperator::Equals => Ok(actual_value == *value),
                    ComparisonOperator::NotEquals => Ok(actual_value != *value),
                    ComparisonOperator::Contains => {
                        match (&actual_value, value) {
                            (serde_json::Value::String(s1), serde_json::Value::String(s2)) => {
                                Ok(s1.contains(s2.as_str()))
                            }
                            _ => Err("Contains operator only works with strings".to_string()),
                        }
                    }
                    ComparisonOperator::GreaterThan | ComparisonOperator::LessThan => {
                        match (&actual_value, value) {
                            (serde_json::Value::Number(n1), serde_json::Value::Number(n2)) => {
                                if let (Some(f1), Some(f2)) = (n1.as_f64(), n2.as_f64()) {
                                    Ok(match operator {
                                        ComparisonOperator::GreaterThan => f1 > f2,
                                        ComparisonOperator::LessThan => f1 < f2,
                                        _ => unreachable!(),
                                    })
                                } else {
                                    Err("Cannot compare numbers".to_string())
                                }
                            }
                            _ => Err("Comparison operators only work with numbers".to_string()),
                        }
                    }
                }
            }
            Condition::CommandExists { command } => {
                Ok(which::which(command).is_ok())
            }
            Condition::AllOf { conditions } => {
                for condition in conditions {
                    if !condition.evaluate()? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Condition::AnyOf { conditions } => {
                for condition in conditions {
                    if condition.evaluate()? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Condition::Not { condition } => Ok(!condition.evaluate()?),
        }
    }

    /// Get a description of the condition
    pub fn describe(&self) -> String {
        match self {
            Condition::FileExists { path } => format!("file exists: {}", path),
            Condition::DirectoryExists { path } => format!("directory exists: {}", path),
            Condition::CommandSucceeds { command, args } => {
                if let Some(args) = args {
                    format!("command succeeds: {} {}", command, args.join(" "))
                } else {
                    format!("command succeeds: {}", command)
                }
            }
            Condition::EnvironmentVariable { name, value } => {
                if let Some(value) = value {
                    format!("env var {}={}", name, value)
                } else {
                    format!("env var {} is set", name)
                }
            }
            Condition::SystemProperty { path, value, operator } => {
                let op_str = match operator {
                    ComparisonOperator::Equals => "==",
                    ComparisonOperator::NotEquals => "!=",
                    ComparisonOperator::Contains => "contains",
                    ComparisonOperator::GreaterThan => ">",
                    ComparisonOperator::LessThan => "<",
                };
                format!("{} {} {}", path, op_str, value)
            }
            Condition::CommandExists { command } => {
                format!("command exists: {}", command)
            }
            Condition::AllOf { conditions } => {
                let descs: Vec<String> = conditions.iter().map(|c| c.describe()).collect();
                format!("all of: [{}]", descs.join(", "))
            }
            Condition::AnyOf { conditions } => {
                let descs: Vec<String> = conditions.iter().map(|c| c.describe()).collect();
                format!("any of: [{}]", descs.join(", "))
            }
            Condition::Not { condition } => format!("not: {}", condition.describe()),
        }
    }
}

// Helper functions for TypeScript
#[typescript_fn]
pub fn file_exists(path: String) -> Condition {
    Condition::FileExists { path }
}

#[typescript_fn]
pub fn directory_exists(path: String) -> Condition {
    Condition::DirectoryExists { path }
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
pub fn command_exists(command: String) -> Condition {
    Condition::CommandExists { command }
}

// Builder-style functions for system properties
#[typescript_type]
pub struct PropertyBuilder {
    pub path: String,
}

#[typescript_impl]
impl PropertyBuilder {
    pub fn equals(self, value: serde_json::Value) -> Condition {
        Condition::SystemProperty {
            path: self.path,
            value,
            operator: ComparisonOperator::Equals,
        }
    }
    
    pub fn not_equals(self, value: serde_json::Value) -> Condition {
        Condition::SystemProperty {
            path: self.path,
            value,
            operator: ComparisonOperator::NotEquals,
        }
    }
    
    pub fn contains(self, value: String) -> Condition {
        Condition::SystemProperty {
            path: self.path,
            value: serde_json::Value::String(value),
            operator: ComparisonOperator::Contains,
        }
    }
    
    pub fn is_true(self) -> Condition {
        Condition::SystemProperty {
            path: self.path,
            value: serde_json::Value::Bool(true),
            operator: ComparisonOperator::Equals,
        }
    }
    
    pub fn is_false(self) -> Condition {
        Condition::SystemProperty {
            path: self.path,
            value: serde_json::Value::Bool(false),
            operator: ComparisonOperator::Equals,
        }
    }
}

#[typescript_fn]
pub fn property(path: String) -> PropertyBuilder {
    PropertyBuilder { path }
}

// Convenient builder for command output checking
#[typescript_type]
pub struct CommandBuilder {
    pub command: String,
}

#[typescript_impl]
impl CommandBuilder {
    pub fn succeeds(self) -> Condition {
        Condition::CommandSucceeds {
            command: self.command,
            args: None,
        }
    }
    
    pub fn exists(self) -> Condition {
        Condition::CommandExists {
            command: self.command,
        }
    }
    
    pub fn contains(self, text: String, case_insensitive: bool) -> Condition {
        let cmd = if case_insensitive {
            format!("{} | grep -qi '{}'", self.command, text)
        } else {
            format!("{} | grep -q '{}'", self.command, text)
        };
        Condition::CommandSucceeds {
            command: cmd,
            args: None,
        }
    }
}

#[typescript_fn]
pub fn command(cmd: String) -> CommandBuilder {
    CommandBuilder { command: cmd }
}

// Aliases for more intuitive API
#[typescript_fn]
pub fn or(conditions: Vec<Condition>) -> Condition {
    any_of(conditions)
}

#[typescript_fn]
pub fn and(conditions: Vec<Condition>) -> Condition {
    all_of(conditions)
}
