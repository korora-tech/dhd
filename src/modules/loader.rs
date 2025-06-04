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

        // Remove comments from source to avoid parsing them
        let source_without_comments = self.remove_comments(&source);

        let mut module_id = String::new();
        let mut description = None;
        let mut dependencies = Vec::new();
        let mut actions = Vec::new();

        // Extract module ID from defineModule("id")
        if let Some(start) = source_without_comments.find("defineModule(\"") {
            let start = start + 14; // length of "defineModule(\""
            if let Some(end) = source_without_comments[start..].find('"') {
                module_id = source_without_comments[start..start + end].to_string();
            }
        }

        // Extract description from .description("desc")
        if let Some(start) = source_without_comments.find(".description(\"") {
            let start = start + 14; // length of ".description(\""
            if let Some(end) = source_without_comments[start..].find('"') {
                description = Some(source_without_comments[start..start + end].to_string());
            }
        }

        // Extract dependencies from .depends("dep")
        let mut search_pos = 0;
        while let Some(start) = source_without_comments[search_pos..].find(".depends(\"") {
            let abs_start = search_pos + start + 10; // length of ".depends(\""
            if let Some(end) = source_without_comments[abs_start..].find('"') {
                dependencies.push(source_without_comments[abs_start..abs_start + end].to_string());
                search_pos = abs_start + end;
            } else {
                break;
            }
        }

        // Extract actions - look for common action patterns
        self.extract_actions(&source_without_comments, &mut actions);

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
            actions,
        })
    }

    fn extract_actions(&self, source: &str, actions: &mut Vec<ModuleAction>) {
        // Extract packageInstall actions
        let mut pos = 0;
        while let Some(start) = source[pos..].find("packageInstall({") {
            let abs_start = pos + start + 16;
            if let Some(end) = self.find_closing_brace(&source[abs_start..]) {
                let action_content = &source[abs_start..abs_start + end];
                if let Some(action) = self.parse_package_install(action_content) {
                    actions.push(action);
                }
                pos = abs_start + end;
            } else {
                break;
            }
        }

        // Extract linkDotfile actions
        pos = 0;
        while let Some(start) = source[pos..].find("linkDotfile({") {
            let abs_start = pos + start + 13;
            if let Some(end) = self.find_closing_brace(&source[abs_start..]) {
                let action_content = &source[abs_start..abs_start + end];
                if let Some(action) = self.parse_link_dotfile(action_content) {
                    actions.push(action);
                }
                pos = abs_start + end;
            } else {
                break;
            }
        }

        // Extract executeCommand actions
        pos = 0;
        while let Some(start) = source[pos..].find("executeCommand({") {
            let abs_start = pos + start + 16;
            if let Some(end) = self.find_closing_brace(&source[abs_start..]) {
                let action_content = &source[abs_start..abs_start + end];
                if let Some(action) = self.parse_execute_command(action_content) {
                    actions.push(action);
                }
                pos = abs_start + end;
            } else {
                break;
            }
        }

        // Extract copyFile actions
        pos = 0;
        while let Some(start) = source[pos..].find("copyFile({") {
            let abs_start = pos + start + 10;
            if let Some(end) = self.find_closing_brace(&source[abs_start..]) {
                let action_content = &source[abs_start..abs_start + end];
                if let Some(action) = self.parse_copy_file(action_content) {
                    actions.push(action);
                }
                pos = abs_start + end;
            } else {
                break;
            }
        }

        // Extract httpDownload actions
        pos = 0;
        while let Some(start) = source[pos..].find("httpDownload({") {
            let abs_start = pos + start + 14;
            if let Some(end) = self.find_closing_brace(&source[abs_start..]) {
                let action_content = &source[abs_start..abs_start + end];
                if let Some(action) = self.parse_http_download(action_content) {
                    actions.push(action);
                }
                pos = abs_start + end;
            } else {
                break;
            }
        }

        // Extract fileWrite actions
        pos = 0;
        while let Some(start) = source[pos..].find("fileWrite({") {
            let abs_start = pos + start + 11;
            if let Some(end) = self.find_closing_brace(&source[abs_start..]) {
                let action_content = &source[abs_start..abs_start + end];
                if let Some(action) = self.parse_file_write(action_content) {
                    actions.push(action);
                }
                pos = abs_start + end;
            } else {
                break;
            }
        }

        // Extract dconfImport actions
        pos = 0;
        while let Some(start) = source[pos..].find("dconfImport({") {
            let abs_start = pos + start + 13;
            if let Some(end) = self.find_closing_brace(&source[abs_start..]) {
                let action_content = &source[abs_start..abs_start + end];
                if let Some(action) = self.parse_dconf_import(action_content) {
                    actions.push(action);
                }
                pos = abs_start + end;
            } else {
                break;
            }
        }

        // Extract systemdService actions
        pos = 0;
        while let Some(start) = source[pos..].find("systemdService({") {
            let abs_start = pos + start + 16;
            if let Some(end) = self.find_closing_brace(&source[abs_start..]) {
                let action_content = &source[abs_start..abs_start + end];
                if let Some(action) = self.parse_systemd_service(action_content) {
                    actions.push(action);
                }
                pos = abs_start + end;
            } else {
                break;
            }
        }

        // Extract systemdSocket actions
        pos = 0;
        while let Some(start) = source[pos..].find("systemdSocket({") {
            let abs_start = pos + start + 15;
            if let Some(end) = self.find_closing_brace(&source[abs_start..]) {
                let action_content = &source[abs_start..abs_start + end];
                if let Some(action) = self.parse_systemd_socket(action_content) {
                    actions.push(action);
                }
                pos = abs_start + end;
            } else {
                break;
            }
        }
    }

    fn remove_comments(&self, source: &str) -> String {
        let mut result = String::new();
        let mut chars = source.chars().peekable();
        let mut in_string = false;
        let mut in_single_string = false;
        let mut in_template = false;
        let mut escape = false;

        while let Some(ch) = chars.next() {
            if escape {
                result.push(ch);
                escape = false;
                continue;
            }

            match ch {
                '\\' => {
                    result.push(ch);
                    escape = true;
                }
                '"' if !in_single_string && !in_template => {
                    result.push(ch);
                    in_string = !in_string;
                }
                '\'' if !in_string && !in_template => {
                    result.push(ch);
                    in_single_string = !in_single_string;
                }
                '`' if !in_string && !in_single_string => {
                    result.push(ch);
                    in_template = !in_template;
                }
                '/' if !in_string && !in_single_string && !in_template => {
                    if let Some(&next_ch) = chars.peek() {
                        if next_ch == '/' {
                            // Single-line comment - skip until newline
                            chars.next(); // consume the second '/'
                            for comment_ch in chars.by_ref() {
                                if comment_ch == '\n' {
                                    result.push('\n');
                                    break;
                                }
                            }
                        } else if next_ch == '*' {
                            // Multi-line comment - skip until */
                            chars.next(); // consume the '*'
                            let mut prev_ch = ' ';
                            for comment_ch in chars.by_ref() {
                                if prev_ch == '*' && comment_ch == '/' {
                                    break;
                                }
                                prev_ch = comment_ch;
                            }
                        } else {
                            result.push(ch);
                        }
                    } else {
                        result.push(ch);
                    }
                }
                _ => {
                    result.push(ch);
                }
            }
        }

        result
    }

    fn find_closing_brace(&self, s: &str) -> Option<usize> {
        let mut depth = 1;
        let mut in_string = false;
        let mut escape = false;

        for (i, ch) in s.chars().enumerate() {
            if escape {
                escape = false;
                continue;
            }

            match ch {
                '\\' => escape = true,
                '"' if !in_string => in_string = true,
                '"' if in_string => in_string = false,
                '{' if !in_string => depth += 1,
                '}' if !in_string => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn parse_package_install(&self, content: &str) -> Option<ModuleAction> {
        let mut params = Vec::new();

        // Extract names
        if let Some(names_start) = content.find("names:") {
            let after_names = &content[names_start + 6..];
            if let Some(array_start) = after_names.find('[') {
                if let Some(array_end) = after_names.find(']') {
                    let names_content = &after_names[array_start + 1..array_end];
                    let packages: Vec<&str> = names_content
                        .split(',')
                        .map(|s| s.trim().trim_matches('"'))
                        .filter(|s| !s.is_empty())
                        .collect();
                    if !packages.is_empty() {
                        params.push(("packages".to_string(), packages.join(", ")));
                    }
                }
            }
        }

        // Extract manager
        if let Some(manager_start) = content.find("manager:") {
            let after_manager = &content[manager_start + 8..];
            if let Some(quote_start) = after_manager.find('"') {
                if let Some(quote_end) = after_manager[quote_start + 1..].find('"') {
                    let manager = &after_manager[quote_start + 1..quote_start + 1 + quote_end];
                    params.push(("manager".to_string(), manager.to_string()));
                }
            }
        }

        if !params.is_empty() {
            Some(ModuleAction {
                action_type: "packageInstall".to_string(),
                params,
            })
        } else {
            None
        }
    }

    fn parse_link_dotfile(&self, content: &str) -> Option<ModuleAction> {
        let mut params = Vec::new();

        // Extract source
        if let Some(source_start) = content.find("source:") {
            let after_source = &content[source_start + 7..];
            if let Some(quote_start) = after_source.find('"') {
                if let Some(quote_end) = after_source[quote_start + 1..].find('"') {
                    let source = &after_source[quote_start + 1..quote_start + 1 + quote_end];

                    // Resolve source path relative to the TypeScript module file
                    let resolved_source = if let Some(current_path) = &self.current_path {
                        if let Some(parent_dir) = current_path.parent() {
                            // If source is not absolute, resolve it relative to the module's directory
                            if !source.starts_with('/') && !source.starts_with('~') {
                                parent_dir.join(source).to_string_lossy().to_string()
                            } else {
                                source.to_string()
                            }
                        } else {
                            source.to_string()
                        }
                    } else {
                        source.to_string()
                    };

                    params.push(("source".to_string(), resolved_source));
                }
            }
        }

        // Extract target
        if let Some(target_start) = content.find("target:") {
            let after_target = &content[target_start + 7..];
            if let Some(quote_start) = after_target.find('"') {
                if let Some(quote_end) = after_target[quote_start + 1..].find('"') {
                    let target = &after_target[quote_start + 1..quote_start + 1 + quote_end];
                    params.push(("target".to_string(), target.to_string()));
                }
            }
        }

        // Check for backup
        if content.contains("backup: true") {
            params.push(("backup".to_string(), "true".to_string()));
        }

        // Check for force
        if content.contains("force: true") {
            params.push(("force".to_string(), "true".to_string()));
        }

        if !params.is_empty() {
            Some(ModuleAction {
                action_type: "linkDotfile".to_string(),
                params,
            })
        } else {
            None
        }
    }

    fn parse_execute_command(&self, content: &str) -> Option<ModuleAction> {
        let mut params = Vec::new();

        // Extract command
        if let Some(cmd_start) = content.find("command:") {
            let after_cmd = &content[cmd_start + 8..];
            if let Some(quote_start) = after_cmd.find('"') {
                if let Some(quote_end) = after_cmd[quote_start + 1..].find('"') {
                    let command = &after_cmd[quote_start + 1..quote_start + 1 + quote_end];
                    params.push(("command".to_string(), command.to_string()));
                }
            }
        }

        // Extract args
        if let Some(args_start) = content.find("args:") {
            let after_args = &content[args_start + 5..];
            if let Some(array_start) = after_args.find('[') {
                if let Some(array_end) = after_args.find(']') {
                    let args_content = &after_args[array_start + 1..array_end];
                    let args: Vec<&str> = args_content
                        .split(',')
                        .map(|s| s.trim().trim_matches('"'))
                        .filter(|s| !s.is_empty())
                        .collect();
                    if !args.is_empty() {
                        params.push(("args".to_string(), args.join(" ")));
                    }
                }
            }
        }

        // Extract shell
        if let Some(shell_start) = content.find("shell:") {
            let after_shell = &content[shell_start + 6..];
            if let Some(quote_start) = after_shell.find('"') {
                if let Some(quote_end) = after_shell[quote_start + 1..].find('"') {
                    let shell = &after_shell[quote_start + 1..quote_start + 1 + quote_end];
                    params.push(("shell".to_string(), shell.to_string()));
                }
            }
        }

        if !params.is_empty() {
            Some(ModuleAction {
                action_type: "executeCommand".to_string(),
                params,
            })
        } else {
            None
        }
    }

    fn parse_copy_file(&self, content: &str) -> Option<ModuleAction> {
        let mut params = Vec::new();

        // Extract source
        if let Some(source_start) = content.find("source:") {
            let after_source = &content[source_start + 7..];
            if let Some(quote_start) = after_source.find('"') {
                if let Some(quote_end) = after_source[quote_start + 1..].find('"') {
                    let source = &after_source[quote_start + 1..quote_start + 1 + quote_end];

                    // Resolve source path relative to the TypeScript module file
                    let resolved_source = if let Some(current_path) = &self.current_path {
                        if let Some(parent_dir) = current_path.parent() {
                            // If source is not absolute, resolve it relative to the module's directory
                            if !source.starts_with('/') && !source.starts_with('~') {
                                parent_dir.join(source).to_string_lossy().to_string()
                            } else {
                                source.to_string()
                            }
                        } else {
                            source.to_string()
                        }
                    } else {
                        source.to_string()
                    };

                    params.push(("source".to_string(), resolved_source));
                }
            }
        }

        // Extract destination
        if let Some(dest_start) = content.find("destination:") {
            let after_dest = &content[dest_start + 12..];
            if let Some(quote_start) = after_dest.find('"') {
                if let Some(quote_end) = after_dest[quote_start + 1..].find('"') {
                    let destination = &after_dest[quote_start + 1..quote_start + 1 + quote_end];
                    params.push(("destination".to_string(), destination.to_string()));
                }
            }
        }

        // Check for privileged
        if content.contains("privileged: true") {
            params.push(("privileged".to_string(), "true".to_string()));
        }

        // Check for backup
        if content.contains("backup: true") {
            params.push(("backup".to_string(), "true".to_string()));
        }

        if !params.is_empty() {
            Some(ModuleAction {
                action_type: "copyFile".to_string(),
                params,
            })
        } else {
            None
        }
    }

    fn parse_http_download(&self, content: &str) -> Option<ModuleAction> {
        let mut params = Vec::new();

        // Extract url
        if let Some(url_start) = content.find("url:") {
            let after_url = &content[url_start + 4..];
            if let Some(quote_start) = after_url.find('"') {
                if let Some(quote_end) = after_url[quote_start + 1..].find('"') {
                    let url = &after_url[quote_start + 1..quote_start + 1 + quote_end];
                    params.push(("url".to_string(), url.to_string()));
                }
            }
        }

        // Extract destination
        if let Some(dest_start) = content.find("destination:") {
            let after_dest = &content[dest_start + 12..];
            if let Some(quote_start) = after_dest.find('"') {
                if let Some(quote_end) = after_dest[quote_start + 1..].find('"') {
                    let destination = &after_dest[quote_start + 1..quote_start + 1 + quote_end];
                    params.push(("destination".to_string(), destination.to_string()));
                }
            }
        }

        // Check for privileged
        if content.contains("privileged: true") {
            params.push(("privileged".to_string(), "true".to_string()));
        }

        if !params.is_empty() {
            Some(ModuleAction {
                action_type: "httpDownload".to_string(),
                params,
            })
        } else {
            None
        }
    }

    fn parse_file_write(&self, content: &str) -> Option<ModuleAction> {
        let mut params = Vec::new();

        // Extract destination
        if let Some(dest_start) = content.find("destination:") {
            let after_dest = &content[dest_start + 12..];
            if let Some(quote_start) = after_dest.find('"') {
                if let Some(quote_end) = after_dest[quote_start + 1..].find('"') {
                    let destination = &after_dest[quote_start + 1..quote_start + 1 + quote_end];
                    params.push(("destination".to_string(), destination.to_string()));
                }
            }
        }

        // Extract content (simplified - just show that content exists)
        if content.contains("content:") {
            params.push(("content".to_string(), "<content>".to_string()));
        }

        // Check for privileged
        if content.contains("privileged: true") {
            params.push(("privileged".to_string(), "true".to_string()));
        }

        // Check for backup
        if content.contains("backup: true") {
            params.push(("backup".to_string(), "true".to_string()));
        }

        if !params.is_empty() {
            Some(ModuleAction {
                action_type: "fileWrite".to_string(),
                params,
            })
        } else {
            None
        }
    }

    fn parse_dconf_import(&self, content: &str) -> Option<ModuleAction> {
        let mut params = Vec::new();

        // Extract source
        if let Some(source_start) = content.find("source:") {
            let after_source = &content[source_start + 7..];
            if let Some(quote_start) = after_source.find('"') {
                if let Some(quote_end) = after_source[quote_start + 1..].find('"') {
                    let source = &after_source[quote_start + 1..quote_start + 1 + quote_end];

                    // Resolve source path relative to the TypeScript module file
                    let resolved_source = if let Some(current_path) = &self.current_path {
                        if let Some(parent_dir) = current_path.parent() {
                            // If source is not absolute, resolve it relative to the module's directory
                            if !source.starts_with('/') && !source.starts_with('~') {
                                parent_dir.join(source).to_string_lossy().to_string()
                            } else {
                                source.to_string()
                            }
                        } else {
                            source.to_string()
                        }
                    } else {
                        source.to_string()
                    };

                    params.push(("source".to_string(), resolved_source));
                }
            }
        }

        // Extract path
        if let Some(path_start) = content.find("path:") {
            let after_path = &content[path_start + 5..];
            if let Some(quote_start) = after_path.find('"') {
                if let Some(quote_end) = after_path[quote_start + 1..].find('"') {
                    let path = &after_path[quote_start + 1..quote_start + 1 + quote_end];
                    params.push(("path".to_string(), path.to_string()));
                }
            }
        }

        // Check for backup
        if content.contains("backup: true") {
            params.push(("backup".to_string(), "true".to_string()));
        }

        if !params.is_empty() {
            Some(ModuleAction {
                action_type: "dconfImport".to_string(),
                params,
            })
        } else {
            None
        }
    }

    fn parse_systemd_service(&self, content: &str) -> Option<ModuleAction> {
        let mut params = Vec::new();

        // Extract name
        if let Some(name_start) = content.find("name:") {
            let after_name = &content[name_start + 5..];
            if let Some(quote_start) = after_name.find('"') {
                if let Some(quote_end) = after_name[quote_start + 1..].find('"') {
                    let name = &after_name[quote_start + 1..quote_start + 1 + quote_end];
                    params.push(("name".to_string(), name.to_string()));
                }
            }
        }

        // Extract content (simplified)
        if content.contains("content:") {
            params.push(("content".to_string(), "<service content>".to_string()));
        }

        // Check for user
        if content.contains("user: true") {
            params.push(("user".to_string(), "true".to_string()));
        }

        // Check for enable
        if content.contains("enable: true") {
            params.push(("enable".to_string(), "true".to_string()));
        }

        // Check for start
        if content.contains("start: true") {
            params.push(("start".to_string(), "true".to_string()));
        }

        // Check for reload
        if content.contains("reload: true") {
            params.push(("reload".to_string(), "true".to_string()));
        }

        if !params.is_empty() {
            Some(ModuleAction {
                action_type: "systemdService".to_string(),
                params,
            })
        } else {
            None
        }
    }

    fn parse_systemd_socket(&self, content: &str) -> Option<ModuleAction> {
        let mut params = Vec::new();

        // Extract name
        if let Some(name_start) = content.find("name:") {
            let after_name = &content[name_start + 5..];
            if let Some(quote_start) = after_name.find('"') {
                if let Some(quote_end) = after_name[quote_start + 1..].find('"') {
                    let name = &after_name[quote_start + 1..quote_start + 1 + quote_end];
                    params.push(("name".to_string(), name.to_string()));
                }
            }
        }

        // Extract content (simplified)
        if content.contains("content:") {
            params.push(("content".to_string(), "<socket content>".to_string()));
        }

        // Check for user
        if content.contains("user: true") {
            params.push(("user".to_string(), "true".to_string()));
        }

        // Check for enable
        if content.contains("enable: true") {
            params.push(("enable".to_string(), "true".to_string()));
        }

        // Check for start
        if content.contains("start: true") {
            params.push(("start".to_string(), "true".to_string()));
        }

        // Check for reload
        if content.contains("reload: true") {
            params.push(("reload".to_string(), "true".to_string()));
        }

        if !params.is_empty() {
            Some(ModuleAction {
                action_type: "systemdSocket".to_string(),
                params,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModuleAction {
    pub action_type: String,
    pub params: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct ModuleData {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub dependencies: Vec<String>,
    pub actions: Vec<ModuleAction>,
}

/// Maximum depth for recursive module loading
const MAX_MODULE_DEPTH: usize = 8;

/// Load all modules from a directory recursively
pub fn load_modules_from_directory(path: impl AsRef<Path>) -> Result<Vec<ModuleData>> {
    let mut loader = ModuleLoader::new();
    let mut modules = Vec::new();

    load_modules_recursive(&mut loader, &mut modules, path.as_ref(), 0)?;

    Ok(modules)
}

/// Recursively load modules from a directory and its subdirectories
fn load_modules_recursive(
    loader: &mut ModuleLoader,
    modules: &mut Vec<ModuleData>,
    path: &Path,
    depth: usize,
) -> Result<()> {
    if !path.is_dir() {
        return Ok(());
    }

    // Check if we've exceeded the maximum depth
    if depth >= MAX_MODULE_DEPTH {
        tracing::debug!(
            "Skipping directory {:?} - exceeded maximum depth of {}",
            path,
            MAX_MODULE_DEPTH
        );
        return Ok(());
    }

    let entries = std::fs::read_dir(path).map_err(|e| {
        DhdError::ModuleParse(format!("Failed to read directory {:?}: {}", path, e))
    })?;

    for entry in entries {
        let entry =
            entry.map_err(|e| DhdError::ModuleParse(format!("Failed to read entry: {}", e)))?;
        let entry_path = entry.path();

        if entry_path.is_dir() {
            // Skip common directories that shouldn't contain modules
            if let Some(dir_name) = entry_path.file_name().and_then(|n| n.to_str()) {
                if dir_name.starts_with('.')
                    || dir_name == "node_modules"
                    || dir_name == "target"
                    || dir_name == "dist"
                    || dir_name == "build"
                {
                    continue;
                }
            }

            // Recursively load from subdirectory
            load_modules_recursive(loader, modules, &entry_path, depth + 1)?;
        } else if entry_path.extension().and_then(|s| s.to_str()) == Some("ts") {
            // Skip test files and type definition files
            if let Some(file_name) = entry_path.file_name().and_then(|n| n.to_str()) {
                if file_name.ends_with(".test.ts")
                    || file_name.ends_with(".spec.ts")
                    || file_name.ends_with(".d.ts")
                {
                    continue;
                }
            }

            match loader.load_module(&entry_path) {
                Ok(module_data) => {
                    tracing::debug!(
                        "Successfully loaded module: {} from {:?}",
                        module_data.id,
                        entry_path
                    );
                    modules.push(module_data);
                }
                Err(e) => {
                    tracing::warn!("Failed to load module from {:?}: {}", entry_path, e);
                }
            }
        }
    }

    Ok(())
}
