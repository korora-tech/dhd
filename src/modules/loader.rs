use crate::{DhdError, Result};
use oxc::allocator::Allocator;
use oxc::ast::ast::Program;
use oxc::parser::Parser;
use oxc::span::SourceType;
use std::path::Path;

pub struct ModuleLoader {
    allocator: Allocator,
    current_path: Option<std::path::PathBuf>,
}

impl Default for ModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleLoader {
    pub fn new() -> Self {
        Self {
            allocator: Allocator::default(),
            current_path: None,
        }
    }

    pub fn load_module(&mut self, path: &Path) -> Result<ModuleData> {
        self.current_path = Some(path.to_path_buf());
        let source = std::fs::read_to_string(path)
            .map_err(|e| DhdError::ModuleParse(format!("Failed to read module: {}", e)))?;

        let source_type = SourceType::from_path(path)
            .map_err(|_| DhdError::ModuleParse("Invalid TypeScript file".to_string()))?;

        let parser = Parser::new(&self.allocator, &source, source_type);
        let parse_result = parser.parse();

        if !parse_result.errors.is_empty() {
            let errors: Vec<String> = parse_result.errors.iter().map(|e| e.to_string()).collect();
            return Err(DhdError::ModuleParse(errors.join("\n")));
        }

        self.extract_module_data(&parse_result.program)
    }

    fn extract_module_data(&self, _program: &Program) -> Result<ModuleData> {
        // For now, use simple string parsing to extract module information
        // TODO: Implement proper AST parsing with oxc
        let path = self
            .current_path
            .as_ref()
            .ok_or_else(|| DhdError::ModuleParse("No current path set".to_string()))?;
        let source = std::fs::read_to_string(path)?;

        let mut module_id = String::new();
        let mut description = None;
        let mut dependencies = Vec::new();

        // Extract module ID from defineModule("id")
        if let Some(start) = source.find("defineModule(\"") {
            let start = start + 14; // length of "defineModule(\""
            if let Some(end) = source[start..].find('"') {
                module_id = source[start..start + end].to_string();
            }
        }

        // Extract description from .description("desc")
        if let Some(start) = source.find(".description(\"") {
            let start = start + 14; // length of ".description(\""
            if let Some(end) = source[start..].find('"') {
                description = Some(source[start..start + end].to_string());
            }
        }

        // Extract dependencies from .depends("dep")
        let mut search_pos = 0;
        while let Some(start) = source[search_pos..].find(".depends(\"") {
            let abs_start = search_pos + start + 10; // length of ".depends(\""
            if let Some(end) = source[abs_start..].find('"') {
                dependencies.push(source[abs_start..abs_start + end].to_string());
                search_pos = abs_start + end;
            } else {
                break;
            }
        }

        if module_id.is_empty() {
            return Err(DhdError::ModuleParse(
                "Could not extract module ID".to_string(),
            ));
        }

        Ok(ModuleData {
            id: module_id.clone(),
            name: module_id,
            description,
            dependencies,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ModuleData {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub dependencies: Vec<String>,
}

/// Load all modules from a directory
pub fn load_modules_from_directory(path: impl AsRef<Path>) -> Result<Vec<ModuleData>> {
    let mut loader = ModuleLoader::new();
    let mut modules = Vec::new();

    let entries = std::fs::read_dir(path.as_ref())
        .map_err(|e| DhdError::ModuleParse(format!("Failed to read modules directory: {}", e)))?;

    for entry in entries {
        let entry =
            entry.map_err(|e| DhdError::ModuleParse(format!("Failed to read entry: {}", e)))?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("ts") {
            match loader.load_module(&path) {
                Ok(module_data) => modules.push(module_data),
                Err(e) => eprintln!("Failed to load module {:?}: {}", path, e),
            }
        }
    }

    Ok(modules)
}
