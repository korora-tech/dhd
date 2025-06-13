use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
        let mut entries = Vec::new();
        
        // Process each scope
        if let Some(global_config) = &self.global {
            if let Some(obj) = global_config.as_object() {
                let map: HashMap<String, HashMap<String, serde_json::Value>> = 
                    obj.iter()
                        .filter_map(|(k, v)| {
                            v.as_object().map(|o| {
                                let inner: HashMap<String, serde_json::Value> = 
                                    o.iter().map(|(k2, v2)| (k2.clone(), v2.clone())).collect();
                                (k.clone(), inner)
                            })
                        })
                        .collect();
                process_config_scope(&map, &mut entries, "global");
            }
        }
        
        if let Some(system_config) = &self.system {
            if let Some(obj) = system_config.as_object() {
                let map: HashMap<String, HashMap<String, serde_json::Value>> = 
                    obj.iter()
                        .filter_map(|(k, v)| {
                            v.as_object().map(|o| {
                                let inner: HashMap<String, serde_json::Value> = 
                                    o.iter().map(|(k2, v2)| (k2.clone(), v2.clone())).collect();
                                (k.clone(), inner)
                            })
                        })
                        .collect();
                process_config_scope(&map, &mut entries, "system");
            }
        }
        
        if let Some(local_config) = &self.local {
            if let Some(obj) = local_config.as_object() {
                let map: HashMap<String, HashMap<String, serde_json::Value>> = 
                    obj.iter()
                        .filter_map(|(k, v)| {
                            v.as_object().map(|o| {
                                let inner: HashMap<String, serde_json::Value> = 
                                    o.iter().map(|(k2, v2)| (k2.clone(), v2.clone())).collect();
                                (k.clone(), inner)
                            })
                        })
                        .collect();
                process_config_scope(&map, &mut entries, "local");
            }
        }
        
        // Create separate GitConfig actions for each scope
        let mut atoms: Vec<Box<dyn crate::atom::Atom>> = Vec::new();
        
        if self.global.is_some() {
            let global_entries: Vec<GitConfigEntry> = entries.iter()
                .filter(|(_, _, scope)| *scope == "global")
                .map(|(entry, _, _)| entry.clone())
                .collect();
            
            if !global_entries.is_empty() {
                let scope = crate::atoms::git_config::GitConfigScope::Global;
                for entry in global_entries {
                    atoms.push(Box::new(AtomCompat::new(
                        Box::new(crate::atoms::git_config::GitConfig::new(
                            vec![crate::atoms::git_config::GitConfigEntry {
                                key: entry.key,
                                value: entry.value,
                                add: entry.add,
                            }],
                            scope.clone(),
                            false,
                        )),
                        "git_config".to_string(),
                    )));
                }
            }
        }
        
        if self.system.is_some() {
            let system_entries: Vec<GitConfigEntry> = entries.iter()
                .filter(|(_, _, scope)| *scope == "system")
                .map(|(entry, _, _)| entry.clone())
                .collect();
            
            if !system_entries.is_empty() {
                let scope = crate::atoms::git_config::GitConfigScope::System;
                for entry in system_entries {
                    atoms.push(Box::new(AtomCompat::new(
                        Box::new(crate::atoms::git_config::GitConfig::new(
                            vec![crate::atoms::git_config::GitConfigEntry {
                                key: entry.key,
                                value: entry.value,
                                add: entry.add,
                            }],
                            scope.clone(),
                            false,
                        )),
                        "git_config".to_string(),
                    )));
                }
            }
        }
        
        if self.local.is_some() {
            let local_entries: Vec<GitConfigEntry> = entries.iter()
                .filter(|(_, _, scope)| *scope == "local")
                .map(|(entry, _, _)| entry.clone())
                .collect();
            
            if !local_entries.is_empty() {
                let scope = crate::atoms::git_config::GitConfigScope::Local;
                for entry in local_entries {
                    atoms.push(Box::new(AtomCompat::new(
                        Box::new(crate::atoms::git_config::GitConfig::new(
                            vec![crate::atoms::git_config::GitConfigEntry {
                                key: entry.key,
                                value: entry.value,
                                add: entry.add,
                            }],
                            scope.clone(),
                            false,
                        )),
                        "git_config".to_string(),
                    )));
                }
            }
        }
        
        atoms
    }
}

/// Process a configuration scope and collect entries
fn process_config_scope(
    config: &HashMap<String, HashMap<String, serde_json::Value>>,
    entries: &mut Vec<(GitConfigEntry, bool, &'static str)>,
    scope: &'static str,
) {
    for (section_name, section) in config {
        for (key, value) in section {
            let full_key = if key.is_empty() {
                section_name.clone()
            } else {
                format!("{}.{}", section_name, key)
            };
            
            match value {
                serde_json::Value::String(val) => {
                    entries.push((
                        GitConfigEntry {
                            key: full_key,
                            value: val.clone(),
                            add: None,
                        },
                        false,
                        scope,
                    ));
                }
                serde_json::Value::Bool(val) => {
                    entries.push((
                        GitConfigEntry {
                            key: full_key,
                            value: val.to_string(),
                            add: None,
                        },
                        false,
                        scope,
                    ));
                }
                serde_json::Value::Number(val) => {
                    entries.push((
                        GitConfigEntry {
                            key: full_key,
                            value: val.to_string(),
                            add: None,
                        },
                        false,
                        scope,
                    ));
                }
                serde_json::Value::Array(values) => {
                    // For arrays, we need to set each one with add=true
                    for val in values {
                        if let Some(string_val) = val.as_str() {
                            entries.push((
                                GitConfigEntry {
                                    key: full_key.clone(),
                                    value: string_val.to_string(),
                                    add: Some(true),
                                },
                                true,
                                scope,
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
    }
}


#[typescript_fn]
pub fn gitConfig(config: GitConfig) -> crate::actions::ActionType {
    crate::actions::ActionType::GitConfig(config)
}