use crate::actions::package::PackageManager;
use crate::atoms::RunCommand;
use crate::platform::PlatformInfo;
use crate::{Atom, Result};
use std::process::Command;

/// Homebrew package manager for macOS and Linux
#[derive(Debug)]
pub struct Brew;

impl PackageManager for Brew {
    fn name(&self) -> &'static str {
        "brew"
    }

    fn supports(&self, platform: &PlatformInfo) -> bool {
        // Brew supports macOS and Linux (but not Windows)
        platform.is_os(os_info::Type::Macos) || platform.is_linux()
    }

    fn is_available(&self) -> bool {
        Command::new("brew")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn is_installed(&self, package: &str) -> bool {
        Command::new("brew")
            .args(&["list", "--formula", package])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn bootstrap(&self) -> Result<Vec<Box<dyn Atom>>> {
        if self.is_available() {
            return Ok(vec![]);
        }

        // Install Homebrew
        Ok(vec![Box::new(RunCommand {
            command: "/bin/bash".to_string(),
            args: Some(vec![
                "-c".to_string(),
                "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
                    .to_string(),
            ]),
            cwd: None,
            env: Some(std::collections::HashMap::from([(
                "NONINTERACTIVE".to_string(),
                "1".to_string(),
            )])),
        })])
    }

    fn install(&self, packages: Vec<String>) -> Result<Box<dyn Atom>> {
        let mut args = vec!["install".to_string()];
        args.extend(packages);

        Ok(Box::new(RunCommand {
            command: "brew".to_string(),
            args: Some(args),
            cwd: None,
            env: None,
        }))
    }

    fn needs_privilege_escalation(&self) -> bool {
        false
    }

    fn priority(&self) -> i32 {
        15 // Higher priority on macOS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brew_name() {
        assert_eq!(Brew.name(), "brew");
    }

    #[test]
    fn test_brew_needs_privilege_escalation() {
        assert!(!Brew.needs_privilege_escalation());
    }
}
