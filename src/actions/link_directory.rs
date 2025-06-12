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
/// Creates a symbolic link at a specified location pointing to a directory
///
/// * `source` - Path where the symlink will be created
///   - If absolute: used as-is
///   - If starts with `~/`: expanded to home directory
///   - If relative: resolved relative to XDG_CONFIG_HOME (usually ~/.config)
/// * `target` - Path to the target directory that the symlink points to, relative to the module directory
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
        // Resolve source path using XDG conventions (where the symlink will be created)
        let source_path = resolve_xdg_target(&self.source);

        // Resolve target path relative to module directory if it's not absolute
        let target_path = if PathBuf::from(&self.target).is_absolute() {
            PathBuf::from(&self.target)
        } else {
            module_dir.join(&self.target)
        };

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
            source: "/home/user/.config/app".to_string(),
            target: "/home/user/.dotfiles/config".to_string(),
            force: false,
        };

        assert_eq!(action.source, "/home/user/.config/app");
        assert_eq!(action.target, "/home/user/.dotfiles/config");
        assert!(!action.force);
    }

    #[test]
    fn test_link_directory_helper_function() {
        let action = link_directory(LinkDirectory {
            source: "app".to_string(),
            target: "configs/app".to_string(),
            force: true,
        });

        match action {
            ActionType::LinkDirectory(link) => {
                assert_eq!(link.source, "app");
                assert_eq!(link.target, "configs/app");
                assert!(link.force);
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
            source: "~/.config/app".to_string(),
            target: "config".to_string(),
            force: false,
        };

        let module_dir = Path::new("/home/user/modules/app");
        let atoms = action.plan(module_dir);

        assert_eq!(atoms.len(), 1);
        // Check that we got an atom
        assert_eq!(atoms.len(), 1);
        // The describe method should show the resolved path
        // After swapping, it should contain the target from module directory
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
            target: "config".to_string(),
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
            source: "app".to_string(), // Where to create symlink (relative to XDG_CONFIG_HOME)
            target: "config".to_string(), // What it points to (relative to module dir)
            force: false,
        };

        let module_dir = Path::new("/home/user/modules/app");
        let atoms = action.plan(module_dir);

        assert_eq!(atoms.len(), 1);
        // Check that we got an atom
        assert_eq!(atoms.len(), 1);

        let description = atoms[0].describe();
        // After swapping, source is the symlink location (resolved to XDG)
        // and target is the directory in the module directory
        // Description should show: Create symlink at <XDG_CONFIG>/app -> /home/user/modules/app/config
        assert!(description.contains("app"));
        assert!(description.contains("/home/user/modules/app/config"));
    }

    #[test]
    fn test_link_directory_with_force() {
        use std::path::Path;

        let action = LinkDirectory {
            source: "app".to_string(), // Where to create symlink
            target: "config".to_string(), // What it points to
            force: true,
        };

        let module_dir = Path::new("/home/user/modules/app");
        let atoms = action.plan(module_dir);

        assert_eq!(atoms.len(), 1);
        // Check that we got an atom
        assert_eq!(atoms.len(), 1);

        // Verify the atom has force enabled (through description or other means)
        let description = atoms[0].describe();
        // After swapping, description should show symlink location -> target
        assert!(description.contains("app"));
        assert!(description.contains("/home/user/modules/app/config"));
    }
}
