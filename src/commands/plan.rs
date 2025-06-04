use crate::{Result, modules::registry::ModuleRegistry, utils};
use std::path::PathBuf;
use tracing::{error, info};

pub struct PlanResult {
    pub registry: ModuleRegistry,
    pub ordered_modules: Vec<String>, // Module IDs in execution order
}

pub fn execute(modules: Option<Vec<String>>, modules_path: Option<PathBuf>) -> Result<PlanResult> {
    info!("Generating execution plan...");

    let modules_dir = if let Some(path) = modules_path {
        utils::resolve_modules_directory(&path.to_string_lossy())?
    } else {
        // Default to current working directory
        std::env::current_dir()?
    };

    let mut registry = ModuleRegistry::new();

    // Load all modules recursively from the directory
    info!("Scanning for modules in: {:?}", modules_dir);
    match registry.load_modules_from_directory(&modules_dir) {
        Ok(loaded_count) => {
            info!("Found {} modules in directory tree", loaded_count);
        }
        Err(e) => {
            error!("Failed to load modules from directory: {}", e);
            return Err(e);
        }
    }

    // Filter to requested modules if specified
    let loaded_modules = if let Some(ref specific_modules) = modules {
        let mut found_modules = Vec::new();
        for module_id in specific_modules {
            if registry.get(module_id).is_some() {
                info!("Found requested module: {}", module_id);
                found_modules.push(module_id.clone());
            } else {
                error!(
                    "Requested module '{}' not found in directory tree",
                    module_id
                );
            }
        }
        found_modules
    } else {
        // Get all loaded module IDs
        registry.list_modules()
    };

    if loaded_modules.is_empty() {
        println!("No modules found or loaded.");
        return Ok(PlanResult {
            registry,
            ordered_modules: vec![],
        });
    }

    // Get modules in dependency order
    let ordered_modules = registry.get_ordered_modules(&loaded_modules)?;
    let ordered_module_ids: Vec<String> = ordered_modules.iter().map(|m| m.id.clone()).collect();

    // Display the execution plan
    println!("\nExecution Plan:");
    println!("================\n");

    if ordered_modules.is_empty() {
        println!("No modules to execute.");
    } else {
        let mut total_actions = 0;

        for (idx, module) in ordered_modules.iter().enumerate() {
            println!(
                "{}. Module: {} {}",
                idx + 1,
                module.id,
                if !module.dependencies.is_empty() {
                    format!("(depends on: {})", module.dependencies.join(", "))
                } else {
                    String::new()
                }
            );

            if let Some(desc) = &module.description {
                println!("   {}", desc);
            }

            if module.actions.is_empty() {
                println!("   └─ No actions defined");
            } else {
                println!("   Actions:");
                for (action_idx, action) in module.actions.iter().enumerate() {
                    let is_last = action_idx == module.actions.len() - 1;
                    let prefix = if is_last { "└─" } else { "├─" };

                    match action.action_type.as_str() {
                        "packageInstall" => {
                            let packages = action
                                .params
                                .iter()
                                .find(|(k, _)| k == "packages")
                                .map(|(_, v)| v.as_str())
                                .unwrap_or("?");
                            let manager = action
                                .params
                                .iter()
                                .find(|(k, _)| k == "manager")
                                .map(|(_, v)| format!(" via {}", v))
                                .unwrap_or_default();
                            println!("   {} Install packages{}: {}", prefix, manager, packages);
                        }
                        "linkDotfile" => {
                            let source = action
                                .params
                                .iter()
                                .find(|(k, _)| k == "source")
                                .map(|(_, v)| v.as_str())
                                .unwrap_or("?");
                            let target = action
                                .params
                                .iter()
                                .find(|(k, _)| k == "target")
                                .map(|(_, v)| format!(" → {}", v))
                                .unwrap_or_else(|| format!(" → $XDG_CONFIG_HOME/{}", source));
                            let options = action
                                .params
                                .iter()
                                .filter(|(k, v)| (k == "backup" || k == "force") && v == "true")
                                .map(|(k, _)| k.as_str())
                                .collect::<Vec<_>>()
                                .join(", ");
                            let options_str = if !options.is_empty() {
                                format!(" ({})", options)
                            } else {
                                String::new()
                            };
                            println!(
                                "   {} Link dotfile: {}{}{}",
                                prefix, source, target, options_str
                            );
                        }
                        "executeCommand" => {
                            let command = action
                                .params
                                .iter()
                                .find(|(k, _)| k == "command")
                                .map(|(_, v)| v.as_str())
                                .unwrap_or("?");
                            let args = action
                                .params
                                .iter()
                                .find(|(k, _)| k == "args")
                                .map(|(_, v)| format!(" {}", v))
                                .unwrap_or_default();
                            println!("   {} Execute: {}{}", prefix, command, args);
                        }
                        "copyFile" => {
                            let source = action
                                .params
                                .iter()
                                .find(|(k, _)| k == "source")
                                .map(|(_, v)| v.as_str())
                                .unwrap_or("?");
                            let destination = action
                                .params
                                .iter()
                                .find(|(k, _)| k == "destination")
                                .map(|(_, v)| v.as_str())
                                .unwrap_or("?");
                            let privileged = action
                                .params
                                .iter()
                                .any(|(k, v)| k == "privileged" && v == "true");
                            let backup = action
                                .params
                                .iter()
                                .any(|(k, v)| k == "backup" && v == "true");
                            let mut options = Vec::new();
                            if privileged {
                                options.push("privileged");
                            }
                            if backup {
                                options.push("backup");
                            }
                            let options_str = if !options.is_empty() {
                                format!(" ({})", options.join(", "))
                            } else {
                                String::new()
                            };
                            println!(
                                "   {} Copy file: {} → {}{}",
                                prefix, source, destination, options_str
                            );
                        }
                        "httpDownload" => {
                            let url = action
                                .params
                                .iter()
                                .find(|(k, _)| k == "url")
                                .map(|(_, v)| v.as_str())
                                .unwrap_or("?");
                            let destination = action
                                .params
                                .iter()
                                .find(|(k, _)| k == "destination")
                                .map(|(_, v)| v.as_str())
                                .unwrap_or("?");
                            let privileged = action
                                .params
                                .iter()
                                .any(|(k, v)| k == "privileged" && v == "true");
                            let suffix = if privileged { " (privileged)" } else { "" };
                            println!(
                                "   {} Download: {} → {}{}",
                                prefix, url, destination, suffix
                            );
                        }
                        "fileWrite" => {
                            let destination = action
                                .params
                                .iter()
                                .find(|(k, _)| k == "destination")
                                .map(|(_, v)| v.as_str())
                                .unwrap_or("?");
                            let privileged = action
                                .params
                                .iter()
                                .any(|(k, v)| k == "privileged" && v == "true");
                            let backup = action
                                .params
                                .iter()
                                .any(|(k, v)| k == "backup" && v == "true");
                            let mut options = Vec::new();
                            if privileged {
                                options.push("privileged");
                            }
                            if backup {
                                options.push("backup");
                            }
                            let options_str = if !options.is_empty() {
                                format!(" ({})", options.join(", "))
                            } else {
                                String::new()
                            };
                            println!("   {} Write file: {}{}", prefix, destination, options_str);
                        }
                        "dconfImport" => {
                            let source = action
                                .params
                                .iter()
                                .find(|(k, _)| k == "source")
                                .map(|(_, v)| v.as_str())
                                .unwrap_or("?");
                            let path = action
                                .params
                                .iter()
                                .find(|(k, _)| k == "path")
                                .map(|(_, v)| v.as_str())
                                .unwrap_or("?");
                            let backup = action
                                .params
                                .iter()
                                .any(|(k, v)| k == "backup" && v == "true");
                            let suffix = if backup { " (with backup)" } else { "" };
                            println!(
                                "   {} Import dconf: {} → {}{}",
                                prefix, source, path, suffix
                            );
                        }
                        "systemdService" => {
                            let name = action
                                .params
                                .iter()
                                .find(|(k, _)| k == "name")
                                .map(|(_, v)| v.as_str())
                                .unwrap_or("?");
                            let user = action
                                .params
                                .iter()
                                .any(|(k, v)| k == "user" && v == "true");
                            let mut options = Vec::new();
                            if user {
                                options.push("user");
                            }
                            if action
                                .params
                                .iter()
                                .any(|(k, v)| k == "enable" && v == "true")
                            {
                                options.push("enable");
                            }
                            if action
                                .params
                                .iter()
                                .any(|(k, v)| k == "start" && v == "true")
                            {
                                options.push("start");
                            }
                            if action
                                .params
                                .iter()
                                .any(|(k, v)| k == "reload" && v == "true")
                            {
                                options.push("reload");
                            }
                            let options_str = if !options.is_empty() {
                                format!(" ({})", options.join(", "))
                            } else {
                                String::new()
                            };
                            println!(
                                "   {} Create systemd service: {}{}",
                                prefix, name, options_str
                            );
                        }
                        "systemdSocket" => {
                            let name = action
                                .params
                                .iter()
                                .find(|(k, _)| k == "name")
                                .map(|(_, v)| v.as_str())
                                .unwrap_or("?");
                            let user = action
                                .params
                                .iter()
                                .any(|(k, v)| k == "user" && v == "true");
                            let mut options = Vec::new();
                            if user {
                                options.push("user");
                            }
                            if action
                                .params
                                .iter()
                                .any(|(k, v)| k == "enable" && v == "true")
                            {
                                options.push("enable");
                            }
                            if action
                                .params
                                .iter()
                                .any(|(k, v)| k == "start" && v == "true")
                            {
                                options.push("start");
                            }
                            if action
                                .params
                                .iter()
                                .any(|(k, v)| k == "reload" && v == "true")
                            {
                                options.push("reload");
                            }
                            let options_str = if !options.is_empty() {
                                format!(" ({})", options.join(", "))
                            } else {
                                String::new()
                            };
                            println!(
                                "   {} Create systemd socket: {}{}",
                                prefix, name, options_str
                            );
                        }
                        _ => {
                            println!("   {} {}: {:?}", prefix, action.action_type, action.params);
                        }
                    }
                    total_actions += 1;
                }
            }
            println!();
        }

        println!(
            "Summary: {} modules, {} actions",
            ordered_modules.len(),
            total_actions
        );
    }

    Ok(PlanResult {
        registry,
        ordered_modules: ordered_module_ids,
    })
}

// Keep a simpler version for CLI use that doesn't return the result
pub fn execute_and_display(
    modules: Option<Vec<String>>,
    modules_path: Option<PathBuf>,
) -> Result<()> {
    execute(modules, modules_path)?;
    Ok(())
}
