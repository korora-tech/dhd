use dhd_macros::{typescript_fn, typescript_type};
use std::path::Path;
use crate::atoms::AtomCompat;

#[typescript_type]
pub struct GitConfigEntry {
    /// The configuration key (e.g., "user.name", "core.editor", "alias.co")
    pub key: String,
    /// The value for this key
    pub value: String,
    /// Whether to add this value (for multi-valued keys) instead of replacing
    pub add: Option<bool>,
}

#[typescript_type]
pub struct GitConfig {
    /// Git configuration entries to set
    pub entries: Vec<GitConfigEntry>,
    /// Whether to set globally (~/.gitconfig)
    pub global: Option<bool>,
    /// Whether to set system-wide (/etc/gitconfig)
    pub system: Option<bool>,
    /// Whether to unset the configuration values
    pub unset: Option<bool>,
}

impl crate::actions::Action for GitConfig {
    fn name(&self) -> &str {
        "GitConfig"
    }

    fn plan(&self, _module_dir: &Path) -> Vec<Box<dyn crate::atom::Atom>> {
        let scope = if self.system.unwrap_or(false) {
            crate::atoms::git_config::GitConfigScope::System
        } else if self.global.unwrap_or(false) {
            crate::atoms::git_config::GitConfigScope::Global
        } else {
            crate::atoms::git_config::GitConfigScope::Local
        };

        let atom_entries: Vec<crate::atoms::git_config::GitConfigEntry> = self.entries
            .iter()
            .map(|e| crate::atoms::git_config::GitConfigEntry {
                key: e.key.clone(),
                value: e.value.clone(),
                add: e.add,
            })
            .collect();

        vec![Box::new(AtomCompat::new(
            Box::new(crate::atoms::git_config::GitConfig::new(
                atom_entries,
                scope,
                self.unset.unwrap_or(false),
            )),
            "git_config".to_string(),
        ))]
    }
}

#[typescript_fn]
pub fn git_config(config: GitConfig) -> crate::actions::ActionType {
    crate::actions::ActionType::GitConfig(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::Action;

    #[test]
    fn test_git_config_creation() {
        let entries = vec![
            GitConfigEntry {
                key: "user.name".to_string(),
                value: "Test User".to_string(),
                add: None,
            },
            GitConfigEntry {
                key: "user.email".to_string(),
                value: "test@example.com".to_string(),
                add: None,
            },
        ];

        let action = GitConfig {
            entries: entries.clone(),
            global: Some(true),
            system: None,
            unset: None,
        };

        assert_eq!(action.entries.len(), 2);
        assert_eq!(action.entries[0].key, "user.name");
        assert_eq!(action.entries[0].value, "Test User");
        assert_eq!(action.global, Some(true));
        assert_eq!(action.system, None);
    }

    #[test]
    fn test_git_config_helper_function() {
        let entries = vec![
            GitConfigEntry {
                key: "core.editor".to_string(),
                value: "vim".to_string(),
                add: None,
            },
        ];

        let action = git_config(GitConfig {
            entries,
            global: Some(false),
            system: Some(false),
            unset: None,
        });

        match action {
            crate::actions::ActionType::GitConfig(git_cfg) => {
                assert_eq!(git_cfg.entries[0].key, "core.editor");
                assert_eq!(git_cfg.entries[0].value, "vim");
                assert_eq!(git_cfg.global, Some(false));
                assert_eq!(git_cfg.system, Some(false));
            }
            _ => panic!("Expected GitConfig action type"),
        }
    }

    #[test]
    fn test_git_config_name() {
        let action = GitConfig {
            entries: vec![],
            global: None,
            system: None,
            unset: None,
        };

        assert_eq!(action.name(), "GitConfig");
    }

    #[test]
    fn test_git_config_plan_global() {
        let entries = vec![
            GitConfigEntry {
                key: "user.name".to_string(),
                value: "Global User".to_string(),
                add: None,
            },
        ];

        let action = GitConfig {
            entries,
            global: Some(true),
            system: None,
            unset: None,
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
    }

    #[test]
    fn test_git_config_plan_system() {
        let entries = vec![
            GitConfigEntry {
                key: "core.autocrlf".to_string(),
                value: "true".to_string(),
                add: None,
            },
        ];

        let action = GitConfig {
            entries,
            global: None,
            system: Some(true),
            unset: None,
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
    }

    #[test]
    fn test_git_config_plan_local() {
        let entries = vec![
            GitConfigEntry {
                key: "remote.origin.url".to_string(),
                value: "https://github.com/example/repo.git".to_string(),
                add: None,
            },
        ];

        let action = GitConfig {
            entries,
            global: Some(false),
            system: Some(false),
            unset: None,
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
    }

    #[test]
    fn test_git_config_multiple_entries() {
        let entries = vec![
            GitConfigEntry {
                key: "user.name".to_string(),
                value: "John Doe".to_string(),
                add: None,
            },
            GitConfigEntry {
                key: "user.email".to_string(),
                value: "john@example.com".to_string(),
                add: None,
            },
            GitConfigEntry {
                key: "core.editor".to_string(),
                value: "nano".to_string(),
                add: None,
            },
            GitConfigEntry {
                key: "init.defaultBranch".to_string(),
                value: "main".to_string(),
                add: None,
            },
        ];

        let action = GitConfig {
            entries: entries.clone(),
            global: Some(true),
            system: None,
            unset: None,
        };

        assert_eq!(action.entries.len(), 4);
        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
    }

    #[test]
    fn test_git_config_multi_value() {
        let entries = vec![
            GitConfigEntry {
                key: "credential.https://github.com.helper".to_string(),
                value: "!/usr/bin/gh auth git-credential".to_string(),
                add: Some(true),
            },
        ];

        let action = GitConfig {
            entries,
            global: Some(true),
            system: None,
            unset: None,
        };

        assert_eq!(action.entries[0].add, Some(true));
    }

    #[test]
    fn test_git_config_unset() {
        let entries = vec![
            GitConfigEntry {
                key: "user.signingkey".to_string(),
                value: "".to_string(),
                add: None,
            },
        ];

        let action = GitConfig {
            entries,
            global: Some(true),
            system: None,
            unset: Some(true),
        };

        assert_eq!(action.unset, Some(true));
    }
}