use dhd_macros::{typescript_fn, typescript_type};
use crate::ActionType;
use crate::atoms::AtomCompat;

use super::Action;

#[typescript_type]
pub struct ExecuteCommand {
    pub shell: Option<String>,
    pub command: String,
    pub args: Option<Vec<String>>,
    pub escalate: bool,
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
                escalate: self.escalate,
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
            command: "echo hello".to_string(),
            args: None,
            escalate: false,
        };

        assert_eq!(action.shell, Some("bash".to_string()));
        assert_eq!(action.command, "echo hello");
        assert_eq!(action.args, None);
        assert_eq!(action.escalate, false);
    }

    #[test]
    fn test_execute_command_helper_function() {
        let action = execute_command(ExecuteCommand {
            shell: Some("sh".to_string()),
            command: "ls -la".to_string(),
            args: None,
            escalate: false,
        });

        match action {
            ActionType::ExecuteCommand(cmd) => {
                assert_eq!(cmd.shell, Some("sh".to_string()));
                assert_eq!(cmd.command, "ls -la");
            }
            _ => panic!("Expected ExecuteCommand action type"),
        }
    }

    #[test]
    fn test_execute_command_name() {
        let action = ExecuteCommand {
            shell: Some("bash".to_string()),
            command: "echo test".to_string(),
            args: None,
            escalate: false,
        };

        assert_eq!(action.name(), "ExecuteCommand");
    }

    #[test]
    fn test_execute_command_plan() {
        let action = ExecuteCommand {
            shell: Some("zsh".to_string()),
            command: "pwd".to_string(),
            args: None,
            escalate: false,
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
        // Check that we got an atom
        assert_eq!(atoms.len(), 1);
    }

    #[test]
    fn test_execute_command_plan_complex() {
        let complex_command = "cd /tmp && echo 'test' > file.txt && cat file.txt";
        let action = ExecuteCommand {
            shell: Some("bash".to_string()),
            command: complex_command.to_string(),
            args: None,
            escalate: false,
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);

        // The atom should have the same shell and command
        let atom = &atoms[0];
        // Check that we got the atom
        assert!(atom.describe().len() > 0);
    }

    #[test]
    fn test_execute_command_different_shells() {
        let shells = vec!["sh", "bash", "zsh", "fish", "powershell"];

        for shell in shells {
            let action = ExecuteCommand {
                shell: Some(shell.to_string()),
                command: "echo test".to_string(),
                args: None,
                escalate: false,
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
            escalate: true,
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
            command: "echo".to_string(),
            args: Some(vec!["hello world".to_string()]),
            escalate: false,
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
    }
}