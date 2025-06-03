use crate::modules::{
    loader::{ModuleLoader, load_modules_from_directory},
    registry::ModuleRegistry,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub name: String,
    pub description: Option<String>,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlanResult {
    pub modules: Vec<ModulePlan>,
    pub total_actions: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModulePlan {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub dependencies: Vec<String>,
    pub actions: Vec<ActionInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionInfo {
    pub description: String,
    pub atoms: Vec<String>,
}

pub struct AppState {
    pub modules_path: PathBuf,
}

#[tauri::command]
pub async fn list_modules(state: State<'_, AppState>) -> Result<Vec<ModuleInfo>, String> {
    let module_data = load_modules_from_directory(&state.modules_path)
        .map_err(|e| format!("Failed to load modules: {}", e))?;

    let modules = module_data
        .into_iter()
        .map(|m| ModuleInfo {
            name: m.id,
            description: m.description,
            dependencies: m.dependencies,
            tags: vec![], // TODO: Extract tags from module
        })
        .collect();

    Ok(modules)
}

#[tauri::command]
pub async fn generate_plan(
    state: State<'_, AppState>,
    modules: Vec<String>,
) -> Result<PlanResult, String> {
    let _loader = ModuleLoader::new();
    let mut registry = ModuleRegistry::new();

    // Load all modules into the registry
    let entries = std::fs::read_dir(&state.modules_path)
        .map_err(|e| format!("Failed to read modules directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("ts") {
            match registry.load_module(&path) {
                Ok(_) => {}
                Err(e) => eprintln!("Failed to load module {:?}: {}", path, e),
            }
        }
    }

    // Get modules in dependency order
    let ordered_modules = registry
        .get_ordered_modules(&modules)
        .map_err(|e| format!("Failed to resolve dependencies: {}", e))?;

    // Build the plan result with placeholder actions for now
    let mut module_plans = Vec::new();
    let mut total_actions = 0;

    for module in ordered_modules {
        // For now, create placeholder actions
        let action_infos = vec![ActionInfo {
            description: format!("Setup {}", module.name),
            atoms: vec!["RunCommand".to_string()],
        }];
        total_actions += action_infos.len();

        module_plans.push(ModulePlan {
            id: module.id.clone(),
            name: module.name.clone(),
            description: module.description.clone(),
            dependencies: module.dependencies.clone(),
            actions: action_infos,
        });
    }

    Ok(PlanResult {
        modules: module_plans,
        total_actions,
    })
}

#[tauri::command]
pub async fn apply_modules(modules: Vec<String>) -> Result<String, String> {
    // TODO: Implement actual apply logic
    Ok(format!("Applied {} modules successfully", modules.len()))
}
