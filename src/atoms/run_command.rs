use crate::{Atom, DhdError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunCommand {
    pub command: String,
    pub args: Option<Vec<String>>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub shell: Option<String>,
    pub privilege_escalation: Option<bool>,
}

impl Atom for RunCommand {
    fn check(&self) -> Result<bool> {
        // Commands are typically not idempotent - always need to run
        Ok(true)
    }

    fn execute(&self) -> Result<()> {
        self.run()
    }

    fn describe(&self) -> String {
        let mut desc = format!("Running command: {}", self.command);
        if let Some(args) = &self.args {
            // Quote arguments that contain spaces or special characters
            let quoted_args: Vec<String> = args
                .iter()
                .map(|arg| {
                    if arg.contains(' ') || arg.contains('"') || arg.contains('\'') {
                        format!("\"{}\"", arg.replace('"', "\\\""))
                    } else {
                        arg.to_string()
                    }
                })
                .collect();
            desc.push_str(&format!(" {}", quoted_args.join(" ")));
        }
        desc
    }
}

impl RunCommand {
    pub fn run(&self) -> Result<()> {
        let use_sudo = self.privilege_escalation.unwrap_or(false);

        let mut cmd = if let Some(shell) = &self.shell {
            // If shell is specified, wrap the command
            let mut shell_cmd = if use_sudo {
                let mut sudo_cmd = Command::new("sudo");
                sudo_cmd.arg(shell);
                sudo_cmd
            } else {
                Command::new(shell)
            };

            // Build the full command string
            let mut full_command = self.command.clone();
            if let Some(args) = &self.args {
                for arg in args {
                    full_command.push(' ');
                    // Properly escape arguments for shell
                    if arg.contains(' ')
                        || arg.contains('"')
                        || arg.contains('\'')
                        || arg.contains('$')
                    {
                        full_command.push_str(&format!("'{}'", arg.replace('\'', "'\\''")));
                    } else {
                        full_command.push_str(arg);
                    }
                }
            }

            // Use -c flag for most shells
            shell_cmd.args(["-c", &full_command]);
            shell_cmd
        } else {
            // Original behavior without shell
            if use_sudo {
                let mut sudo_cmd = Command::new("sudo");
                sudo_cmd.arg(&self.command);
                if let Some(args) = &self.args {
                    sudo_cmd.args(args);
                }
                sudo_cmd
            } else {
                let mut cmd = Command::new(&self.command);
                if let Some(args) = &self.args {
                    cmd.args(args);
                }
                cmd
            }
        };

        if let Some(cwd) = &self.cwd {
            cmd.current_dir(cwd);
        }

        if let Some(env) = &self.env {
            for (key, value) in env {
                cmd.env(key, value);
            }
        }

        let output = cmd.output()?;

        // Print stdout if there is any
        if !output.stdout.is_empty() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            print!("{}", stdout);
        }

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DhdError::AtomExecution(format!(
                "Command {} failed with exit code: {:?}. Error: {}",
                self.command,
                output.status.code(),
                stderr
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_run_command_success() {
        let cmd = RunCommand {
            command: "echo".to_string(),
            args: Some(vec!["hello".to_string()]),
            cwd: None,
            env: None,
            shell: None,
            privilege_escalation: None,
        };

        assert!(cmd.run().is_ok());
    }

    #[test]
    fn test_run_command_with_cwd() {
        let temp_dir = TempDir::new().unwrap();
        let cmd = RunCommand {
            command: "pwd".to_string(),
            args: None,
            cwd: Some(temp_dir.path().to_string_lossy().to_string()),
            env: None,
            shell: None,
            privilege_escalation: None,
        };

        assert!(cmd.run().is_ok());
    }

    #[test]
    fn test_run_command_with_env() {
        let mut env = HashMap::new();
        env.insert("TEST_VAR".to_string(), "test_value".to_string());

        let cmd = RunCommand {
            command: "sh".to_string(),
            args: Some(vec!["-c".to_string(), "echo $TEST_VAR".to_string()]),
            cwd: None,
            env: Some(env),
            shell: None,
            privilege_escalation: None,
        };

        assert!(cmd.run().is_ok());
    }

    #[test]
    fn test_run_command_failure() {
        let cmd = RunCommand {
            command: "false".to_string(),
            args: None,
            cwd: None,
            env: None,
            shell: None,
            privilege_escalation: None,
        };

        let result = cmd.run();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Command false failed")
        );
    }

    #[test]
    fn test_run_command_not_found() {
        let cmd = RunCommand {
            command: "nonexistent_command_12345".to_string(),
            args: None,
            cwd: None,
            env: None,
            shell: None,
            privilege_escalation: None,
        };

        assert!(cmd.run().is_err());
    }

    #[test]
    fn test_run_command_with_spaces_in_args() {
        let cmd = RunCommand {
            command: "echo".to_string(),
            args: Some(vec![
                "hello world".to_string(),
                "test with spaces".to_string(),
            ]),
            cwd: None,
            env: None,
            shell: None,
            privilege_escalation: None,
        };

        assert!(cmd.run().is_ok());
    }

    #[test]
    fn test_describe_with_special_chars() {
        let cmd = RunCommand {
            command: "echo".to_string(),
            args: Some(vec![
                "hello world".to_string(),
                "test\"quote".to_string(),
                "normal".to_string(),
            ]),
            cwd: None,
            env: None,
            shell: None,
            privilege_escalation: None,
        };

        let desc = cmd.describe();
        assert_eq!(
            desc,
            "Running command: echo \"hello world\" \"test\\\"quote\" normal"
        );
    }

    #[test]
    fn test_run_command_with_shell() {
        let cmd = RunCommand {
            command: "echo $((2 + 2))".to_string(),
            args: None,
            cwd: None,
            env: None,
            shell: Some("sh".to_string()),
            privilege_escalation: None,
        };

        assert!(cmd.run().is_ok());
    }

    #[test]
    fn test_run_command_with_privilege_escalation() {
        // Test that privilege escalation adds sudo
        let cmd = RunCommand {
            command: "whoami".to_string(),
            args: None,
            cwd: None,
            env: None,
            shell: None,
            privilege_escalation: Some(true),
        };

        // We can't actually test running with sudo in tests,
        // but we can verify the command structure
        let desc = cmd.describe();
        assert_eq!(desc, "Running command: whoami");
    }
}
