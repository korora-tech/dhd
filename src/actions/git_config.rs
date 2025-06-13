use serde::{Deserialize, Serialize};
use dhd_macros::{typescript_type, typescript_fn};

use crate::actions::Action;
use crate::atoms::AtomCompat;
use std::path::Path;

#[typescript_type]
pub struct GitConfigEntry {
    /// The configuration key (e.g., "user.name", "core.editor", "alias.co")
    pub key: String,
    /// The value for this key
    pub value: String,
    /// Whether to add this value (for multi-valued keys) instead of replacing
    pub add: Option<bool>,
}

/// Git configuration that accepts a nested object structure
#[derive(Serialize, Deserialize)]
#[typescript_type]
pub struct GitConfig {
    /// Global git configuration (~/.gitconfig)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global: Option<serde_json::Value>,
    
    /// System git configuration (/etc/gitconfig)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<serde_json::Value>,
    
    /// Local git configuration (repository-specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local: Option<serde_json::Value>,
}

impl Action for GitConfig {
    fn name(&self) -> &str {
        "GitConfig"
    }

    fn plan(&self, _module_dir: &Path) -> Vec<Box<dyn crate::atom::Atom>> {
        let mut atoms: Vec<Box<dyn crate::atom::Atom>> = Vec::new();
        
        // Process global config
        if let Some(global_config) = &self.global {
            let entries = process_config_value(global_config);
            if !entries.is_empty() {
                atoms.push(Box::new(AtomCompat::new(
                    Box::new(crate::atoms::git_config::GitConfig::new(
                        entries.into_iter().map(|e| crate::atoms::git_config::GitConfigEntry {
                            key: e.key,
                            value: e.value,
                            add: e.add,
                        }).collect(),
                        crate::atoms::git_config::GitConfigScope::Global,
                        false,
                    )),
                    "git_config".to_string(),
                )));
            }
        }
        
        // Process system config
        if let Some(system_config) = &self.system {
            let entries = process_config_value(system_config);
            if !entries.is_empty() {
                atoms.push(Box::new(AtomCompat::new(
                    Box::new(crate::atoms::git_config::GitConfig::new(
                        entries.into_iter().map(|e| crate::atoms::git_config::GitConfigEntry {
                            key: e.key,
                            value: e.value,
                            add: e.add,
                        }).collect(),
                        crate::atoms::git_config::GitConfigScope::System,
                        false,
                    )),
                    "git_config".to_string(),
                )));
            }
        }
        
        // Process local config
        if let Some(local_config) = &self.local {
            let entries = process_config_value(local_config);
            if !entries.is_empty() {
                atoms.push(Box::new(AtomCompat::new(
                    Box::new(crate::atoms::git_config::GitConfig::new(
                        entries.into_iter().map(|e| crate::atoms::git_config::GitConfigEntry {
                            key: e.key,
                            value: e.value,
                            add: e.add,
                        }).collect(),
                        crate::atoms::git_config::GitConfigScope::Local,
                        false,
                    )),
                    "git_config".to_string(),
                )));
            }
        }
        
        atoms
    }
}

/// Process a configuration value and return a flat list of entries
fn process_config_value(config: &serde_json::Value) -> Vec<GitConfigEntry> {
    let mut entries = Vec::new();
    
    if let Some(obj) = config.as_object() {
        for (section_name, section_value) in obj {
            process_section(section_name, section_value, &mut entries);
        }
    }
    
    entries
}

/// Process a configuration section recursively
fn process_section(prefix: &str, value: &serde_json::Value, entries: &mut Vec<GitConfigEntry>) {
    match value {
        serde_json::Value::Object(obj) => {
            // This is a nested section
            for (key, val) in obj {
                let full_key = if key.is_empty() {
                    prefix.to_string()
                } else {
                    format!("{}.{}", prefix, key)
                };
                process_section(&full_key, val, entries);
            }
        }
        serde_json::Value::String(val) => {
            entries.push(GitConfigEntry {
                key: prefix.to_string(),
                value: val.clone(),
                add: None,
            });
        }
        serde_json::Value::Bool(val) => {
            entries.push(GitConfigEntry {
                key: prefix.to_string(),
                value: val.to_string(),
                add: None,
            });
        }
        serde_json::Value::Number(val) => {
            entries.push(GitConfigEntry {
                key: prefix.to_string(),
                value: val.to_string(),
                add: None,
            });
        }
        serde_json::Value::Array(values) => {
            // For arrays, each value becomes a separate entry with add=true
            for val in values {
                if let Some(string_val) = val.as_str() {
                    entries.push(GitConfigEntry {
                        key: prefix.to_string(),
                        value: string_val.to_string(),
                        add: Some(true),
                    });
                }
            }
        }
        _ => {}
    }
}


#[typescript_fn]
pub fn git_config(config: GitConfig) -> crate::actions::ActionType {
    crate::actions::ActionType::GitConfig(config)
}