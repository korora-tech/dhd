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
        for (idx, module) in ordered_modules.iter().enumerate() {
            println!("{}. Module: {}", idx + 1, module.id);
            if let Some(desc) = &module.description {
                println!("   Description: {}", desc);
            }
            if !module.dependencies.is_empty() {
                println!("   Dependencies: {}", module.dependencies.join(", "));
            }
            println!();
        }

        println!("Total modules to execute: {}", ordered_modules.len());
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
