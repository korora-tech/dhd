use crate::modules::loader::{ModuleAction as LoaderModuleAction, ModuleData as LoaderModuleData};
use crate::platform::PlatformInfo;
use crate::{DhdError, Result};
use oxc::allocator::Allocator;
use oxc::ast::ast::*;
use oxc::ast::{AstKind, Visit};
use oxc::parser::Parser;
use oxc::span::SourceType;
use std::collections::HashMap;
use std::path::Path;

struct ModuleData {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub dependencies: Vec<String>,
    pub actions: Vec<LoaderModuleAction>,
}

struct UserInfo {
    username: String,
    homedir: String,
}

struct ModuleVisitor<'a> {
    _allocator: &'a Allocator,
    module_data: ModuleData,
    current_path: Option<std::path::PathBuf>,
    in_with_callback: bool,
    actions: Vec<LoaderModuleAction>,
    platform: PlatformInfo,
    user: UserInfo,
}

impl<'a> ModuleVisitor<'a> {
    fn new(allocator: &'a Allocator, current_path: Option<std::path::PathBuf>) -> Self {
        let username = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| whoami::username());
        
        let homedir = dirs::home_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("/home/{}", username));
        
        Self {
            _allocator: allocator,
            module_data: ModuleData {
                name: String::new(),
                description: None,
                tags: Vec::new(),
                dependencies: Vec::new(),
                actions: Vec::new(),
            },
            current_path,
            in_with_callback: false,
            actions: Vec::new(),
            platform: PlatformInfo::current(),
            user: UserInfo { username, homedir },
        }
    }

    fn extract_string_literal(&self, expr: &Expression<'a>) -> Option<String> {
        match expr {
            Expression::StringLiteral(lit) => Some(lit.value.to_string()),
            Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
                template.quasis.first().map(|q| q.value.raw.to_string())
            }
            // Handle ctx.user and ctx.user.homedir
            Expression::Identifier(id) if id.name == "ctx" => {
                // Just ctx.user returns username
                Some(self.user.username.clone())
            }
            Expression::StaticMemberExpression(member) => {
                self.evaluate_ctx_expression(member)
            }
            Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
                // Handle string concatenation with +
                let left = self.extract_string_literal(&binary.left)?;
                let right = self.extract_string_literal(&binary.right)?;
                Some(format!("{}{}", left, right))
            }
            _ => None,
        }
    }
    
    fn evaluate_ctx_expression(&self, member: &StaticMemberExpression<'a>) -> Option<String> {
        // Check for ctx.user
        if let Expression::Identifier(id) = &member.object {
            if id.name == "ctx" && member.property.name == "user" {
                // ctx.user returns the username
                return Some(self.user.username.clone());
            }
        }
        
        // Check for ctx.user.homedir
        if let Expression::StaticMemberExpression(inner) = &member.object {
            if inner.property.name == "user" {
                if let Expression::Identifier(id) = &inner.object {
                    if id.name == "ctx" && member.property.name == "homedir" {
                        return Some(self.user.homedir.clone());
                    }
                }
            }
        }
        
        None
    }

    fn extract_array_of_strings(&self, expr: &Expression<'a>) -> Option<Vec<String>> {
        if let Expression::ArrayExpression(array) = expr {
            let mut strings = Vec::new();
            for element in &array.elements {
                match element {
                    ArrayExpressionElement::SpreadElement(_) => {
                        return None; // Can't handle spread elements statically
                    }
                    ArrayExpressionElement::Elision(_) => {
                        // Skip elisions (empty array elements)
                        continue;
                    }
                    _ => {
                        // For expression elements
                        if let Some(expr) = element.as_expression() {
                            if let Some(s) = self.extract_string_literal(expr) {
                                strings.push(s);
                            } else {
                                return None; // Non-string element
                            }
                        }
                    }
                }
            }
            Some(strings)
        } else {
            None
        }
    }

    fn extract_object_properties(&self, obj: &ObjectExpression<'a>) -> HashMap<String, String> {
        let mut props = HashMap::new();

        for prop in &obj.properties {
            match prop {
                ObjectPropertyKind::ObjectProperty(prop) => {
                    let key = match &prop.key {
                        PropertyKey::StaticIdentifier(id) => id.name.to_string(),
                        PropertyKey::StringLiteral(lit) => lit.value.to_string(),
                        _ => continue,
                    };

                    let value = match &prop.value {
                        Expression::StringLiteral(lit) => lit.value.to_string(),
                        Expression::BooleanLiteral(lit) => lit.value.to_string(),
                        Expression::NumericLiteral(lit) => lit.value.to_string(),
                        Expression::ArrayExpression(_array) => {
                            if let Some(strings) = self.extract_array_of_strings(&prop.value) {
                                strings.join(", ")
                            } else {
                                continue;
                            }
                        }
                        Expression::CallExpression(call) => {
                            // Check for ctx.platform.select() calls
                            if self.is_platform_select_call(call) {
                                if let Some(selected) = self.evaluate_platform_select(call) {
                                    selected.join(", ")
                                } else {
                                    continue;
                                }
                            } else {
                                continue;
                            }
                        }
                        Expression::StaticMemberExpression(member) => {
                            // Check for ctx.user or ctx.user.homedir
                            if let Some(value) = self.evaluate_ctx_expression(member) {
                                value
                            } else {
                                continue;
                            }
                        }
                        Expression::BinaryExpression(_) => {
                            // Handle string concatenation
                            if let Some(value) = self.extract_string_literal(&prop.value) {
                                value
                            } else {
                                continue;
                            }
                        }
                        _ => continue,
                    };

                    props.insert(key, value);
                }
                _ => continue,
            }
        }

        props
    }

    fn process_action_call(&mut self, name: &str, args: &[Argument<'a>]) {
        if args.is_empty() {
            return;
        }

        if let Some(expr) = args[0].as_expression() {
            if let Expression::ObjectExpression(obj) = expr {
                let props = self.extract_object_properties(obj);
                let params = self.convert_props_to_params(name, props);

                if !params.is_empty() {
                    self.actions.push(LoaderModuleAction {
                        action_type: name.to_string(),
                        params,
                    });
                }
            }
        }
    }

    fn convert_props_to_params(
        &self,
        action_type: &str,
        props: HashMap<String, String>,
    ) -> Vec<(String, String)> {
        let mut params = Vec::new();

        match action_type {
            "packageInstall" => {
                if let Some(names) = props.get("names") {
                    params.push(("packages".to_string(), names.clone()));
                }
                if let Some(manager) = props.get("manager") {
                    params.push(("manager".to_string(), manager.clone()));
                }
            }
            "executeCommand" => {
                if let Some(command) = props.get("command") {
                    params.push(("command".to_string(), command.clone()));
                }
                if let Some(args) = props.get("args") {
                    params.push(("args".to_string(), args.clone()));
                }
                if let Some(cwd) = props.get("cwd") {
                    params.push(("cwd".to_string(), cwd.clone()));
                }
                if let Some(shell) = props.get("shell") {
                    params.push(("shell".to_string(), shell.clone()));
                }
                if let Some(priv_esc) = props.get("privilegeEscalation") {
                    params.push(("privilege_escalation".to_string(), priv_esc.clone()));
                }
            }
            "userGroup" => {
                if let Some(user) = props.get("user") {
                    params.push(("user".to_string(), user.clone()));
                }
                if let Some(groups) = props.get("groups") {
                    params.push(("groups".to_string(), groups.clone()));
                }
                if let Some(append) = props.get("append") {
                    params.push(("append".to_string(), append.clone()));
                }
            }
            "linkDotfile" => {
                if let Some(source) = props.get("source") {
                    let resolved_source = self.resolve_path(source);
                    params.push(("source".to_string(), resolved_source));
                }
                if let Some(target) = props.get("target") {
                    params.push(("target".to_string(), target.clone()));
                }
                if let Some(backup) = props.get("backup") {
                    params.push(("backup".to_string(), backup.clone()));
                }
                if let Some(force) = props.get("force") {
                    params.push(("force".to_string(), force.clone()));
                }
            }
            "copyFile" => {
                if let Some(source) = props.get("source") {
                    let resolved_source = self.resolve_path(source);
                    params.push(("source".to_string(), resolved_source));
                }
                if let Some(dest) = props.get("destination") {
                    params.push(("destination".to_string(), dest.clone()));
                }
                if let Some(privileged) = props.get("privileged") {
                    params.push(("privileged".to_string(), privileged.clone()));
                }
                if let Some(mode) = props.get("mode") {
                    params.push(("mode".to_string(), mode.clone()));
                }
                if let Some(backup) = props.get("backup") {
                    params.push(("backup".to_string(), backup.clone()));
                }
            }
            "httpDownload" => {
                if let Some(url) = props.get("url") {
                    params.push(("url".to_string(), url.clone()));
                }
                if let Some(dest) = props.get("destination") {
                    params.push(("destination".to_string(), dest.clone()));
                }
                if let Some(checksum) = props.get("checksum") {
                    params.push(("checksum".to_string(), checksum.clone()));
                }
                if let Some(checksum_type) = props.get("checksumType") {
                    params.push(("checksum_type".to_string(), checksum_type.clone()));
                }
                if let Some(mode) = props.get("mode") {
                    params.push(("mode".to_string(), mode.clone()));
                }
                if let Some(privileged) = props.get("privileged") {
                    params.push(("privileged".to_string(), privileged.clone()));
                }
            }
            "fileWrite" => {
                if let Some(dest) = props.get("destination") {
                    params.push(("destination".to_string(), dest.clone()));
                }
                if let Some(content) = props.get("content") {
                    params.push(("content".to_string(), content.clone()));
                }
                if let Some(mode) = props.get("mode") {
                    params.push(("mode".to_string(), mode.clone()));
                }
                if let Some(privileged) = props.get("privileged") {
                    params.push(("privileged".to_string(), privileged.clone()));
                }
                if let Some(backup) = props.get("backup") {
                    params.push(("backup".to_string(), backup.clone()));
                }
            }
            "dconfImport" => {
                if let Some(source) = props.get("source") {
                    let resolved_source = self.resolve_path(source);
                    params.push(("source".to_string(), resolved_source));
                }
                if let Some(path) = props.get("path") {
                    params.push(("path".to_string(), path.clone()));
                }
                if let Some(backup) = props.get("backup") {
                    params.push(("backup".to_string(), backup.clone()));
                }
            }
            "systemdService" | "systemdSocket" => {
                if let Some(name) = props.get("name") {
                    params.push(("name".to_string(), name.clone()));
                }
                if let Some(content) = props.get("content") {
                    params.push(("content".to_string(), content.clone()));
                }
                if let Some(user) = props.get("user") {
                    params.push(("user".to_string(), user.clone()));
                }
                if let Some(enable) = props.get("enable") {
                    params.push(("enable".to_string(), enable.clone()));
                }
                if let Some(start) = props.get("start") {
                    params.push(("start".to_string(), start.clone()));
                }
                if let Some(reload) = props.get("reload") {
                    params.push(("reload".to_string(), reload.clone()));
                }
            }
            "gitConfig" => {
                if let Some(scope) = props.get("scope") {
                    params.push(("scope".to_string(), scope.clone()));
                }
                // Note: configs would need special handling as it's an object
                if let Some(configs) = props.get("configs") {
                    params.push(("configs".to_string(), configs.clone()));
                }
            }
            _ => {}
        }

        params
    }

    fn resolve_path(&self, path: &str) -> String {
        if path.starts_with('/') || path.starts_with('~') {
            path.to_string()
        } else if let Some(current_path) = &self.current_path {
            if let Some(parent) = current_path.parent() {
                parent.join(path).to_string_lossy().to_string()
            } else {
                path.to_string()
            }
        } else {
            path.to_string()
        }
    }

    fn process_module_builder_chain(&mut self, expr: &Expression<'a>) {
        if let Expression::CallExpression(call) = expr {
            // Check for method calls on the chain
            if let Expression::StaticMemberExpression(member) = &call.callee {
                // First, process the inner chain (recursively)
                self.process_module_builder_chain(&member.object);

                // Then process this method call
                match member.property.name.as_str() {
                    "description" => {
                        if let Some(arg) = call.arguments.first() {
                            if let Some(expr) = arg.as_expression() {
                                if let Some(desc) = self.extract_string_literal(expr) {
                                    self.module_data.description = Some(desc);
                                }
                            }
                        }
                    }
                    "depends" => {
                        for arg in &call.arguments {
                            if let Some(expr) = arg.as_expression() {
                                if let Some(dep) = self.extract_string_literal(expr) {
                                    self.module_data.dependencies.push(dep);
                                }
                            }
                        }
                    }
                    "tags" => {
                        for arg in &call.arguments {
                            if let Some(expr) = arg.as_expression() {
                                if let Some(tag) = self.extract_string_literal(expr) {
                                    self.module_data.tags.push(tag);
                                }
                            }
                        }
                    }
                    "with" => {
                        // Mark that we're in the with callback
                        self.in_with_callback = true;

                        // Process the callback function to extract actions
                        if let Some(arg) = call.arguments.first() {
                            if let Some(expr) = arg.as_expression() {
                                if let Expression::ArrowFunctionExpression(arrow) = expr {
                                    // Visit the function body to extract actions
                                    self.visit_function_body(&arrow.body);
                                }
                            }
                        }

                        self.in_with_callback = false;
                    }
                    _ => {}
                }
            }
        }
    }

    fn visit_function_body(&mut self, body: &FunctionBody<'a>) {
        for stmt in &body.statements {
            self.visit_statement(stmt);
        }
    }

    fn is_platform_select_call(&self, call: &CallExpression<'a>) -> bool {
        // Check if this is ctx.platform.select(...)
        if let Expression::StaticMemberExpression(member) = &call.callee {
            if member.property.name == "select" {
                if let Expression::StaticMemberExpression(inner) = &member.object {
                    if inner.property.name == "platform" {
                        if let Expression::Identifier(id) = &inner.object {
                            return id.name == "ctx";
                        }
                    }
                }
            }
        }
        false
    }

    fn evaluate_platform_select(&self, call: &CallExpression<'a>) -> Option<Vec<String>> {
        if call.arguments.is_empty() {
            return None;
        }

        if let Some(expr) = call.arguments[0].as_expression() {
            if let Expression::ObjectExpression(obj) = expr {
                return Some(self.select_platform_values(obj));
            }
        }

        None
    }

    fn select_platform_values(&self, obj: &ObjectExpression<'a>) -> Vec<String> {
        let mut default_value = Vec::new();
        let mut selected_value = None;

        for prop in &obj.properties {
            if let ObjectPropertyKind::ObjectProperty(prop) = prop {
                let key = match &prop.key {
                    PropertyKey::StaticIdentifier(id) => id.name.as_str(),
                    PropertyKey::StringLiteral(lit) => lit.value.as_str(),
                    _ => continue,
                };

                match key {
                    "default" => {
                        default_value = self.extract_package_values(&prop.value);
                    }
                    "windows" if self.platform.is_windows() => {
                        selected_value = Some(self.extract_package_values(&prop.value));
                    }
                    "mac" if self.platform.is_macos() => {
                        selected_value = Some(self.extract_package_values(&prop.value));
                    }
                    "linux" if self.platform.is_linux() => {
                        // Check if it's a simple value or LinuxOptions
                        if let Expression::ObjectExpression(linux_obj) = &prop.value {
                            selected_value = Some(self.select_linux_values(linux_obj));
                        } else {
                            selected_value = Some(self.extract_package_values(&prop.value));
                        }
                    }
                    _ => {}
                }
            }
        }

        selected_value.unwrap_or(default_value)
    }

    fn select_linux_values(&self, obj: &ObjectExpression<'a>) -> Vec<String> {
        let mut selected_value = None;

        for prop in &obj.properties {
            if let ObjectPropertyKind::ObjectProperty(prop) = prop {
                let key = match &prop.key {
                    PropertyKey::StaticIdentifier(id) => id.name.as_str(),
                    PropertyKey::StringLiteral(lit) => lit.value.as_str(),
                    _ => continue,
                };

                match key {
                    "distro" => {
                        if let Expression::ObjectExpression(distro_obj) = &prop.value {
                            // Check specific distros
                            if let Some(value) = self.get_matching_distro_value(distro_obj) {
                                selected_value = Some(value);
                            }
                        }
                    }
                    "family" => {
                        if selected_value.is_none() {
                            if let Expression::ObjectExpression(family_obj) = &prop.value {
                                // Check distro families
                                if let Some(value) = self.get_matching_family_value(family_obj) {
                                    selected_value = Some(value);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        selected_value.unwrap_or_default()
    }

    fn get_matching_distro_value(&self, obj: &ObjectExpression<'a>) -> Option<Vec<String>> {
        // Get the actual distribution name
        let distro = self.platform.os_name.to_lowercase();

        for prop in &obj.properties {
            if let ObjectPropertyKind::ObjectProperty(prop) = prop {
                let key = match &prop.key {
                    PropertyKey::StaticIdentifier(id) => id.name.as_str(),
                    PropertyKey::StringLiteral(lit) => lit.value.as_str(),
                    _ => continue,
                };

                // Match common distro names
                let matches = match key {
                    "ubuntu" => distro.contains("ubuntu"),
                    "debian" => distro.contains("debian"),
                    "fedora" => distro.contains("fedora"),
                    "arch" => distro.contains("arch"),
                    "manjaro" => distro.contains("manjaro"),
                    "opensuse" => distro.contains("opensuse") || distro.contains("suse"),
                    "nixos" => distro.contains("nixos"),
                    "rhel" => distro.contains("rhel") || distro.contains("red hat"),
                    "centos" => distro.contains("centos"),
                    _ => false,
                };

                if matches {
                    return Some(self.extract_package_values(&prop.value));
                }
            }
        }

        None
    }

    fn get_matching_family_value(&self, obj: &ObjectExpression<'a>) -> Option<Vec<String>> {
        // Determine Linux family based on distro
        let distro = self.platform.os_name.to_lowercase();
        let family = if distro.contains("debian") || distro.contains("ubuntu") {
            "debian"
        } else if distro.contains("fedora") || distro.contains("rhel") || distro.contains("centos")
        {
            "redhat"
        } else if distro.contains("suse") || distro.contains("opensuse") {
            "suse"
        } else {
            return None;
        };

        for prop in &obj.properties {
            if let ObjectPropertyKind::ObjectProperty(prop) = prop {
                let key = match &prop.key {
                    PropertyKey::StaticIdentifier(id) => id.name.as_str(),
                    PropertyKey::StringLiteral(lit) => lit.value.as_str(),
                    _ => continue,
                };

                if key == family {
                    return Some(self.extract_package_values(&prop.value));
                }
            }
        }

        None
    }

    fn extract_package_values(&self, expr: &Expression<'a>) -> Vec<String> {
        match expr {
            Expression::StringLiteral(lit) => vec![lit.value.to_string()],
            Expression::ArrayExpression(_) => {
                self.extract_array_of_strings(expr).unwrap_or_default()
            }
            _ => vec![],
        }
    }
}

impl<'a> Visit<'a> for ModuleVisitor<'a> {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        match kind {
            // Handle export default
            AstKind::ExportDefaultDeclaration(export) => {
                match &export.declaration {
                    _ => {
                        // Check if it's an expression we can process
                        if let Some(expr) = export.declaration.as_expression() {
                            // Process the entire module builder chain
                            if let Expression::CallExpression(_) = expr {
                                self.process_module_builder_chain(expr);
                            }
                        }
                    }
                }
            }

            // Handle defineModule calls
            AstKind::CallExpression(call) => {
                if let Expression::Identifier(id) = &call.callee {
                    if id.name == "defineModule" && !call.arguments.is_empty() {
                        if let Some(expr) = call.arguments[0].as_expression() {
                            if let Some(name) = self.extract_string_literal(expr) {
                                self.module_data.name = name;
                            }
                        }
                    }
                }

                // Handle action calls within the 'with' callback
                if self.in_with_callback {
                    if let Expression::Identifier(id) = &call.callee {
                        let action_name = id.name.as_str();
                        match action_name {
                            "packageInstall" | "executeCommand" | "userGroup" | "linkDotfile"
                            | "copyFile" | "httpDownload" | "fileWrite" | "dconfImport"
                            | "systemdService" | "systemdSocket" | "gitConfig" => {
                                self.process_action_call(action_name, &call.arguments);
                            }
                            _ => {}
                        }
                    }
                }
            }

            _ => {}
        }
    }

    fn leave_node(&mut self, kind: AstKind<'a>) {
        if let AstKind::CallExpression(call) = kind {
            if let Expression::StaticMemberExpression(member) = &call.callee {
                if member.property.name == "with" {
                    self.in_with_callback = false;
                    // Move collected actions to module data
                    self.module_data.actions = std::mem::take(&mut self.actions);
                }
            }
        }
    }
}

pub struct AstModuleLoader {
    allocator: Allocator,
}

impl AstModuleLoader {
    pub fn new() -> Self {
        Self {
            allocator: Allocator::default(),
        }
    }

    pub fn load_module(&self, path: &Path) -> Result<LoaderModuleData> {
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

        let mut visitor = ModuleVisitor::new(&self.allocator, Some(path.to_path_buf()));
        visitor.visit_program(&parse_result.program);

        // Convert internal ModuleData to LoaderModuleData
        Ok(LoaderModuleData {
            id: visitor.module_data.name.clone(),
            name: visitor.module_data.name,
            description: visitor.module_data.description,
            tags: visitor.module_data.tags,
            dependencies: visitor.module_data.dependencies,
            actions: visitor.module_data.actions,
        })
    }
}
