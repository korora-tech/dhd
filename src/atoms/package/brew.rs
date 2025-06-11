use super::{PackageProvider, command_exists};
use std::process::Command;

pub struct BrewProvider;

impl PackageProvider for BrewProvider {
    fn is_available(&self) -> bool {
        command_exists("brew")
    }

    fn is_package_installed(&self, package: &str) -> Result<bool, String> {
        let output = Command::new("brew")
            .args(["list", "--formula", package])
            .output()
            .map_err(|e| format!("Failed to check package status: {}", e))?;

        Ok(output.status.success())
    }

    fn install_package(&self, package: &str) -> Result<(), String> {
        let output = Command::new("brew")
            .args(["install", package])
            .output()
            .map_err(|e| format!("Failed to install package: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Failed to install {}: {}",
                package,
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    fn uninstall_package(&self, package: &str) -> Result<(), String> {
        let output = Command::new("brew")
            .args(["uninstall", package])
            .output()
            .map_err(|e| format!("Failed to uninstall package: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Failed to uninstall {}: {}",
                package,
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    fn update(&self) -> Result<(), String> {
        let output = Command::new("brew")
            .args(["update"])
            .output()
            .map_err(|e| format!("Failed to update brew: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Failed to update brew: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "brew"
    }

    fn install_command(&self) -> Vec<String> {
        vec!["brew".to_string(), "install".to_string()]
    }
}
