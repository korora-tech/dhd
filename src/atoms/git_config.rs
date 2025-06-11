use std::process::Command;
use crate::atoms::Atom;

#[derive(Debug, Clone, PartialEq)]
pub enum GitConfigScope {
    Local,
    Global,
    System,
}

#[derive(Debug, Clone)]
pub struct GitConfigEntry {
    pub key: String,
    pub value: String,
    pub add: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct GitConfig {
    pub entries: Vec<GitConfigEntry>,
    pub scope: GitConfigScope,
    pub unset: bool,
}

impl GitConfig {
    pub fn new(entries: Vec<GitConfigEntry>, scope: GitConfigScope, unset: bool) -> Self {
        Self { entries, scope, unset }
    }

    fn get_scope_arg(&self) -> Option<&'static str> {
        match self.scope {
            GitConfigScope::Local => None,
            GitConfigScope::Global => Some("--global"),
            GitConfigScope::System => Some("--system"),
        }
    }
}

impl Atom for GitConfig {
    fn name(&self) -> &str {
        "GitConfig"
    }

    fn execute(&self) -> Result<(), String> {
        for entry in &self.entries {
            let mut args = vec!["config"];
            
            if let Some(scope_arg) = self.get_scope_arg() {
                args.push(scope_arg);
            }
            
            if self.unset {
                args.push("--unset");
                args.push(&entry.key);
            } else if entry.add.unwrap_or(false) {
                args.push("--add");
                args.push(&entry.key);
                args.push(&entry.value);
            } else {
                args.push(&entry.key);
                args.push(&entry.value);
            }

            let output = Command::new("git")
                .args(&args)
                .output()
                .map_err(|e| format!("Failed to execute git config: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                // Ignore "no such section" errors when unsetting
                if self.unset && stderr.contains("no such section") {
                    continue;
                }
                return Err(format!(
                    "Failed to {} git config {}: {}",
                    if self.unset { "unset" } else { "set" },
                    entry.key,
                    stderr
                ));
            }
        }

        Ok(())
    }

    fn describe(&self) -> String {
        let scope_desc = match self.scope {
            GitConfigScope::Local => "local",
            GitConfigScope::Global => "global",
            GitConfigScope::System => "system",
        };
        
        let action = if self.unset { "Unset" } else { "Set" };
        
        format!(
            "{} {} git configuration ({} entries)",
            action,
            scope_desc,
            self.entries.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_config_creation() {
        let entries = vec![
            GitConfigEntry {
                key: "user.name".to_string(),
                value: "Test User".to_string(),
                add: None,
            },
        ];
        
        let git_config = GitConfig::new(entries.clone(), GitConfigScope::Global, false);
        
        assert_eq!(git_config.entries.len(), 1);
        assert_eq!(git_config.scope, GitConfigScope::Global);
        assert_eq!(git_config.unset, false);
    }

    #[test]
    fn test_git_config_name() {
        let git_config = GitConfig::new(vec![], GitConfigScope::Local, false);
        assert_eq!(git_config.name(), "GitConfig");
    }

    #[test]
    fn test_git_config_describe() {
        let entries = vec![
            GitConfigEntry {
                key: "user.name".to_string(),
                value: "Test".to_string(),
                add: None,
            },
            GitConfigEntry {
                key: "user.email".to_string(),
                value: "test@example.com".to_string(),
                add: None,
            },
        ];
        
        let git_config = GitConfig::new(entries, GitConfigScope::Global, false);
        assert_eq!(git_config.describe(), "Set global git configuration (2 entries)");
        
        let git_config_unset = GitConfig::new(vec![], GitConfigScope::Local, true);
        assert_eq!(git_config_unset.describe(), "Unset local git configuration (0 entries)");
    }

    #[test]
    fn test_get_scope_arg() {
        let git_config_local = GitConfig::new(vec![], GitConfigScope::Local, false);
        assert_eq!(git_config_local.get_scope_arg(), None);
        
        let git_config_global = GitConfig::new(vec![], GitConfigScope::Global, false);
        assert_eq!(git_config_global.get_scope_arg(), Some("--global"));
        
        let git_config_system = GitConfig::new(vec![], GitConfigScope::System, false);
        assert_eq!(git_config_system.get_scope_arg(), Some("--system"));
    }

    #[test]
    fn test_multi_value_entries() {
        let entries = vec![
            GitConfigEntry {
                key: "credential.helper".to_string(),
                value: "store".to_string(),
                add: Some(true),
            },
            GitConfigEntry {
                key: "credential.helper".to_string(),
                value: "cache".to_string(),
                add: Some(true),
            },
        ];
        
        let git_config = GitConfig::new(entries, GitConfigScope::Global, false);
        assert_eq!(git_config.entries.len(), 2);
        assert_eq!(git_config.entries[0].add, Some(true));
        assert_eq!(git_config.entries[1].add, Some(true));
    }
}