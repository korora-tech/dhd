use super::{PackageProvider, command_exists};

pub struct NpmProvider;

impl PackageProvider for NpmProvider {
    fn is_available(&self) -> bool {
        command_exists("npm")
    }

    fn is_package_installed(&self, _package: &str) -> Result<bool, String> {
        todo!("Implement npm package check")
    }

    fn install_package(&self, _package: &str) -> Result<(), String> {
        todo!("Implement npm install")
    }

    fn uninstall_package(&self, _package: &str) -> Result<(), String> {
        todo!("Implement npm uninstall")
    }

    fn update(&self) -> Result<(), String> {
        todo!("Implement npm update")
    }

    fn name(&self) -> &str {
        "npm"
    }

    fn install_command(&self) -> Vec<String> {
        vec!["npm".to_string(), "install".to_string(), "-g".to_string()]
    }
}
