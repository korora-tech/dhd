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

impl RunCommand {
    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
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

        if !output.status.success() {
            return Err(format!(
                "Command {} failed with exit code: {:?}",
                self.command,
                output.status.code()
            )
            .into());
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
        
        assert!(cmd.execute().is_ok());
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
        
        assert!(cmd.execute().is_ok());
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
        
        assert!(cmd.execute().is_ok());
    }

    #[test]
    fn test_run_command_failure() {
        let cmd = RunCommand {
            command: "false".to_string(),
            args: None,
            cwd: None,
            env: None,
        };
        
        let result = cmd.execute();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Command false failed"));
    }

    #[test]
    fn test_run_command_not_found() {
        let cmd = RunCommand {
            command: "nonexistent_command_12345".to_string(),
            args: None,
            cwd: None,
            env: None,
        };
        
        assert!(cmd.execute().is_err());
    }
}
