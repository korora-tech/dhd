use super::{PackageProvider, command_exists};

pub struct GoProvider;

impl PackageProvider for GoProvider {
    fn is_available(&self) -> bool {
        command_exists("go")
    }
    
    fn is_package_installed(&self, _package: &str) -> Result<bool, String> {
        todo!("Implement go package check")
    }
    
    fn install_package(&self, _package: &str) -> Result<(), String> {
        todo!("Implement go install")
    }
    
    fn uninstall_package(&self, _package: &str) -> Result<(), String> {
        Err("Go does not support uninstalling packages".to_string())
    }
    
    fn update(&self) -> Result<(), String> {
        Ok(()) // Go doesn't have a traditional update command
    }
    
    fn name(&self) -> &str {
        "go"
    }
    
    fn install_command(&self) -> Vec<String> {
        vec!["go".to_string(), "install".to_string()]
    }
}