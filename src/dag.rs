use crate::{DhdError, ExecutionPlan, NodeId, Result};
use petgraph::algo::toposort;
use petgraph::graph::DiGraph;
use rayon::prelude::*;
use std::collections::HashMap;

pub struct DagExecutor {
    plan: ExecutionPlan,
}

impl DagExecutor {
    pub fn new(plan: ExecutionPlan) -> Self {
        Self { plan }
    }

    pub fn validate(&self) -> Result<()> {
        let mut graph = DiGraph::<NodeId, ()>::new();
        let mut node_map = HashMap::new();

        for (idx, _) in self.plan.nodes.iter().enumerate() {
            let node_idx = graph.add_node(idx);
            node_map.insert(idx, node_idx);
        }

        for (from, to) in &self.plan.edges {
            if let (Some(&from_idx), Some(&to_idx)) = (node_map.get(from), node_map.get(to)) {
                graph.add_edge(from_idx, to_idx, ());
            }
        }

        toposort(&graph, None).map_err(|_| DhdError::DagCycle)?;
        Ok(())
    }

    pub fn execute(&self, max_concurrent: usize) -> Result<()> {
        self.validate()?;

        tracing::info!(
            "Executing {} atoms with max concurrency of {}",
            self.plan.nodes.len(),
            max_concurrent
        );

        rayon::ThreadPoolBuilder::new()
            .num_threads(max_concurrent)
            .build()
            .map_err(|e| DhdError::AtomExecution(e.to_string()))?
            .install(|| {
                self.plan
                    .nodes
                    .par_iter()
                    .enumerate()
                    .try_for_each(|(idx, atom)| {
                        tracing::info!("Checking atom {}: {}", idx, atom.describe());
                        if atom.check()? {
                            tracing::info!("Executing atom {}: {}", idx, atom.describe());
                            atom.execute()?;
                            tracing::info!("Completed atom {}: {}", idx, atom.describe());
                        } else {
                            tracing::info!(
                                "Skipping atom {} (check returned false): {}",
                                idx,
                                atom.describe()
                            );
                        }
                        Ok(())
                    })
            })
    }
}
