use super::{PackageProvider, command_exists};

pub struct UvProvider;

impl PackageProvider for UvProvider {
    fn is_available(&self) -> bool {
        command_exists("uv")
    }

    fn is_package_installed(&self, package: &str) -> Result<bool, String> {
        use std::process::Command;
        
        // uv tool list shows all installed tools
        let output = Command::new("uv")
            .args(&["tool", "list"])
            .output()
            .map_err(|e| format!("Failed to run uv tool list: {}", e))?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Check if the package name appears in the list
            Ok(stdout.lines().any(|line| line.contains(package)))
        } else {
            Err(format!(
                "Failed to list uv tools: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    fn install_package(&self, package: &str) -> Result<(), String> {
        use std::process::Command;
        
        let output = Command::new("uv")
            .args(&["tool", "install", package])
            .output()
            .map_err(|e| format!("Failed to run uv tool install: {}", e))?;
        
        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "Failed to install package: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    fn uninstall_package(&self, package: &str) -> Result<(), String> {
        use std::process::Command;
        
        let output = Command::new("uv")
            .args(&["tool", "uninstall", package])
            .output()
            .map_err(|e| format!("Failed to run uv tool uninstall: {}", e))?;
        
        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "Failed to uninstall package: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    fn update(&self) -> Result<(), String> {
        use std::process::Command;
        
        // uv tool upgrade upgrades all installed tools
        let output = Command::new("uv")
            .args(&["tool", "upgrade", "--all"])
            .output()
            .map_err(|e| format!("Failed to run uv tool upgrade: {}", e))?;
        
        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "Failed to upgrade tools: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    fn name(&self) -> &str {
        "uv"
    }

    fn install_command(&self) -> Vec<String> {
        vec!["uv".to_string(), "tool".to_string(), "install".to_string()]
    }
}