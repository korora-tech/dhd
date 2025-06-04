use crate::actions::package::PackageManager;
use crate::atoms::RunCommand;
use crate::platform::PlatformInfo;
use crate::{Atom, Result};
use std::process::Command;

/// Cargo package manager for Rust
#[derive(Debug)]
pub struct Cargo;

impl PackageManager for Cargo {
    fn name(&self) -> &'static str {
        "cargo"
    }

    fn supports(&self, _platform: &PlatformInfo) -> bool {
        // Cargo works on all platforms where Rust is available
        true
    }

    fn is_available(&self) -> bool {
        Command::new("cargo")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn is_installed(&self, package: &str) -> bool {
        // Check if a cargo binary is installed
        Command::new("cargo")
            .args(["install", "--list"])
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

        // Install Rust/Cargo using rustup
        Ok(vec![Box::new(RunCommand {
            command: "/bin/bash".to_string(),
            args: Some(vec![
                "-c".to_string(),
                "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"
                    .to_string(),
            ]),
            cwd: None,
            env: None,
            shell: None,
        })])
    }

    fn install(&self, packages: Vec<String>) -> Result<Box<dyn Atom>> {
        let mut args = vec!["install".to_string()];
        args.extend(packages);

        Ok(Box::new(RunCommand {
            command: "cargo".to_string(),
            args: Some(args),
            cwd: None,
            env: None,
            shell: None,
        }))
    }

    fn needs_privilege_escalation(&self) -> bool {
        false
    }

    fn priority(&self) -> i32 {
        5 // Lower priority than system package managers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cargo_name() {
        assert_eq!(Cargo.name(), "cargo");
    }

    #[test]
    fn test_cargo_needs_privilege_escalation() {
        assert!(!Cargo.needs_privilege_escalation());
    }
}
