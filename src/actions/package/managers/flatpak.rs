use crate::actions::package::PackageManager;
use crate::atoms::RunCommand;
use crate::platform::PlatformInfo;
use crate::{Atom, Result};
use std::process::Command;

/// Flatpak package manager for Linux
#[derive(Debug)]
pub struct Flatpak;

impl PackageManager for Flatpak {
    fn name(&self) -> &'static str {
        "flatpak"
    }

    fn supports(&self, platform: &PlatformInfo) -> bool {
        // Flatpak works on all Linux distributions
        platform.is_linux()
    }

    fn is_available(&self) -> bool {
        Command::new("flatpak")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn is_installed(&self, package: &str) -> bool {
        Command::new("flatpak")
            .args(["list", "--app"])
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

        // Flatpak needs to be installed via system package manager
        // This is handled by the system package manager detection
        Ok(vec![])
    }

    fn install(&self, packages: Vec<String>) -> Result<Box<dyn Atom>> {
        let mut args = vec![
            "install".to_string(),
            "-y".to_string(),
            "--noninteractive".to_string(),
        ];
        args.extend(packages);

        Ok(Box::new(RunCommand {
            command: "flatpak".to_string(),
            args: Some(args),
            cwd: None,
            env: None,
            shell: None,
            privilege_escalation: None,
        }))
    }

    fn needs_privilege_escalation(&self) -> bool {
        false // Flatpak runs in user space
    }

    fn priority(&self) -> i32 {
        5 // Lower priority than native package managers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatpak_name() {
        assert_eq!(Flatpak.name(), "flatpak");
    }

    #[test]
    fn test_flatpak_needs_privilege_escalation() {
        assert!(!Flatpak.needs_privilege_escalation());
    }
}
