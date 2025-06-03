use crate::{Result, modules::registry::ModuleRegistry};
use std::path::PathBuf;
use tracing::{error, info};

pub struct PlanResult {
    pub registry: ModuleRegistry,
    pub ordered_modules: Vec<String>, // Module IDs in execution order
}

pub fn execute(modules: Option<Vec<String>>, modules_path: Option<PathBuf>) -> Result<PlanResult> {
    info!("Generating execution plan...");

    let modules_dir = modules_path.unwrap_or_else(|| PathBuf::from("examples"));

    let mut registry = ModuleRegistry::new();

    // Load all module files from the directory
    let entries = std::fs::read_dir(&modules_dir)?;
    let mut loaded_modules = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("ts") {
            match registry.load_module(&path) {
                Ok(module_data) => {
                    if let Some(ref specific_modules) = modules {
                        // Only track if it's in the requested list
                        if specific_modules.contains(&module_data.id) {
                            info!("Loading module: {}", module_data.id);
                            loaded_modules.push(module_data.id.clone());
                        }
                    } else {
                        // Track all loaded modules
                        info!("Loading module: {}", module_data.id);
                        loaded_modules.push(module_data.id.clone());
                    }
                }
                Err(e) => {
                    error!("Failed to load module {:?}: {}", path, e);
                }
            }
        }
    }

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
