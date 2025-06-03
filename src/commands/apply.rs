use crate::{ExecutionPlan, Result, dag::DagExecutor};
use std::path::PathBuf;
use tracing::info;

pub fn execute(
    modules: Option<Vec<String>>,
    modules_path: Option<PathBuf>,
    max_concurrent: usize,
) -> Result<()> {
    info!("Applying configuration...");

    // First run plan
    crate::commands::plan::execute(modules, modules_path)?;

    // TODO: Build actual execution plan from modules
    let plan = ExecutionPlan {
        nodes: vec![],
        edges: vec![],
    };

    let executor = DagExecutor::new(plan);
    executor.execute(max_concurrent)?;

    info!("Configuration applied successfully!");
    Ok(())
}
