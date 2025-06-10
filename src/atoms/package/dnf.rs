use super::{PackageProvider, command_exists};

pub struct DnfProvider;

impl PackageProvider for DnfProvider {
    fn is_available(&self) -> bool {
        command_exists("dnf")
    }
    
    fn is_package_installed(&self, _package: &str) -> Result<bool, String> {
        todo!("Implement dnf package check")
    }
    
    fn install_package(&self, _package: &str) -> Result<(), String> {
        todo!("Implement dnf install")
    }
    
    fn uninstall_package(&self, _package: &str) -> Result<(), String> {
        todo!("Implement dnf uninstall")
    }
    
    fn update(&self) -> Result<(), String> {
        todo!("Implement dnf update")
    }
    
    fn name(&self) -> &str {
        "dnf"
    }
    
    fn install_command(&self) -> Vec<String> {
        vec!["dnf".to_string(), "install".to_string(), "-y".to_string()]
    }
}