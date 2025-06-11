use super::{PackageProvider, command_exists};

pub struct SnapProvider;

impl PackageProvider for SnapProvider {
    fn is_available(&self) -> bool {
        command_exists("snap")
    }

    fn is_package_installed(&self, _package: &str) -> Result<bool, String> {
        todo!("Implement snap package check")
    }

    fn install_package(&self, _package: &str) -> Result<(), String> {
        todo!("Implement snap install")
    }

    fn uninstall_package(&self, _package: &str) -> Result<(), String> {
        todo!("Implement snap uninstall")
    }

    fn update(&self) -> Result<(), String> {
        todo!("Implement snap update")
    }

    fn name(&self) -> &str {
        "snap"
    }

    fn install_command(&self) -> Vec<String> {
        vec!["snap".to_string(), "install".to_string()]
    }
}
