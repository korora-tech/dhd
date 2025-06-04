use crate::actions::package::PackageManager;
use crate::atoms::RunCommand;
use crate::platform::PlatformInfo;
use crate::{Atom, Result};
use std::process::Command;

/// Windows Package Manager (winget)
#[derive(Debug)]
pub struct Winget;

impl PackageManager for Winget {
    fn name(&self) -> &'static str {
        "winget"
    }

    fn supports(&self, platform: &PlatformInfo) -> bool {
        platform.is_os(os_info::Type::Windows)
    }

    fn is_available(&self) -> bool {
        Command::new("winget")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn is_installed(&self, package: &str) -> bool {
        Command::new("winget")
            .args(&["list", "--id", package])
            .output()
            .map(|output| {
                output.status.success() && String::from_utf8_lossy(&output.stdout).contains(package)
            })
            .unwrap_or(false)
    }

    fn bootstrap(&self) -> Result<Vec<Box<dyn Atom>>> {
        if self.is_available() {
            return Ok(vec![]);
        }

        // Winget should be pre-installed on Windows 10/11
        // If not available, it needs to be installed via Microsoft Store or manually
        Ok(vec![])
    }

    fn install(&self, packages: Vec<String>) -> Result<Box<dyn Atom>> {
        let mut args = vec!["install".to_string()];

        // Add each package with --id flag for exact matching
        for package in packages {
            args.push("--id".to_string());
            args.push(package);
        }

        // Accept agreements and source licenses
        args.push("--accept-package-agreements".to_string());
        args.push("--accept-source-agreements".to_string());
        args.push("--silent".to_string());

        Ok(Box::new(RunCommand {
            command: "winget".to_string(),
            args: Some(args),
            cwd: None,
            env: None,
            shell: None,
        }))
    }

    fn needs_privilege_escalation(&self) -> bool {
        false // Winget handles UAC prompts internally when needed
    }

    fn priority(&self) -> i32 {
        15 // Primary package manager on Windows
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_winget_name() {
        assert_eq!(Winget.name(), "winget");
    }

    #[test]
    fn test_winget_needs_privilege_escalation() {
        assert!(!Winget.needs_privilege_escalation());
    }
}
