use crate::actions::Action;
use crate::{Atom, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub scope: GitConfigScope,
    pub configs: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GitConfigScope {
    Global,
    System,
    Local,
}

impl Action for GitConfig {
    fn plan(&self) -> Result<Vec<Box<dyn Atom>>> {
        debug!(
            "Planning git config: scope={:?}, configs={:?}",
            self.scope, self.configs
        );

        let mut atoms: Vec<Box<dyn Atom>> = vec![];

        // For each config key-value pair, create a git config command
        for (key, value) in &self.configs {
            let scope_flag = match self.scope {
                GitConfigScope::Global => "--global",
                GitConfigScope::System => "--system",
                GitConfigScope::Local => "--local",
            };

            atoms.push(Box::new(crate::atoms::RunCommand {
                command: "git".to_string(),
                args: Some(vec![
                    "config".to_string(),
                    scope_flag.to_string(),
                    key.clone(),
                    value.clone(),
                ]),
                cwd: None,
                env: None,
                shell: None,
            }));
        }

        Ok(atoms)
    }

    fn describe(&self) -> String {
        format!(
            "Set {:?} git config: {} entries",
            self.scope,
            self.configs.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_config_plan() {
        let mut configs = HashMap::new();
        configs.insert("user.name".to_string(), "Test User".to_string());
        configs.insert("user.email".to_string(), "test@example.com".to_string());

        let action = GitConfig {
            scope: GitConfigScope::Global,
            configs,
        };

        let atoms = action.plan().expect("Failed to plan git config action");
        assert_eq!(atoms.len(), 2, "Should create one atom per config entry");
    }

    #[test]
    fn test_git_config_describe() {
        let mut configs = HashMap::new();
        configs.insert("user.name".to_string(), "Test User".to_string());

        let action = GitConfig {
            scope: GitConfigScope::Local,
            configs,
        };

        let description = action.describe();
        assert!(description.contains("Local"));
        assert!(description.contains("1 entries"));
    }

    #[test]
    fn test_git_config_scopes() {
        let configs = HashMap::new();

        // Test global scope
        let global_action = GitConfig {
            scope: GitConfigScope::Global,
            configs: configs.clone(),
        };
        let global_atoms = global_action.plan().expect("Failed to plan");
        assert_eq!(
            global_atoms.len(),
            0,
            "Empty configs should produce no atoms"
        );

        // Test system scope
        let system_action = GitConfig {
            scope: GitConfigScope::System,
            configs: configs.clone(),
        };
        let system_atoms = system_action.plan().expect("Failed to plan");
        assert_eq!(
            system_atoms.len(),
            0,
            "Empty configs should produce no atoms"
        );

        // Test local scope
        let local_action = GitConfig {
            scope: GitConfigScope::Local,
            configs: configs.clone(),
        };
        let local_atoms = local_action.plan().expect("Failed to plan");
        assert_eq!(
            local_atoms.len(),
            0,
            "Empty configs should produce no atoms"
        );
    }
}
