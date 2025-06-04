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
            desc.push_str(&format!(" {}", args.join(" ")));
        }
        desc
    }
}

impl RunCommand {
    pub fn run(&self) -> Result<()> {
        let mut cmd = Command::new(&self.command);

        if let Some(args) = &self.args {
            cmd.args(args);
        }

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
        };

        assert!(cmd.run().is_err());
    }
}
