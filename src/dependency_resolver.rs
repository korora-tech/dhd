use crate::loader::LoadedModule;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, PartialEq)]
pub enum DependencyError {
    CyclicDependency(Vec<String>),
    MissingDependency { module: String, dependency: String },
}

impl std::fmt::Display for DependencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyError::CyclicDependency(cycle) => {
                write!(f, "Cyclic dependency detected: {}", cycle.join(" -> "))
            }
            DependencyError::MissingDependency { module, dependency } => {
                write!(
                    f,
                    "Module '{}' depends on '{}' which was not found",
                    module, dependency
                )
            }
        }
    }
}

impl std::error::Error for DependencyError {}

/// Resolves module dependencies and returns modules in execution order
pub fn resolve_dependencies(
    modules: Vec<LoadedModule>,
) -> Result<Vec<LoadedModule>, DependencyError> {
    // Create a map of module name to module
    let mut module_map: HashMap<String, LoadedModule> = HashMap::new();
    for module in modules {
        module_map.insert(module.definition.name.clone(), module);
    }

    // Build dependency graph
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    let mut in_degree: HashMap<String, usize> = HashMap::new();

    // Initialize graph
    for name in module_map.keys() {
        graph.insert(name.clone(), Vec::new());
        in_degree.insert(name.clone(), 0);
    }

    // Build edges and check for missing dependencies
    for (name, module) in &module_map {
        for dep in &module.definition.dependencies {
            if !module_map.contains_key(dep) {
                return Err(DependencyError::MissingDependency {
                    module: name.clone(),
                    dependency: dep.clone(),
                });
            }
            graph.get_mut(dep).unwrap().push(name.clone());
            *in_degree.get_mut(name).unwrap() += 1;
        }
    }

    // Topological sort using Kahn's algorithm
    let mut queue: VecDeque<String> = VecDeque::new();
    let mut result: Vec<LoadedModule> = Vec::new();

    // Find all nodes with no incoming edges
    for (name, &degree) in &in_degree {
        if degree == 0 {
            queue.push_back(name.clone());
        }
    }

    while let Some(current) = queue.pop_front() {
        result.push(module_map.get(&current).unwrap().clone());

        // For each node that depends on current
        if let Some(dependents) = graph.get(&current) {
            for dependent in dependents {
                let degree = in_degree.get_mut(dependent).unwrap();
                *degree -= 1;
                if *degree == 0 {
                    queue.push_back(dependent.clone());
                }
            }
        }
    }

    // Check for cycles
    if result.len() != module_map.len() {
        // Find a cycle for better error reporting
        let processed: HashSet<_> = result.iter().map(|m| &m.definition.name).collect();
        let unprocessed: Vec<_> = module_map
            .keys()
            .filter(|name| !processed.contains(name))
            .cloned()
            .collect();

        // Find a specific cycle
        if let Some(cycle) = find_cycle(&graph, &unprocessed) {
            return Err(DependencyError::CyclicDependency(cycle));
        }

        // Fallback error
        return Err(DependencyError::CyclicDependency(unprocessed));
    }

    Ok(result)
}

/// Find a cycle in the graph starting from the given nodes
fn find_cycle(graph: &HashMap<String, Vec<String>>, start_nodes: &[String]) -> Option<Vec<String>> {
    for start in start_nodes {
        let mut visited = HashSet::new();
        let mut path = Vec::new();

        if let Some(cycle) = dfs_find_cycle(start, graph, &mut visited, &mut path) {
            return Some(cycle);
        }
    }
    None
}

fn dfs_find_cycle(
    node: &str,
    graph: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    path: &mut Vec<String>,
) -> Option<Vec<String>> {
    if path.contains(&node.to_string()) {
        // Found a cycle
        let cycle_start = path.iter().position(|n| n == node).unwrap();
        let mut cycle = path[cycle_start..].to_vec();
        cycle.push(node.to_string());
        return Some(cycle);
    }

    if visited.contains(node) {
        return None;
    }

    visited.insert(node.to_string());
    path.push(node.to_string());

    if let Some(neighbors) = graph.get(node) {
        for neighbor in neighbors {
            if let Some(cycle) = dfs_find_cycle(neighbor, graph, visited, path) {
                return Some(cycle);
            }
        }
    }

    path.pop();
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::DiscoveredModule;
    use crate::module::ModuleDefinition;
    use std::path::PathBuf;

    fn create_test_module(name: &str, dependencies: Vec<String>) -> LoadedModule {
        LoadedModule {
            source: DiscoveredModule {
                path: PathBuf::from(format!("{}.ts", name)),
                name: name.to_string(),
            },
            definition: ModuleDefinition {
                name: name.to_string(),
                description: None,
                tags: vec![],
                dependencies,
                when: None,
                actions: vec![],
            },
        }
    }

    #[test]
    fn test_no_dependencies() {
        let modules = vec![
            create_test_module("a", vec![]),
            create_test_module("b", vec![]),
            create_test_module("c", vec![]),
        ];

        let result = resolve_dependencies(modules).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_simple_dependency_chain() {
        let modules = vec![
            create_test_module("app", vec!["lib".to_string()]),
            create_test_module("lib", vec!["base".to_string()]),
            create_test_module("base", vec![]),
        ];

        let result = resolve_dependencies(modules).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].definition.name, "base");
        assert_eq!(result[1].definition.name, "lib");
        assert_eq!(result[2].definition.name, "app");
    }

    #[test]
    fn test_diamond_dependency() {
        let modules = vec![
            create_test_module("app", vec!["lib1".to_string(), "lib2".to_string()]),
            create_test_module("lib1", vec!["base".to_string()]),
            create_test_module("lib2", vec!["base".to_string()]),
            create_test_module("base", vec![]),
        ];

        let result = resolve_dependencies(modules).unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].definition.name, "base");
        // lib1 and lib2 can be in any order
        assert!(result[1].definition.name == "lib1" || result[1].definition.name == "lib2");
        assert!(result[2].definition.name == "lib1" || result[2].definition.name == "lib2");
        assert_eq!(result[3].definition.name, "app");
    }

    #[test]
    fn test_cyclic_dependency() {
        let modules = vec![
            create_test_module("a", vec!["b".to_string()]),
            create_test_module("b", vec!["c".to_string()]),
            create_test_module("c", vec!["a".to_string()]),
        ];

        let result = resolve_dependencies(modules);
        assert!(result.is_err());

        match result.unwrap_err() {
            DependencyError::CyclicDependency(cycle) => {
                assert!(cycle.len() >= 3);
            }
            _ => panic!("Expected cyclic dependency error"),
        }
    }

    #[test]
    fn test_missing_dependency() {
        let modules = vec![create_test_module("app", vec!["missing".to_string()])];

        let result = resolve_dependencies(modules);
        assert!(result.is_err());

        match result.unwrap_err() {
            DependencyError::MissingDependency { module, dependency } => {
                assert_eq!(module, "app");
                assert_eq!(dependency, "missing");
            }
            _ => panic!("Expected missing dependency error"),
        }
    }

    #[test]
    fn test_complex_dependency_graph() {
        let modules = vec![
            create_test_module("app", vec!["ui".to_string(), "api".to_string()]),
            create_test_module("ui", vec!["common".to_string(), "theme".to_string()]),
            create_test_module("api", vec!["common".to_string(), "db".to_string()]),
            create_test_module("db", vec!["config".to_string()]),
            create_test_module("theme", vec!["config".to_string()]),
            create_test_module("common", vec!["utils".to_string()]),
            create_test_module("config", vec![]),
            create_test_module("utils", vec![]),
        ];

        let result = resolve_dependencies(modules).unwrap();
        assert_eq!(result.len(), 8);

        // Check that dependencies come before dependents
        let positions: HashMap<_, _> = result
            .iter()
            .enumerate()
            .map(|(i, m)| (m.definition.name.as_str(), i))
            .collect();

        assert!(positions["config"] < positions["db"]);
        assert!(positions["config"] < positions["theme"]);
        assert!(positions["utils"] < positions["common"]);
        assert!(positions["common"] < positions["ui"]);
        assert!(positions["common"] < positions["api"]);
        assert!(positions["ui"] < positions["app"]);
        assert!(positions["api"] < positions["app"]);
    }
}
