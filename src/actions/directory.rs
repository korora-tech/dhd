use dhd_macros::{typescript_fn, typescript_type};

use std::path::{Path, PathBuf};
use crate::atoms::AtomCompat;

#[typescript_type]
pub struct Directory {
    pub path: String,
    pub requires_privilege_escalation: Option<bool>,
}

impl crate::actions::Action for Directory {
    fn name(&self) -> &str {
        "Directory"
    }

    fn plan(&self, _module_dir: &Path) -> Vec<Box<dyn crate::atom::Atom>> {
        let directory_path = if self.path.starts_with("~/") {
            let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/home/user"));
            PathBuf::from(self.path.replacen("~/", &format!("{}/", home), 1))
        } else {
            PathBuf::from(&self.path)
        };

        vec![Box::new(AtomCompat::new(
            Box::new(crate::atoms::create_directory::CreateDirectory::new(
                directory_path,
                self.requires_privilege_escalation.unwrap_or(false),
            )),
            "directory".to_string(),
        ))]
    }
}

#[typescript_fn]
pub fn directory(config: Directory) -> crate::actions::ActionType {
    crate::actions::ActionType::Directory(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::Action;

    #[test]
    fn test_directory_creation() {
        let action = Directory {
            path: "/tmp/test".to_string(),
            requires_privilege_escalation: Some(false),
        };

        assert_eq!(action.path, "/tmp/test");
        assert_eq!(action.requires_privilege_escalation, Some(false));
    }

    #[test]
    fn test_directory_helper_function() {
        let action = directory(Directory {
            path: "/home/user/.config".to_string(),
            requires_privilege_escalation: None,
        });

        match action {
            crate::actions::ActionType::Directory(dir) => {
                assert_eq!(dir.path, "/home/user/.config");
                assert_eq!(dir.requires_privilege_escalation, None);
            }
            _ => panic!("Expected Directory action type"),
        }
    }

    #[test]
    fn test_directory_name() {
        let action = Directory {
            path: "/tmp/test".to_string(),
            requires_privilege_escalation: None,
        };

        assert_eq!(action.name(), "Directory");
    }

    #[test]
    fn test_directory_plan() {
        let action = Directory {
            path: "/tmp/test".to_string(),
            requires_privilege_escalation: None,
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
        // Check that we got an atom
        assert_eq!(atoms.len(), 1);
    }

    #[test]
    fn test_directory_with_privilege_escalation() {
        let action = Directory {
            path: "/etc/test".to_string(),
            requires_privilege_escalation: Some(true),
        };

        assert_eq!(action.requires_privilege_escalation, Some(true));

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
    }

    #[test]
    fn test_directory_home_expansion() {
        unsafe { std::env::set_var("HOME", "/home/testuser"); }

        let action = Directory {
            path: "~/test".to_string(),
            requires_privilege_escalation: None,
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
        assert!(atoms[0].describe().contains("/home/testuser/test"));
    }
}