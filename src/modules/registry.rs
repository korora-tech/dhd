use crate::DhdError;
use crate::modules::loader::{ModuleData, ModuleLoader, load_modules_from_directory};
use std::collections::HashMap;
use std::path::Path;

pub struct ModuleRegistry {
    modules: HashMap<String, ModuleData>,
    loader: ModuleLoader,
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            loader: ModuleLoader::new(),
        }
    }

    pub fn load_module(&mut self, path: &Path) -> Result<&ModuleData, DhdError> {
        let module_data = self.loader.load_module(path)?;
        let id = module_data.id.clone();
        self.modules.insert(id.clone(), module_data);
        Ok(self.modules.get(&id).unwrap())
    }

    pub fn get(&self, id: &str) -> Option<&ModuleData> {
        self.modules.get(id)
    }

    pub fn list(&self) -> Vec<&ModuleData> {
        self.modules.values().collect()
    }

    pub fn list_modules(&self) -> Vec<String> {
        self.modules.keys().cloned().collect()
    }

    pub fn load_modules_from_directory(&mut self, path: &Path) -> Result<usize, DhdError> {
        let modules = load_modules_from_directory(path)?;
        let count = modules.len();

        for module_data in modules {
            let id = module_data.id.clone();
            use std::collections::hash_map::Entry;
            match self.modules.entry(id) {
                Entry::Occupied(entry) => {
                    tracing::warn!(
                        "Module '{}' already loaded, skipping duplicate",
                        entry.key()
                    );
                }
                Entry::Vacant(entry) => {
                    entry.insert(module_data);
                }
            }
        }

        Ok(count)
    }

    pub fn get_ordered_modules(&self, module_ids: &[String]) -> Result<Vec<&ModuleData>, DhdError> {
        let mut ordered = Vec::new();
        let mut visited = std::collections::HashSet::new();

        for id in module_ids {
            self.collect_dependencies(id, &mut ordered, &mut visited)?;
        }

        Ok(ordered)
    }

    fn collect_dependencies<'a>(
        &'a self,
        id: &str,
        ordered: &mut Vec<&'a ModuleData>,
        visited: &mut std::collections::HashSet<String>,
    ) -> Result<(), DhdError> {
        if visited.contains(id) {
            return Ok(());
        }

        let module = self
            .get(id)
            .ok_or_else(|| DhdError::ModuleParse(format!("Module {} not found", id)))?;

        // First add all dependencies
        for dep in &module.dependencies {
            self.collect_dependencies(dep, ordered, visited)?;
        }

        // Then add this module
        visited.insert(id.to_string());
        ordered.push(module);

        Ok(())
    }
}
