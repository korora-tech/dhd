use crate::{
    atom::Atom,
    error::{DhdError, Result},
};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

pub struct DagExecutor {
    graph: DiGraph<Box<dyn Atom>, ()>,
    node_map: HashMap<String, NodeIndex>,
    #[allow(dead_code)]
    concurrency: usize,
}

impl DagExecutor {
    pub fn new(concurrency: usize) -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
            concurrency,
        }
    }

    /// Add an atom to the execution graph
    pub fn add_atom(&mut self, atom: Box<dyn Atom>) {
        let id = atom.id();
        let node = self.graph.add_node(atom);
        self.node_map.insert(id, node);
    }

    /// Add a dependency between atoms
    pub fn add_dependency(&mut self, from_id: &str, to_id: &str) -> Result<()> {
        let from_node = self
            .node_map
            .get(from_id)
            .ok_or_else(|| DhdError::DependencyResolution(format!("Unknown atom: {}", from_id)))?;
        let to_node = self
            .node_map
            .get(to_id)
            .ok_or_else(|| DhdError::DependencyResolution(format!("Unknown atom: {}", to_id)))?;

        self.graph.add_edge(*from_node, *to_node, ());
        Ok(())
    }

    /// Build dependencies based on atom declarations
    pub fn build_dependencies(&mut self) -> Result<()> {
        // Collect all dependencies first to avoid borrowing issues
        let deps: Vec<(String, Vec<String>)> = self
            .graph
            .node_indices()
            .map(|idx| {
                let atom = &self.graph[idx];
                (atom.id(), atom.dependencies())
            })
            .collect();

        // Add edges for dependencies
        for (atom_id, atom_deps) in deps {
            for dep_id in atom_deps {
                self.add_dependency(&dep_id, &atom_id)?;
            }
        }

        Ok(())
    }

    /// Validate the graph (check for cycles)
    pub fn validate(&self) -> Result<()> {
        match toposort(&self.graph, None) {
            Ok(_) => Ok(()),
            Err(cycle) => {
                let atom = &self.graph[cycle.node_id()];
                Err(DhdError::DependencyResolution(format!(
                    "Circular dependency detected involving: {}",
                    atom.describe()
                )))
            }
        }
    }

    /// Execute all atoms in parallel respecting dependencies
    pub fn execute(&self, dry_run: bool) -> Result<ExecutionSummary> {
        self.validate()?;

        let sorted = toposort(&self.graph, None)
            .map_err(|_| DhdError::DependencyResolution("Failed to sort graph".to_string()))?;

        let multi_progress = MultiProgress::new();
        let completed = Arc::new(Mutex::new(HashSet::new()));
        let failed = Arc::new(Mutex::new(Vec::<(String, String)>::new()));
        let skipped = Arc::new(Mutex::new(Vec::new()));

        // Group atoms by their depth in the dependency graph
        let levels = self.group_by_levels(&sorted);

        // Execute each level in parallel
        for level in levels {
            let level_results: Vec<_> = level
                .into_par_iter()
                .map(|node_idx| {
                    let atom = &self.graph[node_idx];
                    let pb = self.create_progress_bar(&multi_progress, atom.as_ref());

                    let result = self.execute_atom(atom.as_ref(), dry_run, &pb);
                    pb.finish_and_clear();

                    (atom.id(), result)
                })
                .collect();

            // Process results
            for (id, result) in level_results {
                match result {
                    Ok(ExecutionResult::Executed) => {
                        completed.lock().unwrap().insert(id);
                    }
                    Ok(ExecutionResult::Skipped) => {
                        skipped.lock().unwrap().push(id);
                    }
                    Err(e) => {
                        failed.lock().unwrap().push((id, e.to_string()));
                    }
                }
            }

            // Stop if any failures occurred
            if !failed.lock().unwrap().is_empty() {
                break;
            }
        }

        Ok(ExecutionSummary {
            total: self.graph.node_count(),
            completed: completed.lock().unwrap().len(),
            skipped: skipped.lock().unwrap().len(),
            failed: failed.lock().unwrap().clone(),
        })
    }

    fn execute_atom(
        &self,
        atom: &dyn Atom,
        dry_run: bool,
        pb: &ProgressBar,
    ) -> Result<ExecutionResult> {
        pb.set_message(format!("Checking: {}", atom.describe()));

        // Check if atom needs to be executed
        match atom.check() {
            Ok(false) => {
                pb.set_message(format!("Skipped: {}", atom.describe()));
                return Ok(ExecutionResult::Skipped);
            }
            Err(e) => {
                return Err(DhdError::AtomExecution(format!(
                    "Check failed for {}: {}",
                    atom.describe(),
                    e
                )));
            }
            _ => {}
        }

        if dry_run {
            pb.set_message(format!("Would execute: {}", atom.describe()));
            return Ok(ExecutionResult::Executed);
        }

        pb.set_message(format!("Executing: {}", atom.describe()));

        atom.execute().map_err(|e| {
            DhdError::AtomExecution(format!("Execution failed for {}: {}", atom.describe(), e))
        })?;

        pb.set_message(format!("Completed: {}", atom.describe()));
        Ok(ExecutionResult::Executed)
    }

    fn group_by_levels(&self, sorted: &[NodeIndex]) -> Vec<Vec<NodeIndex>> {
        let mut levels = Vec::new();
        let mut node_levels = HashMap::new();

        for &node in sorted {
            let level = self
                .graph
                .neighbors_directed(node, petgraph::Direction::Incoming)
                .filter_map(|pred| node_levels.get(&pred))
                .max()
                .map(|&l| l + 1)
                .unwrap_or(0);

            node_levels.insert(node, level);

            if level >= levels.len() {
                levels.resize(level + 1, Vec::new());
            }
            levels[level].push(node);
        }

        levels
    }

    fn create_progress_bar(&self, multi: &MultiProgress, atom: &dyn Atom) -> ProgressBar {
        let pb = multi.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message(format!("Preparing: {}", atom.describe()));
        pb
    }
}

#[derive(Debug)]
pub struct ExecutionSummary {
    pub total: usize,
    pub completed: usize,
    pub skipped: usize,
    pub failed: Vec<(String, String)>,
}

#[derive(Debug)]
enum ExecutionResult {
    Executed,
    Skipped,
}
