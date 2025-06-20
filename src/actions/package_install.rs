use crate::ActionType;
use crate::atoms::AtomCompat;
use crate::atoms::package::PackageManager;
use dhd_macros::{typescript_fn, typescript_type};

use super::Action;

#[typescript_type]
pub struct PackageInstall {
    pub names: Vec<String>,
    pub manager: Option<PackageManager>,
}

#[typescript_fn]
pub fn package_install(config: PackageInstall) -> ActionType {
    ActionType::PackageInstall(config)
}

impl Action for PackageInstall {
    fn name(&self) -> &str {
        "PackageInstall"
    }

    fn plan(&self, _module_dir: &std::path::Path) -> Vec<Box<dyn crate::atom::Atom>> {
        vec![Box::new(AtomCompat::new(
            Box::new(crate::atoms::InstallPackages {
                packages: self.names.clone(),
                manager: self.manager.clone(),
            }),
            "package_install".to_string(),
        ))]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_install_creation() {
        let packages = vec!["docker".to_string(), "docker-compose".to_string(), "kubectl".to_string()];
        let action = PackageInstall {
            names: packages.clone(),
            manager: None,
        };

        assert_eq!(action.names, packages);
        assert_eq!(action.manager, None);
    }

    #[test]
    fn test_package_install_helper_function() {
        let action = package_install(PackageInstall {
            names: vec!["postgresql-client".to_string(), "redis-tools".to_string()],
            manager: None,
        });

        match action {
            ActionType::PackageInstall(pkg) => {
                assert_eq!(pkg.names, vec!["postgresql-client".to_string(), "redis-tools".to_string()]);
                assert_eq!(pkg.manager, None);
            }
            _ => panic!("Expected PackageInstall action type"),
        }
    }

    #[test]
    fn test_package_install_name() {
        let action = PackageInstall {
            names: vec!["nginx".to_string()],
            manager: None,
        };

        assert_eq!(action.name(), "PackageInstall");
    }

    #[test]
    fn test_package_install_plan() {
        let action = PackageInstall {
            names: vec!["rustup".to_string(), "cargo-watch".to_string(), "cargo-edit".to_string()],
            manager: None,
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
        // Check that we got an atom (can't check name directly due to AtomCompat wrapper)
        assert_eq!(atoms.len(), 1);
    }

    #[test]
    fn test_package_install_with_manager() {
        let action = PackageInstall {
            names: vec!["@nestjs/cli".to_string(), "@angular/cli".to_string(), "vite".to_string()],
            manager: Some(PackageManager::Bun),
        };

        assert_eq!(action.names, vec!["@nestjs/cli".to_string(), "@angular/cli".to_string(), "vite".to_string()]);
        assert_eq!(action.manager, Some(PackageManager::Bun));

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
    }
}
