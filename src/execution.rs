use crate::atom::Atom;
use crate::actions::Action;
use crate::loader::LoadedModule;
use crate::dependency_resolver::{resolve_dependencies, DependencyError};
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionMode {
    Execute,
    DryRun,
}

pub struct ExecutionPlan {
    pub atoms: Vec<(Box<dyn Atom>, String)>, // (atom, module_name)
    pub module_count: usize,
    pub action_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionResult {
    pub succeeded: usize,
    pub failed: usize,
    pub skipped: usize,
    pub errors: Vec<ExecutionError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionError {
    pub atom_name: String,
    pub module_name: String,
    pub error: String,
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} in module {}: {}", self.atom_name, self.module_name, self.error)
    }
}

impl std::error::Error for ExecutionError {}

pub struct ExecutionEngine {
    mode: ExecutionMode,
}

impl ExecutionEngine {
    pub fn new(mode: ExecutionMode) -> Self {
        Self { mode }
    }

    /// Create an execution plan from loaded modules
    pub fn plan(&self, modules: Vec<LoadedModule>) -> ExecutionPlan {
        let mut atoms: Vec<(Box<dyn Atom>, String)> = Vec::new();
        let mut action_count = 0;
        
        for module in &modules {
            // Get the module's directory for path resolution
            let module_dir = module.source.path.parent().unwrap_or_else(|| std::path::Path::new("."));
            
            for action in &module.definition.actions {
                let action_atoms = action.plan(module_dir);
                action_count += 1;
                // Associate each atom with the module name
                for atom in action_atoms {
                    atoms.push((atom, module.definition.name.clone()));
                }
            }
        }

        ExecutionPlan {
            atoms,
            module_count: modules.len(),
            action_count,
        }
    }
    
    /// Create an execution plan with dependency resolution
    pub fn plan_with_dependencies(&self, modules: Vec<LoadedModule>) -> Result<ExecutionPlan, DependencyError> {
        // Resolve dependencies first
        let resolved_modules = resolve_dependencies(modules)?;
        
        // Then create the plan with modules in correct order
        Ok(self.plan(resolved_modules))
    }

    /// Execute the plan and return results
    pub fn execute(&self, plan: ExecutionPlan) -> ExecutionResult {
        let mut result = ExecutionResult {
            succeeded: 0,
            failed: 0,
            skipped: 0,
            errors: Vec::new(),
        };

        match self.mode {
            ExecutionMode::DryRun => {
                // In dry-run mode, we just "execute" all atoms without actually doing anything
                result.succeeded = plan.atoms.len();
                println!("🔍 DRY RUN: Would execute {} atoms from {} modules ({} actions)", 
                    plan.atoms.len(), plan.module_count, plan.action_count);
                
                // Create progress bar for dry run
                let pb = ProgressBar::new(plan.atoms.len() as u64);
                pb.set_style(
                    ProgressStyle::default_bar()
                        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>3}/{len:3} {msg}")
                        .unwrap()
                        .progress_chars("█▉▊▋▌▍▎▏ ")
                );
                
                for (i, (atom, module_name)) in plan.atoms.iter().enumerate() {
                    pb.set_message(format!("{} ({})", atom.describe(), module_name));
                    pb.set_position(i as u64);
                    
                }
                
                pb.set_position(plan.atoms.len() as u64);
                pb.finish_with_message("✓ Dry run completed");
            }
            ExecutionMode::Execute => {
                println!("● Executing {} atoms from {} modules ({} actions)\n", 
                    plan.atoms.len(), plan.module_count, plan.action_count);
                
                // Group atoms by module for better organization
                let mut atoms_by_module: std::collections::HashMap<String, Vec<(usize, &Box<dyn Atom>)>> = std::collections::HashMap::new();
                for (i, (atom, module_name)) in plan.atoms.iter().enumerate() {
                    atoms_by_module.entry(module_name.clone()).or_insert_with(Vec::new).push((i, atom));
                }
                
                // Process each module
                for (module_idx, (module_name, atoms)) in atoms_by_module.iter().enumerate() {
                    println!("● {} ({} actions)", module_name, atoms.len());
                    
                    for (atom_idx, (_original_idx, atom)) in atoms.iter().enumerate() {
                        let is_last = atom_idx == atoms.len() - 1;
                        let prefix = if is_last { "  ⎿" } else { "  ├" };
                        
                        print!("{}  ", prefix);
                        std::io::Write::flush(&mut std::io::stdout()).unwrap();
                        
                        match atom.execute() {
                            Ok(_) => {
                                result.succeeded += 1;
                                println!("✓ {}", atom.describe());
                            }
                            Err(e) => {
                                result.failed += 1;
                                let error = ExecutionError {
                                    atom_name: atom.id(),
                                    module_name: module_name.clone(),
                                    error: e.to_string(),
                                };
                                result.errors.push(error.clone());
                                println!("✗ {} - {}", atom.describe(), e);
                            }
                        }
                    }
                    
                    if module_idx < atoms_by_module.len() - 1 {
                        println!();
                    }
                }
                
                // Summary
                println!("\n● Execution Summary");
                println!("  ├ Succeeded: {}", result.succeeded);
                if result.failed > 0 {
                    println!("  ├ Failed: {}", result.failed);
                }
                if result.skipped > 0 {
                    println!("  ⎿ Skipped: {}", result.skipped);
                } else {
                    println!("  ⎿ Completed");
                }
                
                if !result.errors.is_empty() {
                    println!("\n● Errors");
                    for (idx, error) in result.errors.iter().enumerate() {
                        let prefix = if idx == result.errors.len() - 1 { "  ⎿" } else { "  ├" };
                        println!("{} {}", prefix, error);
                    }
                }
            }
        }

        result
    }

    /// Execute modules directly (convenience method)
    pub fn execute_modules(&self, modules: Vec<LoadedModule>) -> ExecutionResult {
        let plan = self.plan(modules);
        self.execute(plan)
    }
    
    /// Execute modules with dependency resolution
    pub fn execute_modules_with_dependencies(&self, modules: Vec<LoadedModule>) -> Result<ExecutionResult, DependencyError> {
        let plan = self.plan_with_dependencies(modules)?;
        Ok(self.execute(plan))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::{ActionType, PackageInstall, LinkDotfile, ExecuteCommand};
    use crate::discovery::DiscoveredModule;
    use crate::module::ModuleDefinition;
    use std::path::PathBuf;

    fn create_test_module(name: &str, actions: Vec<ActionType>) -> LoadedModule {
        LoadedModule {
            source: DiscoveredModule {
                path: PathBuf::from(format!("{}.ts", name)),
                name: name.to_string(),
            },
            definition: ModuleDefinition {
                name: name.to_string(),
                description: Some(format!("Test module {}", name)),
                tags: vec![],
                dependencies: vec![],
                actions,
            },
        }
    }

    #[test]
    fn test_execution_engine_new() {
        let engine = ExecutionEngine::new(ExecutionMode::Execute);
        assert_eq!(engine.mode, ExecutionMode::Execute);

        let dry_run_engine = ExecutionEngine::new(ExecutionMode::DryRun);
        assert_eq!(dry_run_engine.mode, ExecutionMode::DryRun);
    }

    #[test]
    fn test_plan_empty_modules() {
        let engine = ExecutionEngine::new(ExecutionMode::Execute);
        let plan = engine.plan(vec![]);
        
        assert_eq!(plan.atoms.len(), 0);
        assert_eq!(plan.module_count, 0);
        assert_eq!(plan.action_count, 0);
    }

    #[test]
    fn test_plan_single_module() {
        let engine = ExecutionEngine::new(ExecutionMode::Execute);
        let module = create_test_module("test", vec![
            ActionType::PackageInstall(PackageInstall { names: vec!["vim".to_string()], manager: None }),
        ]);
        
        let plan = engine.plan(vec![module]);
        
        assert_eq!(plan.atoms.len(), 1);
        assert_eq!(plan.module_count, 1);
        assert_eq!(plan.action_count, 1);
        // Check that the atom is associated with the correct module
        assert_eq!(plan.atoms[0].1, "test");
    }

    #[test]
    fn test_plan_multiple_modules() {
        let engine = ExecutionEngine::new(ExecutionMode::Execute);
        let modules = vec![
            create_test_module("module1", vec![
                ActionType::PackageInstall(PackageInstall { names: vec!["vim".to_string()], manager: None }),
                ActionType::LinkDotfile(LinkDotfile { 
                    from: ".vimrc".to_string(), 
                    to: "~/.vimrc".to_string(),
                    force: false
                }),
            ]),
            create_test_module("module2", vec![
                ActionType::ExecuteCommand(ExecuteCommand { 
                    shell: Some("bash".to_string()), 
                    command: "echo hello".to_string(),
                    args: None,
                    escalate: false,
                }),
            ]),
        ];
        
        let plan = engine.plan(modules);
        
        assert_eq!(plan.atoms.len(), 3); // 1 + 1 + 1 atoms
        assert_eq!(plan.module_count, 2);
        assert_eq!(plan.action_count, 3);
        // Check that atoms are associated with correct modules
        assert_eq!(plan.atoms[0].1, "module1"); // PackageInstall
        assert_eq!(plan.atoms[1].1, "module1"); // LinkDotfile  
        assert_eq!(plan.atoms[2].1, "module2"); // ExecuteCommand
    }

    #[test]
    fn test_dry_run_execution() {
        let engine = ExecutionEngine::new(ExecutionMode::DryRun);
        let module = create_test_module("test", vec![
            ActionType::PackageInstall(PackageInstall { names: vec!["vim".to_string()], manager: None }),
        ]);
        
        let result = engine.execute_modules(vec![module]);
        
        assert_eq!(result.succeeded, 1);
        assert_eq!(result.failed, 0);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_execution_result_display() {
        let error = ExecutionError {
            atom_name: "TestAtom".to_string(),
            module_name: "TestModule".to_string(),
            error: "Something went wrong".to_string(),
        };
        
        let display = format!("{}", error);
        assert_eq!(display, "TestAtom in module TestModule: Something went wrong");
    }

}