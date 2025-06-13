use crate::atoms::Atom;
use gix_config::file::Metadata;
use gix_config::{File, Source};
use std::path::PathBuf;
use std::fs;
use bstr::BString;

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
        Self {
            entries,
            scope,
            unset,
        }
    }

    fn get_config_path(&self) -> Result<PathBuf, String> {
        match self.scope {
            GitConfigScope::Local => {
                // For local config, use the current directory's .git/config
                let git_dir = std::env::current_dir()
                    .map_err(|e| format!("Failed to get current directory: {}", e))?
                    .join(".git");
                
                if !git_dir.exists() {
                    return Err("Not in a git repository".to_string());
                }
                
                Ok(git_dir.join("config"))
            }
            GitConfigScope::Global => {
                // Use XDG config directory for global git config
                let config_dir = std::env::var("XDG_CONFIG_HOME")
                    .map(PathBuf::from)
                    .or_else(|_| {
                        std::env::var("HOME")
                            .map(|home| PathBuf::from(home).join(".config"))
                    })
                    .or_else(|_| {
                        std::env::var("USERPROFILE")
                            .map(|home| PathBuf::from(home).join(".config"))
                    })
                    .map_err(|_| "Unable to determine config directory".to_string())?;
                
                Ok(config_dir.join("git").join("config"))
            }
            GitConfigScope::System => {
                // System config location varies by OS
                #[cfg(unix)]
                {
                    Ok(PathBuf::from("/etc/gitconfig"))
                }
                #[cfg(windows)]
                {
                    let program_data = std::env::var("PROGRAMDATA")
                        .map_err(|_| "Unable to determine system config location".to_string())?;
                    Ok(PathBuf::from(program_data).join("Git").join("config"))
                }
            }
        }
    }

    fn get_source(&self) -> Source {
        match self.scope {
            GitConfigScope::Local => Source::Local,
            GitConfigScope::Global => Source::User,
            GitConfigScope::System => Source::System,
        }
    }
}

impl Atom for GitConfig {
    fn name(&self) -> &str {
        "GitConfig"
    }

    fn execute(&self) -> Result<(), String> {
        let config_path = self.get_config_path()?;
        
        // Create a new config file from scratch with only the declared entries
        let mut config = File::new(Metadata::from(self.get_source()));

        // Group entries by section for better organization
        let mut sections: std::collections::HashMap<(String, Option<String>), Vec<&GitConfigEntry>> = std::collections::HashMap::new();
        
        for entry in &self.entries {
            // Parse the key into section, subsection, and key name
            // Handle special cases like credential.https://github.com.helper
            // Parse git config keys, handling special cases
            let parts: Vec<&str> = entry.key.split('.').collect();
            let (section_name, subsection_name) = match parts.as_slice() {
                [section, _key] => (section.to_string(), None),
                [section, subsection, _key] => (section.to_string(), Some(subsection.to_string())),
                _ => {
                    // Handle keys with more than 3 parts (e.g., credential.https://github.com.helper)
                    // In this case, everything between the first and last dot is the subsection
                    if parts.len() > 3 {
                        let section = parts[0].to_string();
                        let subsection = parts[1..parts.len()-1].join(".");
                        (section, Some(subsection))
                    } else {
                        return Err(format!("Invalid git config key format: {}", entry.key));
                    }
                }
            };
            
            sections.entry((section_name, subsection_name)).or_insert_with(Vec::new).push(entry);
        }

        // Build the config file with all declared entries
        for ((section_name, subsection_name), entries) in sections {
            let mut section = config
                .new_section(section_name.clone(), subsection_name.clone().map(|s| BString::from(s).into()))
                .map_err(|e| format!("Failed to create section [{}{}]: {}", 
                    section_name,
                    subsection_name.as_ref().map(|s| format!(" \"{}\"", s)).unwrap_or_default(),
                    e
                ))?;
            
            for entry in entries {
                // Extract the key name (last part after the last dot)
                let key_name = if let Some(last_dot) = entry.key.rfind('.') {
                    &entry.key[last_dot + 1..]
                } else {
                    return Err(format!("Invalid key format: {}", entry.key));
                };
                
                // For multi-valued keys (add=true), push multiple values
                if entry.add.unwrap_or(false) {
                    // In declarative mode, we still respect the add flag for multi-valued keys
                    section.push(
                        gix_config::parse::section::ValueName::try_from(key_name)
                            .map_err(|e| format!("Invalid key name '{}': {:?}", key_name, e))?,
                        Some(entry.value.as_bytes().into())
                    );
                } else {
                    // For single-valued keys, push once
                    section.push(
                        gix_config::parse::section::ValueName::try_from(key_name)
                            .map_err(|e| format!("Invalid key name '{}': {:?}", key_name, e))?,
                        Some(entry.value.as_bytes().into())
                    );
                }
            }
        }

        // Create parent directories if needed
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        // Write the config to file (this replaces the entire file)
        let mut buffer = Vec::new();
        config
            .write_to(&mut buffer)
            .map_err(|e| format!("Failed to serialize git config: {}", e))?;
        
        fs::write(&config_path, buffer)
            .map_err(|e| format!("Failed to write git config to {}: {}", config_path.display(), e))?;

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
        let entries = vec![GitConfigEntry {
            key: "user.name".to_string(),
            value: "Test User".to_string(),
            add: None,
        }];

        let git_config = GitConfig::new(entries.clone(), GitConfigScope::Global, false);

        assert_eq!(git_config.entries.len(), 1);
        assert_eq!(git_config.scope, GitConfigScope::Global);
        assert!(!git_config.unset);
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
        assert_eq!(
            git_config.describe(),
            "Set global git configuration (2 entries)"
        );

        let git_config_unset = GitConfig::new(vec![], GitConfigScope::Local, true);
        assert_eq!(
            git_config_unset.describe(),
            "Unset local git configuration (0 entries)"
        );
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