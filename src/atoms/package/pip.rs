use super::{PackageProvider, command_exists};

pub struct PipProvider;

impl PackageProvider for PipProvider {
    fn is_available(&self) -> bool {
        command_exists("pip3") || command_exists("pip")
    }

    fn is_package_installed(&self, _package: &str) -> Result<bool, String> {
        todo!("Implement pip package check")
    }

    fn install_package(&self, _package: &str) -> Result<(), String> {
        todo!("Implement pip install")
    }

    fn uninstall_package(&self, _package: &str) -> Result<(), String> {
        todo!("Implement pip uninstall")
    }

    fn update(&self) -> Result<(), String> {
        Ok(()) // pip doesn't have a traditional update command
    }

    fn name(&self) -> &str {
        "pip"
    }

    fn install_command(&self) -> Vec<String> {
        vec!["pip3".to_string(), "install".to_string()]
    }
}
