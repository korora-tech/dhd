use crate::{
    actions::Action,
    dag_executor::{DagExecutor, ExecutionSummary},
    error::{DhdError, Result},
    loader::LoadedModule,
};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;

pub struct ExecutionEngine {
    concurrency: usize,
    dry_run: bool,
}

impl ExecutionEngine {
    pub fn new(concurrency: usize, dry_run: bool) -> Self {
        Self {
            concurrency,
            dry_run,
        }
    }

    pub fn execute(&self, modules: Vec<LoadedModule>) -> Result<()> {
        let start = Instant::now();

        println!("🚀 Starting execution of {} modules", modules.len());

        // Planning phase
        let pb = ProgressBar::new(modules.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} Planning modules... [{bar:40.cyan/blue}] {pos}/{len}")
                .unwrap()
        );

        let mut dag = DagExecutor::new(self.concurrency);
        let mut total_atoms = 0;

        for module in modules {
            pb.set_message(format!("Planning {}", module.definition.name));

            for action in module.definition.actions {
                let atoms = action.plan(std::path::Path::new(&module.source.path).parent().unwrap());
                for atom in atoms {
                    total_atoms += 1;
                    dag.add_atom(atom);
                }
            }

            pb.inc(1);
        }

        pb.finish_with_message("Planning complete");

        // Build dependencies
        println!("📊 Building dependency graph...");
        dag.build_dependencies()?;

        // Execute
        println!("⚡ Executing {} atoms with {} parallel workers", total_atoms, self.concurrency);

        let summary = dag.execute(self.dry_run)?;

        // Report results
        let duration = start.elapsed();
        self.print_summary(&summary, duration);

        if !summary.failed.is_empty() {
            return Err(DhdError::AtomExecution(
                format!("{} atoms failed", summary.failed.len())
            ));
        }

        Ok(())
    }

    fn print_summary(&self, summary: &ExecutionSummary, duration: std::time::Duration) {
        println!("\n📋 Execution Summary:");
        println!("   Total atoms: {}", summary.total);
        println!("   ✅ Completed: {}", summary.completed);
        println!("   ⏭️  Skipped: {}", summary.skipped);
        println!("   ❌ Failed: {}", summary.failed.len());
        println!("   ⏱️  Duration: {:.2}s", duration.as_secs_f64());

        if !summary.failed.is_empty() {
            println!("\n❌ Failed atoms:");
            for (id, error) in &summary.failed {
                println!("   - {}: {}", id, error);
            }
        }
    }
}