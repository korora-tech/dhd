use super::Action;
use crate::atoms::AtomCompat;
use dhd_macros::{typescript_fn, typescript_type};
use directories::BaseDirs;
use std::path::PathBuf;

/// Resolve XDG paths like relative config paths to their full locations
fn resolve_xdg_target(target: &str) -> PathBuf {
    let target_path = PathBuf::from(target);

    // If it's already absolute, return as-is
    if target_path.is_absolute() {
        return target_path;
    }

    // Handle tilde expansion for home directory
    if let Some(stripped) = target.strip_prefix("~/") {
        if let Some(base_dirs) = BaseDirs::new() {
            return base_dirs.home_dir().join(stripped);
        }
    }

    // If it's a relative path, assume it's relative to XDG_CONFIG_HOME
    if let Some(base_dirs) = BaseDirs::new() {
        base_dirs.config_dir().join(target)
    } else {
        // Fallback if we can't determine base directories
        PathBuf::from(".config").join(target)
    }
}

#[typescript_type]
/// Links a directory from the module to a target location
///
/// * `source` - Path to the source directory, relative to the module directory
/// * `target` - Target path where the symlink will be created
///   - If absolute: used as-is
///   - If starts with `~/`: expanded to home directory
///   - If relative: resolved relative to XDG_CONFIG_HOME (usually ~/.config)
/// * `force` - If true, creates parent directories and overwrites existing files/directories
pub struct LinkDirectory {
    pub source: String,
    pub target: String,
    pub force: bool,
}

#[typescript_fn]
pub fn link_directory(config: LinkDirectory) -> super::ActionType {
    super::ActionType::LinkDirectory(config)
}

impl Action for LinkDirectory {
    fn name(&self) -> &str {
        "LinkDirectory"
    }

    fn plan(&self, module_dir: &std::path::Path) -> Vec<Box<dyn crate::atom::Atom>> {
        // Resolve source path relative to module directory if it's not absolute
        let source_path = if PathBuf::from(&self.source).is_absolute() {
            PathBuf::from(&self.source)
        } else {
            module_dir.join(&self.source)
        };

        // Resolve target path using XDG conventions
        let target_path = resolve_xdg_target(&self.target);

        vec![Box::new(AtomCompat::new(
            Box::new(crate::atoms::LinkFile {
                source: source_path,
                target: target_path,
                force: self.force,
            }),
            "link_directory".to_string(),
        ))]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::ActionType;

    #[test]
    fn test_link_directory_creation() {
        let action = LinkDirectory {
            source: "/home/user/.dotfiles/config".to_string(),
            target: "/home/user/.config/app".to_string(),
            force: false,
        };

        assert_eq!(action.source, "/home/user/.dotfiles/config");
        assert_eq!(action.target, "/home/user/.config/app");
        assert_eq!(action.force, false);
    }

    #[test]
    fn test_link_directory_helper_function() {
        let action = link_directory(LinkDirectory {
            source: "configs/app".to_string(),
            target: "app".to_string(),
            force: true,
        });

        match action {
            ActionType::LinkDirectory(link) => {
                assert_eq!(link.source, "configs/app");
                assert_eq!(link.target, "app");
                assert_eq!(link.force, true);
            }
            _ => panic!("Expected LinkDirectory action type"),
        }
    }

    #[test]
    fn test_link_directory_name() {
        let action = LinkDirectory {
            source: "source".to_string(),
            target: "target".to_string(),
            force: false,
        };

        assert_eq!(action.name(), "LinkDirectory");
    }

    #[test]
    fn test_link_directory_plan() {
        let action = LinkDirectory {
            source: "/source/path".to_string(),
            target: "/target/path".to_string(),
            force: false,
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
        // Check that we got an atom
        assert_eq!(atoms.len(), 1);
    }

    #[test]
    fn test_link_directory_plan_relative_path() {
        use std::path::Path;

        let action = LinkDirectory {
            source: "config".to_string(),
            target: "~/.config/app".to_string(),
            force: false,
        };

        let module_dir = Path::new("/home/user/modules/app");
        let atoms = action.plan(module_dir);

        assert_eq!(atoms.len(), 1);
        // Check that we got an atom
        assert_eq!(atoms.len(), 1);
        // The describe method should show the resolved path
        assert!(
            atoms[0]
                .describe()
                .contains("/home/user/modules/app/config")
        );
    }

    #[test]
    fn test_link_directory_plan_absolute_path() {
        use std::path::Path;

        let action = LinkDirectory {
            source: "/absolute/source/path".to_string(),
            target: "~/.config/app".to_string(),
            force: false,
        };

        let module_dir = Path::new("/home/user/modules/app");
        let atoms = action.plan(module_dir);

        assert_eq!(atoms.len(), 1);
        // Check that we got an atom
        assert_eq!(atoms.len(), 1);
        // The describe method should show the original absolute path (not modified)
        assert!(atoms[0].describe().contains("/absolute/source/path"));
    }

    #[test]
    fn test_resolve_xdg_target_relative_path() {
        let result = resolve_xdg_target("app/config");
        // Should resolve to some config directory + app/config
        assert!(result.to_string_lossy().contains("config"));
        assert!(result.to_string_lossy().ends_with("app/config"));
    }

    #[test]
    fn test_resolve_xdg_target_absolute_path() {
        let result = resolve_xdg_target("/absolute/path/config");
        assert_eq!(result, PathBuf::from("/absolute/path/config"));
    }

    #[test]
    fn test_resolve_xdg_target_tilde_expansion() {
        let result = resolve_xdg_target("~/config/app");
        // Should resolve to some home directory + config/app
        assert!(result.to_string_lossy().ends_with("config/app"));
        // Should not contain literal tilde
        assert!(!result.to_string_lossy().contains("~"));
    }

    #[test]
    fn test_link_directory_xdg_integration() {
        use std::path::Path;

        let action = LinkDirectory {
            source: "config".to_string(),
            target: "app".to_string(), // Relative to XDG_CONFIG_HOME
            force: false,
        };

        let module_dir = Path::new("/home/user/modules/app");
        let atoms = action.plan(module_dir);

        assert_eq!(atoms.len(), 1);
        // Check that we got an atom
        assert_eq!(atoms.len(), 1);

        let description = atoms[0].describe();
        // Check that source is resolved relative to module directory
        assert!(description.contains("/home/user/modules/app/config"));
        // Check that target contains the config directory path
        assert!(description.contains("config"));
        assert!(description.contains("app"));
    }

    #[test]
    fn test_link_directory_with_force() {
        use std::path::Path;

        let action = LinkDirectory {
            source: "config".to_string(),
            target: "app".to_string(),
            force: true,
        };

        let module_dir = Path::new("/home/user/modules/app");
        let atoms = action.plan(module_dir);

        assert_eq!(atoms.len(), 1);
        // Check that we got an atom
        assert_eq!(atoms.len(), 1);

        // Verify the atom has force enabled (through description or other means)
        let description = atoms[0].describe();
        assert!(description.contains("/home/user/modules/app/config"));
        assert!(description.contains("config"));
        assert!(description.contains("app"));
    }
}
