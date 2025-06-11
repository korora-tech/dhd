use super::{PackageProvider, command_exists};
use std::process::Command;

pub struct AptProvider;

impl PackageProvider for AptProvider {
    fn is_available(&self) -> bool {
        command_exists("apt-get")
    }

    fn is_package_installed(&self, package: &str) -> Result<bool, String> {
        let output = Command::new("dpkg")
            .arg("-l")
            .arg(package)
            .output()
            .map_err(|e| format!("Failed to check package status: {}", e))?;

        Ok(output.status.success())
    }

    fn install_package(&self, package: &str) -> Result<(), String> {
        let output = Command::new("sudo")
            .args(["apt-get", "install", "-y", package])
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
        let output = Command::new("sudo")
            .args(["apt-get", "remove", "-y", package])
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
        let output = Command::new("sudo")
            .args(["apt-get", "update"])
            .output()
            .map_err(|e| format!("Failed to update package database: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Failed to update package database: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "apt"
    }

    fn install_command(&self) -> Vec<String> {
        vec![
            "apt-get".to_string(),
            "install".to_string(),
            "-y".to_string(),
        ]
    }
}
