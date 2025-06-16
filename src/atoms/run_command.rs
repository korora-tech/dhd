use crate::atoms::Atom;
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct RunCommand {
    pub shell: String,
    pub command: String,
    pub escalate: bool,
    pub environment: Option<HashMap<String, String>>,
}

impl RunCommand {
    fn get_escalation_tool(&self) -> Result<String, String> {
        // Check for pkexec first (preferred on Linux with GUI)
        if std::process::Command::new("which")
            .arg("pkexec")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            return Ok("pkexec".to_string());
        }

        // Check for sudo (common on Unix-like systems)
        if std::process::Command::new("which")
            .arg("sudo")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            return Ok("sudo".to_string());
        }

        // Check for doas (OpenBSD/some BSDs)
        if std::process::Command::new("which")
            .arg("doas")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            return Ok("doas".to_string());
        }

        // Windows - would need runas or similar
        #[cfg(target_os = "windows")]
        {
            return Err("Privilege escalation on Windows is not yet supported".to_string());
        }

        Err("No suitable privilege escalation tool found (tried: pkexec, sudo, doas)".to_string())
    }
}

impl Atom for RunCommand {
    fn name(&self) -> &str {
        "RunCommand"
    }

    fn execute(&self) -> Result<(), String> {
        let mut cmd = if self.escalate {
            let escalation_tool = self.get_escalation_tool()?;
            let mut c = Command::new(&escalation_tool);
            c.arg(&self.shell).arg("-c").arg(&self.command);
            c
        } else {
            let mut c = Command::new(&self.shell);
            c.arg("-c").arg(&self.command);
            c
        };

        // Add environment variables if provided
        if let Some(env_vars) = &self.environment {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Command failed with exit code: {:?}\nstderr: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    fn describe(&self) -> String {
        if self.escalate {
            format!(
                "Run command (elevated): {} -c '{}'",
                self.shell, self.command
            )
        } else {
            format!("Run command: {} -c '{}'", self.shell, self.command)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_command_name() {
        let atom = RunCommand {
            shell: "bash".to_string(),
            command: "npm install --global typescript".to_string(),
            escalate: false,
            environment: None,
        };
        assert_eq!(atom.name(), "RunCommand");
    }

    #[test]
    fn test_run_command_clone() {
        let atom = RunCommand {
            shell: "bash".to_string(),
            command: "docker build -t myapp:latest .".to_string(),
            escalate: false,
            environment: None,
        };

        let cloned = atom.clone();
        assert_eq!(cloned.shell, atom.shell);
        assert_eq!(cloned.command, atom.command);
        assert_eq!(cloned.escalate, atom.escalate);
    }

    #[test]
    fn test_run_command_execute_success() {
        let atom = RunCommand {
            shell: "sh".to_string(), // Use sh for better portability
            command: "git --version > /dev/null".to_string(),
            escalate: false,
            environment: None,
        };

        let result = atom.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_command_execute_failure() {
        let atom = RunCommand {
            shell: "sh".to_string(),
            command: "exit 1".to_string(),
            escalate: false,
            environment: None,
        };

        let result = atom.execute();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Command failed with exit code"));
    }

    #[test]
    fn test_run_command_execute_invalid_command() {
        let atom = RunCommand {
            shell: "sh".to_string(),
            command: "nonexistent_command_that_should_not_exist".to_string(),
            escalate: false,
            environment: None,
        };

        let result = atom.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_run_command_execute_invalid_shell() {
        let atom = RunCommand {
            shell: "nonexistent_shell_that_should_not_exist".to_string(),
            command: "echo test".to_string(),
            escalate: false,
            environment: None,
        };

        let result = atom.execute();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to execute command"));
    }

    #[test]
    fn test_run_command_execute_with_stderr() {
        let atom = RunCommand {
            shell: "sh".to_string(),
            command: "echo 'error' >&2 && exit 1".to_string(),
            escalate: false,
            environment: None,
        };

        let result = atom.execute();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("stderr:"));
        assert!(error.contains("error"));
    }

    #[test]
    fn test_run_command_execute_complex_command() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join(".gitignore");

        let atom = RunCommand {
            shell: "sh".to_string(),
            command: format!("echo 'node_modules/\n*.log\n.env' > {}", config_file.display()),
            escalate: false,
            environment: None,
        };

        let result = atom.execute();
        assert!(result.is_ok());

        // Verify the command actually ran
        let content = fs::read_to_string(&config_file).unwrap();
        assert!(content.contains("node_modules/"));
        assert!(content.contains("*.log"));
        assert!(content.contains(".env"));
    }

    #[test]
    fn test_run_command_describe_with_escalate() {
        let atom = RunCommand {
            shell: "bash".to_string(),
            command: "apt update && apt install -y nginx".to_string(),
            escalate: true,
            environment: None,
        };
        assert_eq!(
            atom.describe(),
            "Run command (elevated): bash -c 'apt update && apt install -y nginx'"
        );

        let atom_no_escalate = RunCommand {
            shell: "bash".to_string(),
            command: "cargo build --release".to_string(),
            escalate: false,
            environment: None,
        };
        assert_eq!(
            atom_no_escalate.describe(),
            "Run command: bash -c 'cargo build --release'"
        );
    }
}
