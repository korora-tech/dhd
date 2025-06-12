use crate::actions::{
    ActionType, CopyFile, DconfImport, Directory, ExecuteCommand, GitConfig, HttpDownload,
    InstallGnomeExtensions, LinkDirectory, LinkFile, PackageInstall, PackageRemove, SystemdManage,
    SystemdService, SystemdSocket, git_config::GitConfigEntry, Condition, ComparisonOperator,
};
use crate::atoms::package::PackageManager;
use crate::discovery::DiscoveredModule;
use crate::module::ModuleDefinition;
use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::fs;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct LoadedModule {
    pub source: DiscoveredModule,
    pub definition: ModuleDefinition,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadError {
    IoError(String),
    ParseError(String),
    ValidationError(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::IoError(msg) => write!(f, "IO error: {}", msg),
            LoadError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            LoadError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for LoadError {}

pub fn load_module(discovered: &DiscoveredModule) -> Result<LoadedModule, LoadError> {
    // Read the file content
    let content = fs::read_to_string(&discovered.path)
        .map_err(|e| LoadError::IoError(format!("Failed to read file: {}", e)))?;

    if content.trim().is_empty() {
        return Err(LoadError::ParseError("Empty file".to_string()));
    }

    // Parse with oxc
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(&discovered.path).unwrap_or_default();
    let ret = Parser::new(&allocator, &content, source_type).parse();

    if !ret.errors.is_empty() {
        let error_msg = ret
            .errors
            .iter()
            .map(|e| format!("{:?}", e))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(LoadError::ParseError(format!(
            "Failed to parse TypeScript: {}",
            error_msg
        )));
    }

    let program = ret.program;

    // Look for default export
    let module_def = extract_module_definition(&program)
        .ok_or_else(|| LoadError::ValidationError("No valid export default found".to_string()))?;

    Ok(LoadedModule {
        source: discovered.clone(),
        definition: module_def,
    })
}

fn extract_module_definition(program: &Program) -> Option<ModuleDefinition> {
    // Look for two patterns:
    // 1. export default defineModule("name").description("...").actions([...])
    // 2. export default { name: "...", description: "...", actions: [...] }

    for stmt in &program.body {
        if let Statement::ExportDefaultDeclaration(export) = stmt {
            // Check if it's an expression
            if let Some(expr) = export.declaration.as_expression() {
                // First try the fluent API pattern
                if let Some(module) = parse_fluent_api(expr) {
                    return Some(module);
                }
                // Then try the object literal pattern
                if let Expression::ObjectExpression(obj) = expr {
                    return parse_object_literal(obj);
                }
            }
        }
    }

    None
}

fn parse_fluent_api(expr: &Expression) -> Option<ModuleDefinition> {
    // Parse defineModule("name").description("...").actions([...])
    // We expect the outermost expression to be the last method call in the chain

    let mut module_def = ModuleDefinition {
        name: String::new(),
        description: None,
        tags: Vec::new(),
        dependencies: Vec::new(),
        when: None,
        actions: Vec::new(),
    };

    // Start from the outermost call and work inward
    let mut current_expr = expr;
    let mut method_calls = Vec::new();

    // Collect all method calls in the chain
    while let Expression::CallExpression(call) = current_expr {
        // Get the method name
        if let Some(member) = call.callee.as_member_expression() {
            if let Some(prop_name) = get_property_name(member) {
                method_calls.push((prop_name, &call.arguments));
            }
            current_expr = member.object();
        } else if let Expression::Identifier(ident) = &call.callee {
            // This should be defineModule
            method_calls.push((ident.name.to_string(), &call.arguments));
            break;
        } else {
            break;
        }
    }

    // Process in reverse order (defineModule -> description -> actions)
    method_calls.reverse();

    // First should be defineModule
    if let Some((method_name, args)) = method_calls.first() {
        if method_name != "defineModule" {
            return None;
        }
        if args.len() == 1 {
            if let Some(Expression::StringLiteral(lit)) = args[0].as_expression() {
                module_def.name = lit.value.to_string();
            }
        }
    } else {
        return None;
    }

    // Process remaining method calls
    for (method_name, args) in method_calls.iter().skip(1) {
        match method_name.as_str() {
            "description" => {
                if args.len() == 1 {
                    if let Some(Expression::StringLiteral(lit)) = args[0].as_expression() {
                        module_def.description = Some(lit.value.to_string());
                    }
                }
            }
            "tags" => {
                // Parse tags - can be multiple string arguments
                for arg in args.iter() {
                    if let Some(Expression::StringLiteral(lit)) = arg.as_expression() {
                        module_def.tags.push(lit.value.to_string());
                    }
                }
            }
            "dependsOn" => {
                // Parse dependencies - can be an array or multiple string arguments
                for arg in args.iter() {
                    match arg.as_expression() {
                        Some(Expression::StringLiteral(lit)) => {
                            module_def.dependencies.push(lit.value.to_string());
                        }
                        Some(Expression::ArrayExpression(arr)) => {
                            for elem in &arr.elements {
                                if let Some(Expression::StringLiteral(lit)) = elem.as_expression() {
                                    module_def.dependencies.push(lit.value.to_string());
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            "when" => {
                if args.len() == 1 {
                    if let Some(expr) = args[0].as_expression() {
                        module_def.when = parse_condition_expr(expr);
                    }
                }
            }
            "actions" => {
                if args.len() == 1 {
                    if let Some(Expression::ArrayExpression(arr)) = args[0].as_expression() {
                        for (idx, elem) in arr.elements.iter().enumerate() {
                            if let Some(action_expr) = elem.as_expression() {
                                match parse_action_call(action_expr) {
                                    Ok(action) => module_def.actions.push(action),
                                    Err(err) => {
                                        eprintln!("⚠️  Warning: Failed to parse action at index {} in module '{}': {}", idx, module_def.name, err);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if module_def.name.is_empty() {
        None
    } else {
        Some(module_def)
    }
}

fn get_property_name(member: &MemberExpression) -> Option<String> {
    match member {
        MemberExpression::StaticMemberExpression(static_member) => {
            Some(static_member.property.name.to_string())
        }
        _ => None,
    }
}

fn parse_action_call(expr: &Expression) -> Result<ActionType, String> {
    // Parse packageInstall({ names: [...] }), linkDotfile({ ... }), etc.
    if let Expression::CallExpression(call) = expr {
        if let Expression::Identifier(ident) = &call.callee {
            let action_name = ident.name.as_str();

            if call.arguments.len() == 1 {
                if let Some(Expression::ObjectExpression(obj)) = call.arguments[0].as_expression() {
                    match action_name {
                        "packageInstall" => {
                            let names = get_string_array_prop(obj, "names")
                                .ok_or_else(|| format!("packageInstall requires 'names' array property"))?;
                            let manager = get_package_manager(obj, "manager");
                            return Ok(ActionType::PackageInstall(PackageInstall {
                                names,
                                manager,
                            }));
                        }
                        "linkDotfile" | "linkFile" => {
                            // Support both old linkDotfile and new linkFile names
                            let source = get_string_prop(obj, "source")
                                .or_else(|| get_string_prop(obj, "from"))
                                .ok_or_else(|| format!("linkFile requires 'source' or 'from' property"))?;
                            let target = get_string_prop(obj, "target")
                                .or_else(|| get_string_prop(obj, "to"))
                                .ok_or_else(|| format!("linkFile requires 'target' or 'to' property"))?;
                            let force = get_bool_prop(obj, "force").unwrap_or(false);
                            return Ok(ActionType::LinkFile(LinkFile {
                                source,
                                target,
                                force,
                            }));
                        }
                        "linkDirectory" => {
                            let source = get_string_prop(obj, "source")
                                .or_else(|| get_string_prop(obj, "from"))
                                .ok_or_else(|| format!("linkDirectory requires 'source' or 'from' property"))?;
                            let target = get_string_prop(obj, "target")
                                .or_else(|| get_string_prop(obj, "to"))
                                .ok_or_else(|| format!("linkDirectory requires 'target' or 'to' property"))?;
                            let force = get_bool_prop(obj, "force").unwrap_or(false);
                            return Ok(ActionType::LinkDirectory(LinkDirectory {
                                source,
                                target,
                                force,
                            }));
                        }
                        "executeCommand" => {
                            let shell = get_string_prop(obj, "shell");
                            let command = get_string_prop(obj, "command")
                                .ok_or_else(|| format!("executeCommand requires 'command' property"))?;
                            let args = get_array_of_strings(obj, "args");
                            let escalate = get_bool_prop(obj, "escalate").unwrap_or(false);
                            return Ok(ActionType::ExecuteCommand(ExecuteCommand {
                                shell,
                                command,
                                args,
                                escalate,
                            }));
                        }
                        "copyFile" => {
                            let source = get_string_prop(obj, "source")
                                .ok_or_else(|| format!("copyFile requires 'source' property"))?;
                            let target = get_string_prop(obj, "target")
                                .or_else(|| get_string_prop(obj, "destination"))
                                .ok_or_else(|| format!("copyFile requires 'target' or 'destination' property"))?;
                            let escalate = get_bool_prop(obj, "escalate")
                                .or_else(|| get_bool_prop(obj, "requiresPrivilegeEscalation"))
                                .unwrap_or(false);
                            return Ok(ActionType::CopyFile(CopyFile {
                                source,
                                target,
                                escalate,
                            }));
                        }
                        "directory" => {
                            let path = get_string_prop(obj, "path")
                                .ok_or_else(|| format!("directory requires 'path' property"))?;
                            let escalate = get_bool_prop(obj, "escalate");
                            return Ok(ActionType::Directory(Directory { path, escalate }));
                        }
                        "httpDownload" => {
                            let url = get_string_prop(obj, "url")
                                .ok_or_else(|| format!("httpDownload requires 'url' property"))?;
                            let destination = get_string_prop(obj, "destination")
                                .ok_or_else(|| format!("httpDownload requires 'destination' property"))?;
                            let checksum = None; // TODO: Parse checksum object if provided
                            let mode = get_number_prop(obj, "mode").map(|n| n as u32);
                            return Ok(ActionType::HttpDownload(HttpDownload {
                                url,
                                destination,
                                checksum,
                                mode,
                            }));
                        }
                        "systemdService" => {
                            let name = get_string_prop(obj, "name")
                                .ok_or_else(|| format!("systemdService requires 'name' property"))?;
                            let description = get_string_prop(obj, "description")
                                .ok_or_else(|| format!("systemdService requires 'description' property"))?;
                            let exec_start = get_string_prop(obj, "execStart")
                                .ok_or_else(|| format!("systemdService requires 'execStart' property"))?;
                            let service_type = get_string_prop(obj, "serviceType")
                                .ok_or_else(|| format!("systemdService requires 'serviceType' property"))?;
                            let scope = get_string_prop(obj, "scope")
                                .ok_or_else(|| format!("systemdService requires 'scope' property"))?;
                            let restart = get_string_prop(obj, "restart");
                            let restart_sec = get_number_prop(obj, "restartSec").map(|n| n as u32);
                            return Ok(ActionType::SystemdService(SystemdService {
                                name,
                                description,
                                exec_start,
                                service_type,
                                scope,
                                restart,
                                restart_sec,
                            }));
                        }
                        "systemdSocket" => {
                            let name = get_string_prop(obj, "name")
                                .ok_or_else(|| format!("systemdSocket requires 'name' property"))?;
                            let description = get_string_prop(obj, "description")
                                .ok_or_else(|| format!("systemdSocket requires 'description' property"))?;
                            let listen_stream = get_string_prop(obj, "listenStream")
                                .ok_or_else(|| format!("systemdSocket requires 'listenStream' property"))?;
                            let scope = get_string_prop(obj, "scope")
                                .ok_or_else(|| format!("systemdSocket requires 'scope' property"))?;
                            return Ok(ActionType::SystemdSocket(SystemdSocket {
                                name,
                                description,
                                listen_stream,
                                scope,
                            }));
                        }
                        "systemdManage" => {
                            let name = get_string_prop(obj, "name")
                                .ok_or_else(|| format!("systemdManage requires 'name' property"))?;
                            let operation = get_string_prop(obj, "operation")
                                .ok_or_else(|| format!("systemdManage requires 'operation' property"))?;
                            let scope = get_string_prop(obj, "scope")
                                .ok_or_else(|| format!("systemdManage requires 'scope' property"))?;
                            return Ok(ActionType::SystemdManage(SystemdManage {
                                name,
                                operation,
                                scope,
                            }));
                        }
                        "gitConfig" => {
                            let entries = get_git_config_entries(obj, "entries")
                                .ok_or_else(|| format!("gitConfig requires 'entries' property"))?;
                            let global = get_bool_prop(obj, "global");
                            let system = get_bool_prop(obj, "system");
                            let unset = get_bool_prop(obj, "unset");
                            return Ok(ActionType::GitConfig(GitConfig {
                                entries,
                                global,
                                system,
                                unset,
                            }));
                        }
                        _ => {
                            return Err(format!("Unknown action type: '{}'. Available actions: packageInstall, linkFile, linkDirectory, executeCommand, copyFile, directory, httpDownload, systemdService, systemdSocket, systemdManage, packageRemove, dconfImport, installGnomeExtensions, gitConfig", action_name));
                        }
                    }
                } else {
                    return Err(format!("Action '{}' requires an object as argument", action_name));
                }
            } else {
                return Err(format!("Action '{}' requires exactly one argument", action_name));
            }
        } else {
            return Err(format!("Invalid action call: expected function call"));
        }
    } else {
        return Err(format!("Invalid expression: expected function call"));
    }
}

fn get_string_prop(obj: &ObjectExpression, key: &str) -> Option<String> {
    for prop in &obj.properties {
        if let ObjectPropertyKind::ObjectProperty(prop) = prop {
            let prop_key = match &prop.key {
                PropertyKey::StaticIdentifier(ident) => ident.name.as_str(),
                PropertyKey::StringLiteral(lit) => lit.value.as_str(),
                _ => continue,
            };

            if prop_key == key {
                if let Expression::StringLiteral(lit) = &prop.value {
                    return Some(lit.value.to_string());
                }
            }
        }
    }
    None
}

fn get_string_array_prop(obj: &ObjectExpression, key: &str) -> Option<Vec<String>> {
    for prop in &obj.properties {
        if let ObjectPropertyKind::ObjectProperty(prop) = prop {
            let prop_key = match &prop.key {
                PropertyKey::StaticIdentifier(ident) => ident.name.as_str(),
                PropertyKey::StringLiteral(lit) => lit.value.as_str(),
                _ => continue,
            };

            if prop_key == key {
                if let Expression::ArrayExpression(arr) = &prop.value {
                    let mut strings = Vec::new();
                    for elem in &arr.elements {
                        if let Some(Expression::StringLiteral(lit)) = elem.as_expression() {
                            strings.push(lit.value.to_string());
                        }
                    }
                    return Some(strings);
                }
            }
        }
    }
    None
}

fn get_array_of_strings(obj: &ObjectExpression, key: &str) -> Option<Vec<String>> {
    get_string_array_prop(obj, key)
}

fn get_package_manager(obj: &ObjectExpression, key: &str) -> Option<PackageManager> {
    let manager_str = get_string_prop(obj, key)?;
    PackageManager::from_str(&manager_str).ok()
}

fn get_bool_prop(obj: &ObjectExpression, key: &str) -> Option<bool> {
    for prop in &obj.properties {
        if let ObjectPropertyKind::ObjectProperty(prop) = prop {
            let prop_key = match &prop.key {
                PropertyKey::StaticIdentifier(ident) => ident.name.as_str(),
                PropertyKey::StringLiteral(lit) => lit.value.as_str(),
                _ => continue,
            };

            if prop_key == key {
                if let Expression::BooleanLiteral(lit) = &prop.value {
                    return Some(lit.value);
                }
            }
        }
    }
    None
}

fn get_number_prop(obj: &ObjectExpression, key: &str) -> Option<f64> {
    for prop in &obj.properties {
        if let ObjectPropertyKind::ObjectProperty(prop) = prop {
            let prop_key = match &prop.key {
                PropertyKey::StaticIdentifier(ident) => ident.name.as_str(),
                PropertyKey::StringLiteral(lit) => lit.value.as_str(),
                _ => continue,
            };

            if prop_key == key {
                if let Expression::NumericLiteral(lit) = &prop.value {
                    return Some(lit.value);
                }
            }
        }
    }
    None
}

fn parse_object_literal(obj: &ObjectExpression) -> Option<ModuleDefinition> {
    let mut name = None;
    let mut description = None;
    let mut tags = Vec::new();
    let mut dependencies = Vec::new();
    let mut actions = Vec::new();

    for prop in &obj.properties {
        if let ObjectPropertyKind::ObjectProperty(prop) = prop {
            let key = match &prop.key {
                PropertyKey::StaticIdentifier(ident) => ident.name.as_str(),
                PropertyKey::StringLiteral(lit) => lit.value.as_str(),
                _ => continue,
            };

            match key {
                "name" => {
                    if let Expression::StringLiteral(lit) = &prop.value {
                        name = Some(lit.value.to_string());
                    }
                }
                "description" => {
                    if let Expression::StringLiteral(lit) = &prop.value {
                        description = Some(lit.value.to_string());
                    }
                }
                "tags" => {
                    if let Expression::ArrayExpression(arr) = &prop.value {
                        for elem in &arr.elements {
                            if let Some(Expression::StringLiteral(lit)) = elem.as_expression() {
                                tags.push(lit.value.to_string());
                            }
                        }
                    }
                }
                "dependencies" => {
                    if let Expression::ArrayExpression(arr) = &prop.value {
                        for elem in &arr.elements {
                            if let Some(Expression::StringLiteral(lit)) = elem.as_expression() {
                                dependencies.push(lit.value.to_string());
                            }
                        }
                    }
                }
                "actions" => {
                    if let Expression::ArrayExpression(arr) = &prop.value {
                        for elem in &arr.elements {
                            if let Some(expr) = elem.as_expression() {
                                if let Some(action) = parse_action(expr) {
                                    actions.push(action);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    name.map(|n| ModuleDefinition {
        name: n,
        description,
        tags,
        dependencies,
        when: None,
        actions,
    })
}

fn parse_action(expr: &Expression) -> Option<ActionType> {
    // For JSON format: { type: "PackageInstall", names: [...] }
    if let Expression::ObjectExpression(obj) = expr {
        let mut action_type = None;
        let mut props = serde_json::Map::new();

        for prop in &obj.properties {
            if let ObjectPropertyKind::ObjectProperty(prop) = prop {
                let key = match &prop.key {
                    PropertyKey::StaticIdentifier(ident) => ident.name.as_str(),
                    PropertyKey::StringLiteral(lit) => lit.value.as_str(),
                    _ => continue,
                };

                if key == "type" {
                    if let Expression::StringLiteral(lit) = &prop.value {
                        action_type = Some(lit.value.to_string());
                    }
                } else {
                    // Convert expression to JSON value
                    if let Some(value) = expression_to_json(&prop.value) {
                        props.insert(key.to_string(), value);
                    }
                }
            }
        }

        // Create action based on type
        match action_type.as_deref() {
            Some("PackageInstall") => {
                if let Some(serde_json::Value::Array(names)) = props.get("names") {
                    let names: Vec<String> = names
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                    let manager = props
                        .get("manager")
                        .and_then(|v| v.as_str())
                        .and_then(|s| crate::atoms::package::PackageManager::from_str(s).ok());
                    return Some(ActionType::PackageInstall(PackageInstall {
                        names,
                        manager,
                    }));
                }
            }
            Some("LinkFile") => {
                let source = props
                    .get("source")
                    .and_then(|v| v.as_str())
                    .map(String::from)?;
                let target = props
                    .get("target")
                    .and_then(|v| v.as_str())
                    .map(String::from)?;
                let force = props
                    .get("force")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                return Some(ActionType::LinkFile(LinkFile {
                    source,
                    target,
                    force,
                }));
            }
            Some("LinkDirectory") => {
                let from = props
                    .get("from")
                    .and_then(|v| v.as_str())
                    .map(String::from)?;
                let to = props.get("to").and_then(|v| v.as_str()).map(String::from)?;
                let force = props
                    .get("force")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                return Some(ActionType::LinkDirectory(LinkDirectory {
                    source: from,
                    target: to,
                    force,
                }));
            }
            Some("ExecuteCommand") => {
                let shell = props
                    .get("shell")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let command = props
                    .get("command")
                    .and_then(|v| v.as_str())
                    .map(String::from)?;
                let args = props.get("args").and_then(|v| v.as_array()).map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                });
                let escalate = props
                    .get("escalate")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                return Some(ActionType::ExecuteCommand(ExecuteCommand {
                    shell,
                    command,
                    args,
                    escalate,
                }));
            }
            Some("CopyFile") => {
                let source = props
                    .get("source")
                    .and_then(|v| v.as_str())
                    .map(String::from)?;
                let target = props
                    .get("target")
                    .and_then(|v| v.as_str())
                    .map(String::from)?;
                let escalate = props
                    .get("escalate")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                return Some(ActionType::CopyFile(CopyFile {
                    source,
                    target,
                    escalate,
                }));
            }
            Some("Directory") => {
                let path = props
                    .get("path")
                    .and_then(|v| v.as_str())
                    .map(String::from)?;
                let escalate = props.get("escalate").and_then(|v| v.as_bool());
                return Some(ActionType::Directory(Directory { path, escalate }));
            }
            Some("HttpDownload") => {
                let url = props
                    .get("url")
                    .and_then(|v| v.as_str())
                    .map(String::from)?;
                let destination = props
                    .get("destination")
                    .and_then(|v| v.as_str())
                    .map(String::from)?;
                let checksum = None; // TODO: Parse checksum object if provided
                let mode = props.get("mode").and_then(|v| v.as_u64()).map(|n| n as u32);
                return Some(ActionType::HttpDownload(HttpDownload {
                    url,
                    destination,
                    checksum,
                    mode,
                }));
            }
            Some("SystemdService") => {
                let name = props
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(String::from)?;
                let description = props
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(String::from)?;
                let exec_start = props
                    .get("execStart")
                    .and_then(|v| v.as_str())
                    .map(String::from)?;
                let service_type = props
                    .get("serviceType")
                    .and_then(|v| v.as_str())
                    .map(String::from)?;
                let scope = props
                    .get("scope")
                    .and_then(|v| v.as_str())
                    .map(String::from)?;
                let restart = props
                    .get("restart")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let restart_sec = props
                    .get("restartSec")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as u32);
                return Some(ActionType::SystemdService(SystemdService {
                    name,
                    description,
                    exec_start,
                    service_type,
                    scope,
                    restart,
                    restart_sec,
                }));
            }
            Some("SystemdSocket") => {
                let name = props
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(String::from)?;
                let description = props
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(String::from)?;
                let listen_stream = props
                    .get("listenStream")
                    .and_then(|v| v.as_str())
                    .map(String::from)?;
                let scope = props
                    .get("scope")
                    .and_then(|v| v.as_str())
                    .map(String::from)?;
                return Some(ActionType::SystemdSocket(SystemdSocket {
                    name,
                    description,
                    listen_stream,
                    scope,
                }));
            }
            Some("DconfImport") => {
                let source = props
                    .get("source")
                    .and_then(|v| v.as_str())
                    .map(String::from)?;
                let path = props
                    .get("path")
                    .and_then(|v| v.as_str())
                    .map(String::from)?;
                return Some(ActionType::DconfImport(DconfImport { source, path }));
            }
            Some("InstallGnomeExtensions") => {
                if let Some(serde_json::Value::Array(extensions)) = props.get("extensions") {
                    let extensions: Vec<String> = extensions
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                    return Some(ActionType::InstallGnomeExtensions(InstallGnomeExtensions {
                        extensions,
                    }));
                }
            }
            Some("PackageRemove") => {
                if let Some(serde_json::Value::Array(names)) = props.get("names") {
                    let names: Vec<String> = names
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                    let manager = props
                        .get("manager")
                        .and_then(|v| v.as_str())
                        .and_then(|s| crate::atoms::package::PackageManager::from_str(s).ok());
                    return Some(ActionType::PackageRemove(PackageRemove { names, manager }));
                }
            }
            Some("SystemdManage") => {
                let name = props
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(String::from)?;
                let operation = props
                    .get("operation")
                    .and_then(|v| v.as_str())
                    .map(String::from)?;
                let scope = props
                    .get("scope")
                    .and_then(|v| v.as_str())
                    .map(String::from)?;
                return Some(ActionType::SystemdManage(SystemdManage {
                    name,
                    operation,
                    scope,
                }));
            }
            Some("GitConfig") => {
                if let Some(serde_json::Value::Array(entries_arr)) = props.get("entries") {
                    let entries: Vec<GitConfigEntry> = entries_arr
                        .iter()
                        .filter_map(|v| {
                            if let serde_json::Value::Object(entry) = v {
                                let key = entry.get("key")?.as_str()?.to_string();
                                let value = entry.get("value")?.as_str()?.to_string();
                                let add = entry.get("add").and_then(|v| v.as_bool());
                                Some(GitConfigEntry { key, value, add })
                            } else {
                                None
                            }
                        })
                        .collect();
                    let global = props.get("global").and_then(|v| v.as_bool());
                    let system = props.get("system").and_then(|v| v.as_bool());
                    let unset = props.get("unset").and_then(|v| v.as_bool());
                    return Some(ActionType::GitConfig(GitConfig {
                        entries,
                        global,
                        system,
                        unset,
                    }));
                }
            }
            _ => {}
        }
    }

    None
}

fn expression_to_json(expr: &Expression) -> Option<serde_json::Value> {
    match expr {
        Expression::StringLiteral(lit) => Some(serde_json::Value::String(lit.value.to_string())),
        Expression::NumericLiteral(lit) => Some(serde_json::Value::Number(
            serde_json::Number::from_f64(lit.value).unwrap_or_else(|| serde_json::Number::from(0)),
        )),
        Expression::BooleanLiteral(lit) => Some(serde_json::Value::Bool(lit.value)),
        Expression::NullLiteral(_) => Some(serde_json::Value::Null),
        Expression::ArrayExpression(arr) => {
            let values: Vec<serde_json::Value> = arr
                .elements
                .iter()
                .filter_map(|elem| elem.as_expression().and_then(expression_to_json))
                .collect();
            Some(serde_json::Value::Array(values))
        }
        Expression::ObjectExpression(obj) => {
            let mut map = serde_json::Map::new();
            for prop in &obj.properties {
                if let ObjectPropertyKind::ObjectProperty(prop) = prop {
                    let key = match &prop.key {
                        PropertyKey::StaticIdentifier(ident) => ident.name.as_str(),
                        PropertyKey::StringLiteral(lit) => lit.value.as_str(),
                        _ => continue,
                    };
                    if let Some(value) = expression_to_json(&prop.value) {
                        map.insert(key.to_string(), value);
                    }
                }
            }
            Some(serde_json::Value::Object(map))
        }
        _ => None,
    }
}

fn parse_condition_expr(expr: &Expression) -> Option<Condition> {
    // Parse condition builder expressions like:
    // property("hardware.fingerprint").isTrue()
    // command("lsusb").contains("fingerprint", true)
    // or([condition1, condition2])
    
    if let Expression::CallExpression(call) = expr {
        // Check if it's a method call on a builder
        if let Some(member) = call.callee.as_member_expression() {
            if let Some(method_name) = get_property_name(member) {
                // This is a builder method call like .isTrue(), .equals(), etc.
                return parse_condition_builder_method(member.object(), method_name.as_str(), &call.arguments);
            }
        }
        
        // Check if it's a direct function call
        if let Expression::Identifier(ident) = &call.callee {
            let func_name = ident.name.as_str();
            return parse_condition_function(func_name, &call.arguments);
        }
    }
    
    // For now, we don't support parsing string conditions from TypeScript
    // since we're walking the AST and can't evaluate complex expressions
    None
}

fn parse_condition_builder_method(builder_expr: &Expression, method: &str, args: &[Argument]) -> Option<Condition> {
    // First, identify what kind of builder this is
    if let Expression::CallExpression(builder_call) = builder_expr {
        if let Expression::Identifier(ident) = &builder_call.callee {
            match ident.name.as_str() {
                "property" => {
                    // property("path") builder
                    if builder_call.arguments.len() == 1 {
                        if let Some(Expression::StringLiteral(lit)) = builder_call.arguments[0].as_expression() {
                            let path = lit.value.to_string();
                            return match method {
                                "isTrue" => Some(Condition::SystemProperty {
                                    path,
                                    value: serde_json::Value::Bool(true),
                                    operator: ComparisonOperator::Equals,
                                }),
                                "isFalse" => Some(Condition::SystemProperty {
                                    path,
                                    value: serde_json::Value::Bool(false),
                                    operator: ComparisonOperator::Equals,
                                }),
                                "equals" => {
                                    if args.len() == 1 {
                                        if let Some(value) = parse_json_value(args[0].as_expression()?) {
                                            return Some(Condition::SystemProperty {
                                                path,
                                                value,
                                                operator: ComparisonOperator::Equals,
                                            });
                                        }
                                    }
                                    None
                                }
                                "contains" => {
                                    if args.len() == 1 {
                                        if let Some(Expression::StringLiteral(lit)) = args[0].as_expression() {
                                            return Some(Condition::SystemProperty {
                                                path,
                                                value: serde_json::Value::String(lit.value.to_string()),
                                                operator: ComparisonOperator::Contains,
                                            });
                                        }
                                    }
                                    None
                                }
                                _ => None,
                            };
                        }
                    }
                }
                "command" => {
                    // command("cmd") builder
                    if builder_call.arguments.len() == 1 {
                        if let Some(Expression::StringLiteral(lit)) = builder_call.arguments[0].as_expression() {
                            let cmd = lit.value.to_string();
                            return match method {
                                "exists" => Some(Condition::CommandExists { command: cmd }),
                                "succeeds" => Some(Condition::CommandSucceeds { command: cmd, args: None }),
                                "contains" => {
                                    if args.len() >= 1 {
                                        if let Some(Expression::StringLiteral(text_lit)) = args[0].as_expression() {
                                            let text = text_lit.value.to_string();
                                            let case_insensitive = if args.len() >= 2 {
                                                args[1].as_expression()
                                                    .and_then(|e| if let Expression::BooleanLiteral(b) = e {
                                                        Some(b.value)
                                                    } else {
                                                        None
                                                    })
                                                    .unwrap_or(false)
                                            } else {
                                                false
                                            };
                                            let grep_cmd = if case_insensitive {
                                                format!("{} | grep -qi '{}'", cmd, text)
                                            } else {
                                                format!("{} | grep -q '{}'", cmd, text)
                                            };
                                            return Some(Condition::CommandSucceeds {
                                                command: grep_cmd,
                                                args: None,
                                            });
                                        }
                                    }
                                    None
                                }
                                _ => None,
                            };
                        }
                    }
                }
                _ => {}
            }
        }
    }
    None
}

fn parse_condition_function(func_name: &str, args: &[Argument]) -> Option<Condition> {
    match func_name {
        "fileExists" => {
            if args.len() == 1 {
                if let Some(Expression::StringLiteral(lit)) = args[0].as_expression() {
                    return Some(Condition::FileExists { path: lit.value.to_string() });
                }
            }
        }
        "directoryExists" => {
            if args.len() == 1 {
                if let Some(Expression::StringLiteral(lit)) = args[0].as_expression() {
                    return Some(Condition::DirectoryExists { path: lit.value.to_string() });
                }
            }
        }
        "commandExists" => {
            if args.len() == 1 {
                if let Some(Expression::StringLiteral(lit)) = args[0].as_expression() {
                    return Some(Condition::CommandExists { command: lit.value.to_string() });
                }
            }
        }
        "envVar" => {
            if args.len() >= 1 {
                if let Some(Expression::StringLiteral(name_lit)) = args[0].as_expression() {
                    let name = name_lit.value.to_string();
                    let value = if args.len() >= 2 {
                        args[1].as_expression()
                            .and_then(|e| if let Expression::StringLiteral(lit) = e {
                                Some(lit.value.to_string())
                            } else {
                                None
                            })
                    } else {
                        None
                    };
                    return Some(Condition::EnvironmentVariable { name, value });
                }
            }
        }
        "or" | "anyOf" => {
            if args.len() == 1 {
                if let Some(Expression::ArrayExpression(arr)) = args[0].as_expression() {
                    let conditions: Vec<Condition> = arr.elements.iter()
                        .filter_map(|elem| elem.as_expression())
                        .filter_map(parse_condition_expr)
                        .collect();
                    if !conditions.is_empty() {
                        return Some(Condition::AnyOf { conditions });
                    }
                }
            }
        }
        "and" | "allOf" => {
            if args.len() == 1 {
                if let Some(Expression::ArrayExpression(arr)) = args[0].as_expression() {
                    let conditions: Vec<Condition> = arr.elements.iter()
                        .filter_map(|elem| elem.as_expression())
                        .filter_map(parse_condition_expr)
                        .collect();
                    if !conditions.is_empty() {
                        return Some(Condition::AllOf { conditions });
                    }
                }
            }
        }
        "not" => {
            if args.len() == 1 {
                if let Some(expr) = args[0].as_expression() {
                    if let Some(condition) = parse_condition_expr(expr) {
                        return Some(Condition::Not { condition: Box::new(condition) });
                    }
                }
            }
        }
        _ => {}
    }
    None
}

fn parse_json_value(expr: &Expression) -> Option<serde_json::Value> {
    match expr {
        Expression::StringLiteral(lit) => Some(serde_json::Value::String(lit.value.to_string())),
        Expression::NumericLiteral(lit) => Some(serde_json::Value::Number(
            serde_json::Number::from_f64(lit.value).unwrap_or_else(|| serde_json::Number::from(0))
        )),
        Expression::BooleanLiteral(lit) => Some(serde_json::Value::Bool(lit.value)),
        Expression::NullLiteral(_) => Some(serde_json::Value::Null),
        _ => None,
    }
}

fn get_git_config_entries(obj: &ObjectExpression, key: &str) -> Option<Vec<GitConfigEntry>> {
    for prop in &obj.properties {
        if let ObjectPropertyKind::ObjectProperty(prop) = prop {
            let prop_key = match &prop.key {
                PropertyKey::StaticIdentifier(ident) => ident.name.as_str(),
                PropertyKey::StringLiteral(lit) => lit.value.as_str(),
                _ => continue,
            };

            if prop_key == key {
                if let Expression::ArrayExpression(arr) = &prop.value {
                    let mut entries = Vec::new();
                    for elem in &arr.elements {
                        if let Some(Expression::ObjectExpression(entry_obj)) = elem.as_expression()
                        {
                            let key = get_string_prop(entry_obj, "key")?;
                            let value = get_string_prop(entry_obj, "value")?;
                            let add = get_bool_prop(entry_obj, "add");
                            entries.push(GitConfigEntry { key, value, add });
                        }
                    }
                    return Some(entries);
                }
            }
        }
    }
    None
}

pub fn load_modules(discovered: Vec<DiscoveredModule>) -> Vec<Result<LoadedModule, LoadError>> {
    discovered.iter().map(load_module).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    fn create_test_module(dir: &Path, name: &str, content: &str) -> DiscoveredModule {
        let path = dir.join(format!("{}.ts", name));
        let mut file = File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        DiscoveredModule {
            path,
            name: name.to_string(),
        }
    }

    #[test]
    fn test_load_module_simple() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
import { defineModule, packageInstall } from "./types";

export default defineModule("test")
    .description("Test module")
    .actions([
        packageInstall({ names: ["vim"] })
    ]);
"#;

        let discovered = create_test_module(temp_dir.path(), "test", content);
        let result = load_module(&discovered);

        assert!(result.is_ok());
        let loaded = result.unwrap();
        assert_eq!(loaded.source, discovered);
        assert_eq!(loaded.definition.name, "test");
        assert_eq!(
            loaded.definition.description,
            Some("Test module".to_string())
        );
        assert_eq!(loaded.definition.actions.len(), 1);
    }

    #[test]
    fn test_load_module_no_description() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
import { defineModule, packageInstall } from "./types";

export default defineModule("nodesc")
    .actions([
        packageInstall({ names: ["git"] })
    ]);
"#;

        let discovered = create_test_module(temp_dir.path(), "nodesc", content);
        let result = load_module(&discovered);

        assert!(result.is_ok());
        let loaded = result.unwrap();
        assert_eq!(loaded.definition.name, "nodesc");
        assert_eq!(loaded.definition.description, None);
        assert_eq!(loaded.definition.actions.len(), 1);
    }

    #[test]
    fn test_load_module_multiple_actions() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
import { defineModule, packageInstall, linkFile, executeCommand } from "./types";

export default defineModule("multi")
    .description("Multiple actions")
    .actions([
        packageInstall({ names: ["vim", "git"] }),
        linkFile({ source: ".vimrc", target: "~/.vimrc" }),
        executeCommand({ shell: "bash", command: "echo done" })
    ]);
"#;

        let discovered = create_test_module(temp_dir.path(), "multi", content);
        let result = load_module(&discovered);

        assert!(result.is_ok());
        let loaded = result.unwrap();
        assert_eq!(loaded.definition.actions.len(), 3);
    }

    #[test]
    fn test_load_module_invalid_syntax() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
export default {
    name: "invalid"
    description: "Missing comma"
    actions: []
};
"#;

        let discovered = create_test_module(temp_dir.path(), "invalid", content);
        let result = load_module(&discovered);

        assert!(result.is_err());
        match result.unwrap_err() {
            LoadError::ParseError(msg) => assert!(msg.contains("syntax") || msg.contains("parse")),
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_load_module_no_default_export() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
import { defineModule } from "./types";

const module = defineModule("noexport")
    .actions([]);
"#;

        let discovered = create_test_module(temp_dir.path(), "noexport", content);
        let result = load_module(&discovered);

        assert!(result.is_err());
        match result.unwrap_err() {
            LoadError::ValidationError(msg) => assert!(msg.contains("export default")),
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_load_module_file_not_found() {
        let discovered = DiscoveredModule {
            path: PathBuf::from("/nonexistent/file.ts"),
            name: "nonexistent".to_string(),
        };

        let result = load_module(&discovered);
        assert!(result.is_err());
        match result.unwrap_err() {
            LoadError::IoError(msg) => assert!(msg.contains("No such file")),
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_load_module_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let discovered = create_test_module(temp_dir.path(), "empty", "");

        let result = load_module(&discovered);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_module_json_format() {
        let temp_dir = TempDir::new().unwrap();
        // Some users might try to export JSON directly
        let content = r#"
export default {
    "name": "jsonmodule",
    "description": "JSON format",
    "actions": []
};
"#;

        let discovered = create_test_module(temp_dir.path(), "json", content);
        let result = load_module(&discovered);

        // We should accept this format too
        assert!(result.is_ok());
        let loaded = result.unwrap();
        assert_eq!(loaded.definition.name, "jsonmodule");
        assert_eq!(
            loaded.definition.description,
            Some("JSON format".to_string())
        );
    }

    #[test]
    fn test_load_modules_batch() {
        let temp_dir = TempDir::new().unwrap();

        let module1 = create_test_module(
            temp_dir.path(),
            "mod1",
            r#"
export default { name: "mod1", actions: [] };
"#,
        );

        let module2 = create_test_module(
            temp_dir.path(),
            "mod2",
            r#"
export default { name: "mod2", actions: [] };
"#,
        );

        let module3 = create_test_module(temp_dir.path(), "invalid", "invalid syntax");

        let results = load_modules(vec![module1, module2, module3]);

        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert!(results[2].is_err());
    }

    #[test]
    fn test_load_module_with_variables() {
        let temp_dir = TempDir::new().unwrap();
        // For now, we don't support variable references in our simple parser
        // This would require more complex AST traversal
        let content = r#"
export default {
    name: "tools",
    description: "Install dev tools",
    actions: [
        { type: "PackageInstall", names: ["vim", "git", "curl"] }
    ]
};
"#;

        let discovered = create_test_module(temp_dir.path(), "tools", content);
        let result = load_module(&discovered);

        assert!(result.is_ok());
        let loaded = result.unwrap();
        assert_eq!(
            loaded.definition.description,
            Some("Install dev tools".to_string())
        );
    }
}
