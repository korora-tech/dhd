use crate::Result;
use std::path::Path;

pub struct ModuleParser;

impl Default for ModuleParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_modules_dir(&self, _dir: &Path) -> Result<Vec<String>> {
        // TODO: Implement directory parsing
        Ok(vec![])
    }
}
