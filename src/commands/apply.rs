use crate::actions::{
    Action, ExecuteCommand, LinkDotfile as LinkDotfileAction,
    PackageInstall as PackageInstallAction, UserGroup,
};
use crate::atoms::{CopyFile, DconfImport, FileWrite, HttpDownload, SystemdService, SystemdSocket};
use crate::modules::loader::ModuleAction;
use crate::{Atom, DhdError, ExecutionPlan, Result, dag::DagExecutor};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{error, info, warn};

// Helper functions for parameter extraction
fn get_param_required(params: &[(String, String)], key: &str) -> Result<String> {
    params
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
        .ok_or_else(|| DhdError::AtomExecution(format!("Required parameter '{}' not found", key)))
}

fn get_param_optional(params: &[(String, String)], key: &str) -> Option<String> {
    params
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
}

fn get_param_bool(params: &[(String, String)], key: &str, default: bool) -> bool {
    get_param_optional(params, key)
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(default)
}

fn get_param_u32_octal(params: &[(String, String)], key: &str) -> Result<Option<u32>> {
    if let Some(mode_str) = get_param_optional(params, key) {
        u32::from_str_radix(&mode_str, 8).map(Some).map_err(|e| {
            DhdError::AtomExecution(format!("Invalid octal mode '{}': {}", mode_str, e))
        })
    } else {
        Ok(None)
    }
}

fn parse_env_params(params: &[(String, String)]) -> Option<HashMap<String, String>> {
    let env_params: HashMap<String, String> = params
        .iter()
        .filter(|(k, _)| k.starts_with("env."))
        .map(|(k, v)| (k.strip_prefix("env.").unwrap().to_string(), v.clone()))
        .collect();

    if env_params.is_empty() {
        None
    } else {
        Some(env_params)
    }
}

// Convert a ModuleAction to one or more Atoms
fn action_to_atoms(action: &ModuleAction) -> Result<Vec<Box<dyn Atom>>> {
    let params = &action.params;

    match action.action_type.as_str() {
        "copyFile" => {
            let source = get_param_required(params, "source")?;
            let destination = get_param_required(params, "destination")?;
            let privileged = get_param_bool(params, "privileged", false);
            let mode = get_param_u32_octal(params, "mode")?;
            let backup = get_param_bool(params, "backup", false);

            Ok(vec![Box::new(CopyFile::new(
                source,
                destination,
                privileged,
                mode,
                backup,
            ))])
        }
        "dconfImport" => {
            let source = get_param_required(params, "source")?;
            let path = get_param_required(params, "path")?;
            let backup = get_param_bool(params, "backup", false);

            Ok(vec![Box::new(DconfImport::new(source, path, backup))])
        }
        "fileWrite" => {
            let destination = get_param_required(params, "destination")?;
            let content = get_param_required(params, "content")?;
            let mode = get_param_u32_octal(params, "mode")?;
            let privileged = get_param_bool(params, "privileged", false);
            let backup = get_param_bool(params, "backup", false);

            Ok(vec![Box::new(FileWrite::new(
                destination,
                content,
                mode,
                privileged,
                backup,
            ))])
        }
        "httpDownload" => {
            let url = get_param_required(params, "url")?;
            let destination = get_param_required(params, "destination")?;
            let checksum = get_param_optional(params, "checksum");
            let checksum_type = get_param_optional(params, "checksum_type");
            let mode = get_param_u32_octal(params, "mode")?;
            let privileged = get_param_bool(params, "privileged", false);

            Ok(vec![Box::new(
                HttpDownload::new(url, destination, checksum, checksum_type, mode, privileged)
                    .map_err(|e| {
                        DhdError::AtomExecution(format!("Failed to create HttpDownload: {}", e))
                    })?,
            )])
        }
        "linkDotfile" => {
            let source = get_param_required(params, "source")?;
            let target = get_param_optional(params, "target");
            let backup = get_param_bool(params, "backup", false);
            let force = get_param_bool(params, "force", false);

            let action = LinkDotfileAction::new(source, target, backup, force);
            action.plan()
        }
        "packageInstall" => {
            let packages_str = get_param_required(params, "packages")?;
            let packages: Vec<String> = packages_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            let manager = get_param_optional(params, "manager");

            let action = PackageInstallAction::new(packages, manager);
            action.plan()
        }
        "executeCommand" => {
            let command = get_param_required(params, "command")?;
            let args = get_param_optional(params, "args")
                .map(|s| s.split_whitespace().map(String::from).collect())
                .unwrap_or_default();
            let cwd = get_param_optional(params, "cwd");
            let env = parse_env_params(params);
            let shell = get_param_optional(params, "shell");
            let privilege_escalation = get_param_bool(params, "privilege_escalation", false);

            let mut action = ExecuteCommand::new(command, args, cwd, env, shell);
            action.privilege_escalation = Some(privilege_escalation);
            action.plan()
        }
        "systemdService" => {
            let name = get_param_required(params, "name")?;
            let content = get_param_required(params, "content")?;
            let user = get_param_bool(params, "user", false);
            let enable = get_param_bool(params, "enable", false);
            let start = get_param_bool(params, "start", false);
            let reload = get_param_bool(params, "reload", false);

            Ok(vec![Box::new(SystemdService::new(
                name, content, user, enable, start, reload,
            ))])
        }
        "systemdSocket" => {
            let name = get_param_required(params, "name")?;
            let content = get_param_required(params, "content")?;
            let user = get_param_bool(params, "user", false);
            let enable = get_param_bool(params, "enable", false);
            let start = get_param_bool(params, "start", false);
            let reload = get_param_bool(params, "reload", false);

            Ok(vec![Box::new(SystemdSocket::new(
                name, content, user, enable, start, reload,
            ))])
        }
        "userGroup" => {
            let user = get_param_required(params, "user")?;
            let groups_str = get_param_required(params, "groups")?;
            let groups: Vec<String> = groups_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            let append = get_param_bool(params, "append", true);

            let action = UserGroup {
                user,
                groups,
                append: Some(append),
            };
            action.plan()
        }
        "gitConfig" => {
            let scope = get_param_required(params, "scope")?;
            let configs_str = get_param_required(params, "configs")?;

            // Parse the configs string (format: "key1=value1,key2=value2")
            let mut configs = HashMap::new();
            for pair in configs_str.split(',') {
                if let Some(eq_pos) = pair.find('=') {
                    let key = pair[..eq_pos].trim().to_string();
                    let value = pair[eq_pos + 1..].trim().to_string();
                    configs.insert(key, value);
                }
            }

            let scope = match scope.as_str() {
                "global" => crate::actions::GitConfigScope::Global,
                "system" => crate::actions::GitConfigScope::System,
                "local" => crate::actions::GitConfigScope::Local,
                _ => {
                    return Err(DhdError::AtomExecution(format!(
                        "Invalid git config scope: {}",
                        scope
                    )));
                }
            };

            let action = crate::actions::GitConfig { scope, configs };
            action.plan()
        }
        _ => Err(DhdError::AtomExecution(format!(
            "Unknown action type: {}",
            action.action_type
        ))),
    }
}

pub fn execute(
    modules: Option<Vec<String>>,
    modules_path: Option<PathBuf>,
    max_concurrent: usize,
    tags: Option<Vec<String>>,
) -> Result<()> {
    info!("Applying configuration...");

    // First run plan to get loaded modules
    let plan_result = crate::commands::plan::execute(modules, modules_path, tags)?;

    if plan_result.ordered_modules.is_empty() {
        info!("No modules to apply.");
        return Ok(());
    }

    // Build execution plan from module actions
    let mut nodes: Vec<Box<dyn Atom>> = Vec::new();
    let mut edges: Vec<(usize, usize)> = Vec::new();
    let mut module_action_indices: HashMap<String, Vec<usize>> = HashMap::new();

    let mut current_idx = 0;

    // Create atoms for each action in each module
    for module_id in &plan_result.ordered_modules {
        if let Some(module) = plan_result.registry.get(module_id) {
            info!(
                "Creating atoms for module: {} ({} actions)",
                module_id,
                module.actions.len()
            );

            let mut module_indices = Vec::new();
            let mut last_action_end_idx = None;

            for action in module.actions.iter() {
                match action_to_atoms(action) {
                    Ok(action_atoms) => {
                        let first_atom_idx = current_idx;
                        let has_atoms = !action_atoms.is_empty();

                        for atom in action_atoms {
                            info!(
                                "  Creating atom {} for action: {}",
                                current_idx, action.action_type
                            );
                            nodes.push(atom);
                            module_indices.push(current_idx);

                            // Add edges to enforce atom order within action
                            if current_idx > first_atom_idx {
                                edges.push((current_idx - 1, current_idx));
                            }

                            current_idx += 1;
                        }

                        // Add edges to enforce action order within module
                        if let Some(prev_end_idx) = last_action_end_idx {
                            edges.push((prev_end_idx, first_atom_idx));
                        }

                        if has_atoms {
                            last_action_end_idx = Some(current_idx - 1);
                        }
                    }
                    Err(e) => {
                        error!(
                            "Failed to create atoms for action {} in module {}: {}",
                            action.action_type, module_id, e
                        );
                        return Err(DhdError::AtomExecution(format!(
                            "Failed to create atoms: {}",
                            e
                        )));
                    }
                }
            }

            module_action_indices.insert(module_id.clone(), module_indices);
        }
    }

    // Build edges based on module dependencies
    for module_id in &plan_result.ordered_modules {
        if let Some(module) = plan_result.registry.get(module_id) {
            if let Some(module_indices) = module_action_indices.get(module_id) {
                for dep in &module.dependencies {
                    if let Some(dep_indices) = module_action_indices.get(dep) {
                        // Add edge from last action of dependency to first action of dependent
                        if !dep_indices.is_empty() && !module_indices.is_empty() {
                            edges.push((dep_indices[dep_indices.len() - 1], module_indices[0]));
                            info!("Adding dependency edge: {} -> {}", dep, module_id);
                        }
                    } else {
                        warn!(
                            "Module {} depends on {} which is not loaded",
                            module_id, dep
                        );
                    }
                }
            }
        }
    }

    let plan = ExecutionPlan { nodes, edges };

    let executor = DagExecutor::new(plan);
    executor.execute(max_concurrent)?;

    info!("Configuration applied successfully!");
    Ok(())
}
