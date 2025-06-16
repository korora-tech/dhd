use crate::{
    actions::{Action, ActionType},
    dag_executor::{DagExecutor, ExecutionSummary},
    error::{DhdError, Result},
    loader::LoadedModule,
    secrets::{onepassword::OnePasswordProvider, SecretProvider, SecretResolver},
};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;
use tokio::runtime::Runtime;

thread_local! {
    pub static VERBOSE_MODE: std::cell::RefCell<bool> = std::cell::RefCell::new(false);
}

pub struct ExecutionEngine {
    concurrency: usize,
    dry_run: bool,
    verbose: bool,
    secret_provider: Option<Box<dyn SecretProvider>>,
}

impl ExecutionEngine {
    pub fn new(concurrency: usize, dry_run: bool, verbose: bool) -> Self {
        // Initialize with 1Password provider if available
        let secret_provider: Option<Box<dyn SecretProvider>> = if std::process::Command::new("op")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            Some(Box::new(OnePasswordProvider::new(None)))
        } else {
            None
        };

        Self {
            concurrency,
            dry_run,
            verbose,
            secret_provider,
        }
    }

    pub fn with_secret_provider(mut self, provider: Box<dyn SecretProvider>) -> Self {
        self.secret_provider = Some(provider);
        self
    }

    pub fn execute(&self, modules: Vec<LoadedModule>) -> Result<()> {
        let start = Instant::now();

        println!("🚀 Starting execution of {} modules", modules.len());

        // Planning phase
        let mut dag = DagExecutor::new(self.concurrency);
        let mut total_atoms = 0;

        // Set verbose mode for the planning phase
        VERBOSE_MODE.with(|v| *v.borrow_mut() = self.verbose);

        // Create a runtime for async operations if we have a secret provider
        let rt = if self.secret_provider.is_some() {
            Some(tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| DhdError::ExecutionEngine(format!("Failed to create async runtime: {}", e)))?)
        } else {
            None
        };

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
                        let atoms = self.plan_action_with_secrets(&action, &module.source.path.parent().unwrap_or(std::path::Path::new(".")), &rt)?;
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
                        let atoms = self.plan_action_with_secrets(&action, &module.source.path.parent().unwrap_or(std::path::Path::new(".")), &rt)?;
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

    fn plan_action_with_secrets(
        &self,
        action: &ActionType,
        module_dir: &std::path::Path,
        rt: &Option<Runtime>,
    ) -> Result<Vec<Box<dyn crate::atom::Atom>>> {
        // Check if this is an ExecuteCommand with environment variables that need secret resolution
        if let ActionType::ExecuteCommand(cmd) = action {
            if let Some(env) = &cmd.environment {
                if let (Some(provider), Some(runtime)) = (&self.secret_provider, rt) {
                    // Create a resolver for this action
                    let mut resolver = SecretResolver::new();
                    
                    // Resolve secrets
                    match runtime.block_on(resolver.resolve_map(env, provider.as_ref())) {
                        Ok(resolved_env) => {
                            // Create a modified command with resolved secrets
                            let mut modified_cmd = cmd.clone();
                            modified_cmd.environment = Some(resolved_env);
                            
                            // Plan with the modified command
                            return Ok(modified_cmd.plan(module_dir));
                        }
                        Err(e) => {
                            return Err(DhdError::ExecutionEngine(format!(
                                "Failed to resolve secrets: {}",
                                e
                            )));
                        }
                    }
                }
            }
        }
        
        // For all other actions or when no secrets need resolution
        Ok(action.plan(module_dir))
    }
}
