use super::{PackageProvider, command_exists};

pub struct GoProvider;

impl PackageProvider for GoProvider {
    fn is_available(&self) -> bool {
        command_exists("go")
    }

    fn is_package_installed(&self, package: &str) -> Result<bool, String> {
        // For Go packages, we check if the binary exists in GOPATH/bin
        // Extract the binary name from the package path (e.g., github.com/user/tool -> tool)
        let binary_name = package.split('/').last().unwrap_or(package);
        let binary_name = binary_name.trim_end_matches("@latest");
        
        // Check in PATH (which should include GOPATH/bin)
        Ok(command_exists(binary_name))
    }

    fn install_package(&self, package: &str) -> Result<(), String> {
        use std::process::Command;
        
        let output = Command::new("go")
            .args(&["install", package])
            .output()
            .map_err(|e| format!("Failed to run go install: {}", e))?;
        
        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "Failed to install package: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
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
