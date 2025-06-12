use crate::{
    actions::Action,
    dag_executor::{DagExecutor, ExecutionSummary},
    error::{DhdError, Result},
    loader::LoadedModule,
};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;

thread_local! {
    pub static VERBOSE_MODE: std::cell::RefCell<bool> = std::cell::RefCell::new(false);
}

pub struct ExecutionEngine {
    concurrency: usize,
    dry_run: bool,
    verbose: bool,
}

impl ExecutionEngine {
    pub fn new(concurrency: usize, dry_run: bool, verbose: bool) -> Self {
        Self {
            concurrency,
            dry_run,
            verbose,
        }
    }

    pub fn execute(&self, modules: Vec<LoadedModule>) -> Result<()> {
        let start = Instant::now();

        println!("🚀 Starting execution of {} modules", modules.len());

        // Planning phase
        let mut dag = DagExecutor::new(self.concurrency);
        let mut total_atoms = 0;

        // Set verbose mode for the planning phase
        VERBOSE_MODE.with(|v| *v.borrow_mut() = self.verbose);

        if self.verbose {
            println!("📋 Planning modules with verbose output...\n");
            
            for module in modules {
                println!("● Planning module: {}", module.definition.name);
                
                // Check module-level condition if present
                let should_execute = if let Some(condition) = &module.definition.when {
                    match condition.evaluate() {
                        Ok(result) => {
                            if !result {
                                println!("  ⏭️  Module skipped due to condition: {}", condition.describe());
                            }
                            result
                        }
                        Err(e) => {
                            eprintln!("  ❌ Error evaluating module condition: {}", e);
                            false
                        }
                    }
                } else {
                    true
                };

                if should_execute {
                    let module_atoms_before = total_atoms;
                    for action in module.definition.actions {
                        let atoms =
                            action.plan(std::path::Path::new(&module.source.path).parent().unwrap());
                        for atom in atoms {
                            total_atoms += 1;
                            dag.add_atom(atom);
                        }
                    }
                    
                    if total_atoms == module_atoms_before {
                        println!("  ⚠️  Module produced no atoms (all actions were skipped)\n");
                    } else {
                        println!("  ✓ Module produced {} atoms\n", total_atoms - module_atoms_before);
                    }
                } else {
                    println!("  ⏭️  Module skipped\n");
                }
            }
        } else {
            let pb = ProgressBar::new(modules.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} Planning modules... [{bar:40.cyan/blue}] {pos}/{len}")
                    .unwrap(),
            );

            for module in modules {
                pb.set_message(format!("Planning {}", module.definition.name));

                // Check module-level condition if present
                let should_execute = if let Some(condition) = &module.definition.when {
                    match condition.evaluate() {
                        Ok(result) => result,
                        Err(_) => false,
                    }
                } else {
                    true
                };

                if should_execute {
                    for action in module.definition.actions {
                        let atoms =
                            action.plan(std::path::Path::new(&module.source.path).parent().unwrap());
                        for atom in atoms {
                            total_atoms += 1;
                            dag.add_atom(atom);
                        }
                    }
                }

                pb.inc(1);
            }

            pb.finish_with_message("Planning complete");
        }

        // Build dependencies
        println!("📊 Building dependency graph...");
        dag.build_dependencies()?;

        // Execute
        println!(
            "⚡ Executing {} atoms with {} parallel workers",
            total_atoms, self.concurrency
        );

        let summary = dag.execute(self.dry_run)?;

        // Report results
        let duration = start.elapsed();
        self.print_summary(&summary, duration);

        if !summary.failed.is_empty() {
            return Err(DhdError::AtomExecution(format!(
                "{} atoms failed",
                summary.failed.len()
            )));
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
