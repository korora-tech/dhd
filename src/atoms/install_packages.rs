use crate::atoms::Atom;
use super::package::PackageManager;

#[derive(Debug, Clone)]
pub struct InstallPackages {
    pub packages: Vec<String>,
    pub manager: Option<PackageManager>,
}

impl Atom for InstallPackages {
    fn name(&self) -> &str {
        "InstallPackages"
    }

    fn execute(&self) -> Result<(), String> {
        if self.packages.is_empty() {
            return Ok(());
        }
        
        let manager = if let Some(mgr) = &self.manager {
            mgr.clone()
        } else {
            // Auto-detect package manager
            PackageManager::detect()
                .ok_or_else(|| "No supported package manager found".to_string())?
        };
        
        let provider = manager.get_provider();
        
        // Filter out already installed packages
        let mut packages_to_install = Vec::new();
        for package in &self.packages {
            match provider.is_package_installed(package) {
                Ok(true) => {
                    // Silently skip already installed packages
                }
                Ok(false) => {
                    packages_to_install.push(package.clone());
                }
                Err(_e) => {
                    // If we can't check, assume it needs to be installed
                    packages_to_install.push(package.clone());
                }
            }
        }
        
        if packages_to_install.is_empty() {
            return Ok(());
        }
        
        // Install each package
        for package in &packages_to_install {
            match provider.install_package(package) {
                Ok(_) => {},
                Err(e) => {
                    return Err(format!("Failed to install package {}: {}", package, e));
                }
            }
        }
        
        Ok(())
    }

    fn describe(&self) -> String {
        let manager_str = if let Some(mgr) = &self.manager {
            let provider = mgr.get_provider();
            format!(" ({})", provider.name())
        } else {
            String::new()
        };
        
        if self.packages.is_empty() {
            format!("Install packages{}: (none)", manager_str)
        } else if self.packages.len() == 1 {
            format!("Install package{}: {}", manager_str, self.packages[0])
        } else {
            format!("Install packages{}: {}", manager_str, self.packages.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_packages_name() {
        let atom = InstallPackages {
            packages: vec!["vim".to_string()],
            manager: None,
        };
        assert_eq!(atom.name(), "InstallPackages");
    }

    #[test]
    fn test_install_packages_execute_empty() {
        let atom = InstallPackages {
            packages: vec![],
            manager: None,
        };
        
        // Should succeed even with empty package list
        let result = atom.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_install_packages_execute_single() {
        let atom = InstallPackages {
            packages: vec!["vim".to_string()],
            manager: None,
        };
        
        // Currently just prints, should succeed
        let result = atom.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_install_packages_execute_multiple() {
        let atom = InstallPackages {
            packages: vec!["vim".to_string(), "git".to_string(), "curl".to_string()],
            manager: None,
        };
        
        // Currently just prints, should succeed
        let result = atom.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_install_packages_clone() {
        let atom = InstallPackages {
            packages: vec!["vim".to_string()],
            manager: Some(PackageManager::Apt),
        };
        
        let cloned = atom.clone();
        assert_eq!(cloned.packages, atom.packages);
        assert_eq!(cloned.manager, atom.manager);
        assert_eq!(cloned.name(), atom.name());
    }
}