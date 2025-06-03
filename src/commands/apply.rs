use crate::{Atom, ExecutionPlan, Result, dag::DagExecutor};
use std::path::PathBuf;
use tracing::{info, warn};

// Simple atom implementation for module execution
struct ModuleAtom {
    module_id: String,
}

impl Atom for ModuleAtom {
    fn check(&self) -> Result<bool> {
        // For now, always execute
        Ok(true)
    }

    fn execute(&self) -> Result<()> {
        info!("Executing module: {}", self.module_id);
        // TODO: Actually parse and execute module actions
        // For now, just print a message
        println!("Module '{}' executed", self.module_id);
        Ok(())
    }

    fn describe(&self) -> String {
        format!("Execute module: {}", self.module_id)
    }
}

pub fn execute(
    modules: Option<Vec<String>>,
    modules_path: Option<PathBuf>,
    max_concurrent: usize,
) -> Result<()> {
    info!("Applying configuration...");

    // First run plan to get loaded modules
    let plan_result = crate::commands::plan::execute(modules, modules_path)?;

    if plan_result.ordered_modules.is_empty() {
        info!("No modules to apply.");
        return Ok(());
    }

    // Build execution plan from modules
    let mut nodes: Vec<Box<dyn Atom>> = Vec::new();
    let mut edges: Vec<(usize, usize)> = Vec::new();
    let mut module_node_map = std::collections::HashMap::new();

    // Create an atom for each module
    for (idx, module_id) in plan_result.ordered_modules.iter().enumerate() {
        if let Some(_module) = plan_result.registry.get(module_id) {
            info!("Creating atom for module: {}", module_id);

            let atom: Box<dyn Atom> = Box::new(ModuleAtom {
                module_id: module_id.clone(),
            });

            nodes.push(atom);
            module_node_map.insert(module_id.clone(), idx);
        }
    }

    // Build edges based on module dependencies
    for (idx, module_id) in plan_result.ordered_modules.iter().enumerate() {
        if let Some(module) = plan_result.registry.get(module_id) {
            for dep in &module.dependencies {
                if let Some(&dep_idx) = module_node_map.get(dep) {
                    // Add edge from dependency to dependent
                    edges.push((dep_idx, idx));
                    info!("Adding dependency edge: {} -> {}", dep, module_id);
                } else {
                    warn!(
                        "Module {} depends on {} which is not loaded",
                        module_id, dep
                    );
                }
            }
        }
    }

    let plan = ExecutionPlan { nodes, edges };

    let executor = DagExecutor::new(plan);
    executor.execute(max_concurrent)?;

    info!("Configuration applied successfully!");
    Ok(())
}
