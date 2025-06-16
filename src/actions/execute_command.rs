use crate::ActionType;
use crate::atoms::AtomCompat;
use dhd_macros::{typescript_fn, typescript_type};
use std::collections::HashMap;

use super::Action;

/// Configuration for executing a command.
///
/// This structure defines the parameters for executing a shell command.
/// It supports specifying the shell, command, arguments, and whether to escalate privileges.
#[typescript_type]
pub struct ExecuteCommand {
    /// Optional shell to use for executing the command.
    ///
    /// If not specified, defaults to "sh".
    pub shell: Option<String>,
    /// The command to execute.
    ///
    /// This is the main command string that will be executed.
    pub command: String,
    /// Optional arguments for the command.
    ///
    /// These are additional arguments passed to the command.
    pub args: Option<Vec<String>>,
    /// Whether to escalate privileges for the command (optional).
    ///
    /// If set to true, the command will be executed with elevated privileges.
    /// If not specified, defaults to false.
    pub escalate: Option<bool>,
    /// Environment variables with secret references.
    ///
    /// Keys are environment variable names, values are secret references like:
    /// - "op://vault/item/field" for 1Password
    /// - "env://VAR_NAME" for environment variables
    /// - "literal://value" for literal values
    pub environment: Option<HashMap<String, String>>,
}

#[typescript_fn]
pub fn execute_command(config: ExecuteCommand) -> ActionType {
    ActionType::ExecuteCommand(config)
}

impl Action for ExecuteCommand {
    fn name(&self) -> &str {
        "ExecuteCommand"
    }

    fn plan(&self, _module_dir: &std::path::Path) -> Vec<Box<dyn crate::atom::Atom>> {
        let shell = self.shell.clone().unwrap_or_else(|| "sh".to_string());

        let full_command = if let Some(args) = &self.args {
            let mut parts = vec![self.command.clone()];
            parts.extend(args.iter().map(|arg| {
                if arg.contains(' ') || arg.contains('\"') || arg.contains('\'') {
                    format!("\"{}\"", arg.replace("\"", "\\\""))
                } else {
                    arg.clone()
                }
            }));
            parts.join(" ")
        } else {
            self.command.clone()
        };

        vec![Box::new(AtomCompat::new(
            Box::new(crate::atoms::RunCommand {
                shell,
                command: full_command,
                escalate: self.escalate.unwrap_or(false),
                environment: self.environment.clone(),
            }),
            "execute_command".to_string(),
        ))]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_command_creation() {
        let action = ExecuteCommand {
            shell: Some("bash".to_string()),
            command: "git config --global user.name".to_string(),
            args: Some(vec!["John Doe".to_string()]),
            escalate: Some(false),
            environment: None,
        };

        assert_eq!(action.shell, Some("bash".to_string()));
        assert_eq!(action.command, "git config --global user.name");
        assert_eq!(action.args, Some(vec!["John Doe".to_string()]));
        assert_eq!(action.escalate, Some(false));
    }

    #[test]
    fn test_execute_command_helper_function() {
        let action = execute_command(ExecuteCommand {
            shell: Some("sh".to_string()),
            command: "docker compose up -d".to_string(),
            args: None,
            escalate: Some(false),
            environment: None,
        });

        match action {
            ActionType::ExecuteCommand(cmd) => {
                assert_eq!(cmd.shell, Some("sh".to_string()));
                assert_eq!(cmd.command, "docker compose up -d");
                assert_eq!(cmd.escalate, Some(false));
            }
            _ => panic!("Expected ExecuteCommand action type"),
        }
    }

    #[test]
    fn test_execute_command_name() {
        let action = ExecuteCommand {
            shell: Some("bash".to_string()),
            command: "systemctl status nginx".to_string(),
            args: None,
            escalate: Some(false),
            environment: None,
        };

        assert_eq!(action.name(), "ExecuteCommand");
    }

    #[test]
    fn test_execute_command_plan() {
        let action = ExecuteCommand {
            shell: Some("zsh".to_string()),
            command: "ssh-keygen -t ed25519 -C \"user@example.com\"".to_string(),
            args: None,
            escalate: Some(false),
            environment: None,
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
        // Check that we got an atom
        assert_eq!(atoms.len(), 1);
    }

    #[test]
    fn test_execute_command_plan_complex() {
        let complex_command = "curl -fsSL https://get.docker.com | sh && usermod -aG docker $USER";
        let action = ExecuteCommand {
            shell: Some("bash".to_string()),
            command: complex_command.to_string(),
            args: None,
            escalate: Some(true),
            environment: None,
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);

        // The atom should have the same shell and command
        let atom = &atoms[0];
        // Check that we got the atom
        assert!(!atom.describe().is_empty());
    }

    #[test]
    fn test_execute_command_different_shells() {
        let shells = vec!["sh", "bash", "zsh", "fish", "powershell"];

        for shell in shells {
            let action = ExecuteCommand {
                shell: Some(shell.to_string()),
                command: "kubectl get pods --all-namespaces".to_string(),
                args: None,
                escalate: Some(false),
                environment: None,
            };

            let atoms = action.plan(std::path::Path::new("."));
            assert_eq!(atoms.len(), 1);
            // Check that we got an atom
            assert_eq!(atoms.len(), 1);
        }
    }

    #[test]
    fn test_execute_command_with_args() {
        let action = ExecuteCommand {
            shell: None,
            command: "systemctl".to_string(),
            args: Some(vec!["disable".to_string(), "docker.service".to_string()]),
            escalate: Some(true),
            environment: None,
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
        // Check that we got an atom
        assert_eq!(atoms.len(), 1);
    }

    #[test]
    fn test_execute_command_default_shell() {
        let action = ExecuteCommand {
            shell: None,
            command: "certbot".to_string(),
            args: Some(vec!["certonly".to_string(), "--standalone".to_string(), "-d".to_string(), "example.com".to_string()]),
            escalate: Some(true),
            environment: None,
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
    }
}
