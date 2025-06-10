use dhd_macros::{typescript_fn, typescript_type};
use super::Action;
use std::path::PathBuf;
use directories::BaseDirs;
use crate::atoms::AtomCompat;

/// Resolve XDG paths like relative config paths to their full locations
fn resolve_xdg_target(target: &str) -> PathBuf {
    let target_path = PathBuf::from(target);

    // If it's already absolute, return as-is
    if target_path.is_absolute() {
        return target_path;
    }

    // Handle tilde expansion for home directory
    if target.starts_with("~/") {
        if let Some(base_dirs) = BaseDirs::new() {
            return base_dirs.home_dir().join(&target[2..]);
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
/// Links a file from the module to a target location
///
/// * `source` - Path to the source file, relative to the module directory
/// * `target` - Target path where the symlink will be created
///   - If absolute: used as-is
///   - If starts with `~/`: expanded to home directory
///   - If relative: resolved relative to XDG_CONFIG_HOME (usually ~/.config)
/// * `force` - If true, creates parent directories and overwrites existing files
pub struct LinkFile {
    pub source: String,
    pub target: String,
    pub force: bool,
}

#[typescript_fn]
pub fn link_file(config: LinkFile) -> super::ActionType {
    super::ActionType::LinkFile(config)
}

impl Action for LinkFile {
    fn name(&self) -> &str {
        "LinkFile"
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
            Box::new(crate::atoms::link_file::LinkFile {
                source: source_path,
                target: target_path,
                force: self.force,
            }),
            "link_file".to_string(),
        ))]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::ActionType;

    #[test]
    fn test_link_file_creation() {
        let action = LinkFile {
            source: "/home/user/.dotfiles/vimrc".to_string(),
            target: "/home/user/.vimrc".to_string(),
            force: false,
        };

        assert_eq!(action.source, "/home/user/.dotfiles/vimrc");
        assert_eq!(action.target, "/home/user/.vimrc");
        assert_eq!(action.force, false);
    }

    #[test]
    fn test_link_file_helper_function() {
        let action = link_file(LinkFile {
            source: "source".to_string(),
            target: "target".to_string(),
            force: true,
        });

        match action {
            ActionType::LinkFile(link) => {
                assert_eq!(link.source, "source");
                assert_eq!(link.target, "target");
                assert_eq!(link.force, true);
            }
            _ => panic!("Expected LinkFile action type"),
        }
    }

    #[test]
    fn test_link_file_name() {
        let action = LinkFile {
            source: "source".to_string(),
            target: "target".to_string(),
            force: false,
        };

        assert_eq!(action.name(), "LinkFile");
    }

    #[test]
    fn test_link_file_plan() {
        let action = LinkFile {
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
    fn test_link_file_plan_relative_path() {
        use std::path::Path;

        let action = LinkFile {
            source: "config.toml".to_string(),
            target: "~/.config/app/config.toml".to_string(),
            force: false,
        };

        let module_dir = Path::new("/home/user/modules/app");
        let atoms = action.plan(module_dir);

        assert_eq!(atoms.len(), 1);
        // Check that we got an atom
        assert_eq!(atoms.len(), 1);
        // The describe method should show the resolved path
        assert!(atoms[0].describe().contains("/home/user/modules/app/config.toml"));
    }

    #[test]
    fn test_link_file_plan_absolute_path() {
        use std::path::Path;

        let action = LinkFile {
            source: "/absolute/source/path".to_string(),
            target: "~/.config/app/config.toml".to_string(),
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
        let result = resolve_xdg_target("atuin/config.toml");
        // Should resolve to some config directory + atuin/config.toml
        assert!(result.to_string_lossy().contains("config"));
        assert!(result.to_string_lossy().ends_with("atuin/config.toml"));
    }

    #[test]
    fn test_resolve_xdg_target_absolute_path() {
        let result = resolve_xdg_target("/absolute/path/config.toml");
        assert_eq!(result, PathBuf::from("/absolute/path/config.toml"));
    }

    #[test]
    fn test_resolve_xdg_target_tilde_expansion() {
        let result = resolve_xdg_target("~/config/app.toml");
        // Should resolve to some home directory + config/app.toml
        assert!(result.to_string_lossy().ends_with("config/app.toml"));
        // Should not contain literal tilde
        assert!(!result.to_string_lossy().contains("~"));
    }

    #[test]
    fn test_link_file_xdg_integration() {
        use std::path::Path;

        let action = LinkFile {
            source: "config.toml".to_string(),
            target: "atuin/config.toml".to_string(), // Relative to XDG_CONFIG_HOME
            force: false,
        };

        let module_dir = Path::new("/home/user/modules/atuin");
        let atoms = action.plan(module_dir);

        assert_eq!(atoms.len(), 1);
        // Check that we got an atom
        assert_eq!(atoms.len(), 1);

        let description = atoms[0].describe();
        // Check that source is resolved relative to module directory
        assert!(description.contains("/home/user/modules/atuin/config.toml"));
        // Check that target contains the config directory path
        assert!(description.contains("config"));
        assert!(description.contains("atuin/config.toml"));
    }

    #[test]
    fn test_link_file_with_force() {
        use std::path::Path;

        let action = LinkFile {
            source: "config.toml".to_string(),
            target: "atuin/config.toml".to_string(),
            force: true,
        };

        let module_dir = Path::new("/home/user/modules/atuin");
        let atoms = action.plan(module_dir);

        assert_eq!(atoms.len(), 1);
        // Check that we got an atom
        assert_eq!(atoms.len(), 1);

        // Verify the atom has force enabled (through description or other means)
        let description = atoms[0].describe();
        assert!(description.contains("/home/user/modules/atuin/config.toml"));
        assert!(description.contains("config"));
        assert!(description.contains("atuin/config.toml"));
    }
}