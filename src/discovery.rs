use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct DiscoveredModule {
    pub path: PathBuf,
    pub name: String,
}

impl DiscoveredModule {
    /// Get the relative path from a base directory
    pub fn relative_path(&self, base: &Path) -> Option<PathBuf> {
        self.path.strip_prefix(base).ok().map(|p| p.to_path_buf())
    }

    /// Check if this module is in a subdirectory
    pub fn is_nested(&self, base: &Path) -> bool {
        self.relative_path(base)
            .map(|p| p.components().count() > 1)
            .unwrap_or(false)
    }
}

// Directories to exclude from module discovery
const EXCLUDED_DIRS: &[&str] = &["node_modules", "dist", "build", ".git", "target"];

pub fn discover_modules(dir: &Path) -> Result<Vec<DiscoveredModule>, std::io::Error> {
    let mut modules = Vec::new();
    let excluded_dirs: HashSet<&str> = EXCLUDED_DIRS.iter().copied().collect();

    discover_modules_recursive(dir, &mut modules, &excluded_dirs)?;

    // Sort modules by name for consistent ordering
    modules.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(modules)
}

/// Discover modules in the current directory
pub fn discover_modules_in_current_dir() -> Result<Vec<DiscoveredModule>, std::io::Error> {
    let current_dir = std::env::current_dir()?;
    discover_modules(&current_dir)
}

/// Discover modules and print a summary
pub fn discover_and_summarize(dir: &Path) -> Result<String, std::io::Error> {
    let modules = discover_modules(dir)?;

    if modules.is_empty() {
        Ok("No TypeScript modules found".to_string())
    } else {
        let mut summary = format!("Found {} TypeScript module(s):\n", modules.len());
        for module in &modules {
            if let Some(relative) = module.relative_path(dir) {
                summary.push_str(&format!("  - {} ({})\n", module.name, relative.display()));
            } else {
                summary.push_str(&format!(
                    "  - {} ({})\n",
                    module.name,
                    module.path.display()
                ));
            }
        }
        Ok(summary)
    }
}

fn discover_modules_recursive(
    dir: &Path,
    modules: &mut Vec<DiscoveredModule>,
    excluded_dirs: &HashSet<&str>,
) -> Result<(), std::io::Error> {
    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        // Skip hidden files and directories (starting with .)
        if file_name_str.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            // Skip excluded directories
            if excluded_dirs.contains(file_name_str.as_ref()) {
                continue;
            }

            // Recursively search subdirectories
            discover_modules_recursive(&path, modules, excluded_dirs)?;
        } else if path.is_file() {
            // Check if it's a TypeScript file
            if let Some(extension) = path.extension() {
                if extension == "ts" {
                    // Skip generated files
                    if file_name_str == "types.d.ts" {
                        continue;
                    }

                    // Extract the module name (filename without extension)
                    if let Some(stem) = path.file_stem() {
                        let name = stem.to_string_lossy().into_owned();
                        modules.push(DiscoveredModule {
                            path: path.clone(),
                            name,
                        });
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_discover_modules_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        let modules = discover_modules(temp_dir.path()).unwrap();
        assert_eq!(modules.len(), 0);
    }

    #[test]
    fn test_discover_modules_single_ts_file() {
        let temp_dir = TempDir::new().unwrap();
        let module_path = temp_dir.path().join("test.ts");
        File::create(&module_path).unwrap();

        let modules = discover_modules(temp_dir.path()).unwrap();
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].name, "test");
        assert_eq!(modules[0].path, module_path);
    }

    #[test]
    fn test_discover_modules_multiple_ts_files() {
        let temp_dir = TempDir::new().unwrap();

        let files = vec!["module1.ts", "module2.ts", "config.ts"];
        for file in &files {
            File::create(temp_dir.path().join(file)).unwrap();
        }

        let modules = discover_modules(temp_dir.path()).unwrap();
        assert_eq!(modules.len(), 3);

        let names: Vec<String> = modules.iter().map(|m| m.name.clone()).collect();
        assert!(names.contains(&"module1".to_string()));
        assert!(names.contains(&"module2".to_string()));
        assert!(names.contains(&"config".to_string()));
    }

    #[test]
    fn test_discover_modules_ignores_non_ts_files() {
        let temp_dir = TempDir::new().unwrap();

        File::create(temp_dir.path().join("module.ts")).unwrap();
        File::create(temp_dir.path().join("readme.md")).unwrap();
        File::create(temp_dir.path().join("config.json")).unwrap();
        File::create(temp_dir.path().join("script.js")).unwrap();

        let modules = discover_modules(temp_dir.path()).unwrap();
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].name, "module");
    }

    #[test]
    fn test_discover_modules_nested_directories() {
        let temp_dir = TempDir::new().unwrap();

        // Create files in root
        File::create(temp_dir.path().join("root.ts")).unwrap();

        // Create nested directory with files
        let nested_dir = temp_dir.path().join("nested");
        fs::create_dir(&nested_dir).unwrap();
        File::create(nested_dir.join("nested.ts")).unwrap();

        let modules = discover_modules(temp_dir.path()).unwrap();
        assert_eq!(modules.len(), 2);

        let names: Vec<String> = modules.iter().map(|m| m.name.clone()).collect();
        assert!(names.contains(&"root".to_string()));
        assert!(names.contains(&"nested".to_string()));
    }

    #[test]
    fn test_discover_modules_hidden_files() {
        let temp_dir = TempDir::new().unwrap();

        File::create(temp_dir.path().join("visible.ts")).unwrap();
        File::create(temp_dir.path().join(".hidden.ts")).unwrap();

        let modules = discover_modules(temp_dir.path()).unwrap();
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].name, "visible");
    }

    #[test]
    fn test_discover_modules_special_characters_in_names() {
        let temp_dir = TempDir::new().unwrap();

        File::create(temp_dir.path().join("my-module.ts")).unwrap();
        File::create(temp_dir.path().join("another_module.ts")).unwrap();
        File::create(temp_dir.path().join("module.test.ts")).unwrap();

        let modules = discover_modules(temp_dir.path()).unwrap();
        assert_eq!(modules.len(), 3);

        let names: Vec<String> = modules.iter().map(|m| m.name.clone()).collect();
        assert!(names.contains(&"my-module".to_string()));
        assert!(names.contains(&"another_module".to_string()));
        assert!(names.contains(&"module.test".to_string()));
    }

    #[test]
    fn test_discover_modules_excludes_node_modules() {
        let temp_dir = TempDir::new().unwrap();

        File::create(temp_dir.path().join("app.ts")).unwrap();

        // Create node_modules directory
        let node_modules = temp_dir.path().join("node_modules");
        fs::create_dir(&node_modules).unwrap();
        File::create(node_modules.join("library.ts")).unwrap();

        let modules = discover_modules(temp_dir.path()).unwrap();
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].name, "app");
    }

    #[test]
    fn test_discover_modules_excludes_dist_directories() {
        let temp_dir = TempDir::new().unwrap();

        File::create(temp_dir.path().join("source.ts")).unwrap();

        // Create dist and build directories
        let dist = temp_dir.path().join("dist");
        fs::create_dir(&dist).unwrap();
        File::create(dist.join("compiled.ts")).unwrap();

        let build = temp_dir.path().join("build");
        fs::create_dir(&build).unwrap();
        File::create(build.join("output.ts")).unwrap();

        let modules = discover_modules(temp_dir.path()).unwrap();
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].name, "source");
    }

    #[test]
    fn test_discover_modules_permission_error() {
        // This test would require setting up a directory with no read permissions
        // Skip for now as it's platform-specific
    }

    #[test]
    fn test_discover_modules_symlinks() {
        // This test would require creating symlinks which is platform-specific
        // Skip for now
    }

    #[test]
    fn test_discovered_module_relative_path() {
        let base = Path::new("/home/user/project");
        let module = DiscoveredModule {
            path: PathBuf::from("/home/user/project/src/module.ts"),
            name: "module".to_string(),
        };

        let relative = module.relative_path(base).unwrap();
        assert_eq!(relative, Path::new("src/module.ts"));
    }

    #[test]
    fn test_discovered_module_relative_path_not_under_base() {
        let base = Path::new("/home/user/project");
        let module = DiscoveredModule {
            path: PathBuf::from("/home/other/module.ts"),
            name: "module".to_string(),
        };

        assert!(module.relative_path(base).is_none());
    }

    #[test]
    fn test_discovered_module_is_nested() {
        let base = Path::new("/home/user/project");

        let root_module = DiscoveredModule {
            path: PathBuf::from("/home/user/project/module.ts"),
            name: "module".to_string(),
        };
        assert!(!root_module.is_nested(base));

        let nested_module = DiscoveredModule {
            path: PathBuf::from("/home/user/project/src/module.ts"),
            name: "module".to_string(),
        };
        assert!(nested_module.is_nested(base));

        let deeply_nested = DiscoveredModule {
            path: PathBuf::from("/home/user/project/src/components/module.ts"),
            name: "module".to_string(),
        };
        assert!(deeply_nested.is_nested(base));
    }

    #[test]
    fn test_discover_and_summarize_empty() {
        let temp_dir = TempDir::new().unwrap();

        let summary = discover_and_summarize(temp_dir.path()).unwrap();
        assert_eq!(summary, "No TypeScript modules found");
    }

    #[test]
    fn test_discover_and_summarize_with_modules() {
        let temp_dir = TempDir::new().unwrap();

        File::create(temp_dir.path().join("app.ts")).unwrap();
        File::create(temp_dir.path().join("config.ts")).unwrap();

        let summary = discover_and_summarize(temp_dir.path()).unwrap();
        assert!(summary.contains("Found 2 TypeScript module(s):"));
        assert!(summary.contains("app (app.ts)"));
        assert!(summary.contains("config (config.ts)"));
    }

    #[test]
    fn test_discover_and_summarize_nested() {
        let temp_dir = TempDir::new().unwrap();

        File::create(temp_dir.path().join("index.ts")).unwrap();

        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        File::create(src_dir.join("main.ts")).unwrap();

        let summary = discover_and_summarize(temp_dir.path()).unwrap();
        assert!(summary.contains("Found 2 TypeScript module(s):"));
        assert!(summary.contains("index (index.ts)"));
        assert!(summary.contains("main (src/main.ts)"));
    }
}
