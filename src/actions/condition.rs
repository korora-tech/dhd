use dhd_macros::{typescript_enum, typescript_fn};
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
    /// All conditions must pass
    AllOf { conditions: Vec<Condition> },
    /// At least one condition must pass
    AnyOf { conditions: Vec<Condition> },
    /// Negate a condition
    Not { condition: Box<Condition> },
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
                    cmd.arg(&format!("{} {}", command, args.join(" ")));
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
